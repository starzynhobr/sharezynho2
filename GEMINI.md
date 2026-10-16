# GEMINI.md — Sharezynho 2

Regras específicas deste repositório. Elas complementam o `~/.gemini/GEMINI.md` global
(exploração, escopo, verificação, git) e têm prioridade sobre ele em caso de conflito.

Responda ao usuário em português do Brasil. Código, identificadores e mensagens de commit em inglês.

---

## 1. O que é o projeto

App desktop Windows de chat de voz e compartilhamento de tela entre poucos amigos, P2P pela
rede Tailscale. O host sobe um servidor embutido; os convidados conectam nele.

**Stack real (confira antes de assumir outra coisa):**

- Shell: **Tauri 2** (`src-tauri/`), Rust edition 2024, runtime **Tokio**.
- UI: **React 19 + TypeScript 5 + Vite 6 + Tailwind CSS 4** (`src/`), `framer-motion`, `lucide-react`.
- Áudio: `cpal` (WASAPI). Tela: `windows-capture` + `jpeg-encoder`. Somente Windows.

## 2. Armadilhas do estado atual (leia antes de qualquer tarefa)

O projeto foi migrado de egui para Tauri e sobraram restos da versão antiga:

- `README.md` e `ARCHITECTURE.md` **descrevem a UI egui antiga** e caminhos `src/*.rs` que não
  existem mais. O protocolo, as portas e os pipelines de áudio/vídeo descritos continuam valendo;
  a parte de interface e os caminhos de arquivo, não. Em conflito, **o código vence a doc**.
- `Cargo.toml`, `Cargo.lock` e `target/` **na raiz são legado** (crate egui sem código-fonte).
  O crate verdadeiro é `src-tauri/`. Nunca rode `cargo` na raiz.
- `tests/integration_tests.rs` na raiz é órfão: importa `sharezynho::…`, mas não é compilado pelo
  crate `src-tauri`. Hoje **não há teste rodando**. Testes Rust novos vão em `src-tauri/tests/`
  ou em `#[cfg(test)]` no próprio módulo.
- Não existe ESLint, Prettier nem runner de teste de frontend configurado. Não invente comandos
  (`npm run lint`, `npm test`): eles não existem.
- `.gitignore` só cobre `/target`. Não adicione ao git `node_modules/`, `dist/` nem
  `src-tauri/target/`.

Não "arrume" nenhum desses itens por conta própria. Se a tarefa esbarrar em um deles, avise e
pergunte.

## 3. Mapa do código

| Caminho | Responsabilidade |
| --- | --- |
| `src-tauri/src/lib.rs` | Builder do Tauri, registro dos comandos, sobe o servidor MJPEG. |
| `src-tauri/src/commands.rs` | `AppState` e todos os `#[tauri::command]`; emite eventos para a UI. |
| `src-tauri/src/protocol.rs` | Mensagens TCP (JSON por linha), tags UDP e portas padrão. |
| `src-tauri/src/server.rs` | Servidor da sala no host: sinalização TCP + relay UDP. |
| `src-tauri/src/client.rs` | Sessão do participante: handshake, envio/recepção de áudio e vídeo. |
| `src-tauri/src/audio/` | `capture.rs` (mic, resample, VAD) e `playback.rs` (jitter buffer, deafen). |
| `src-tauri/src/screen/` | `capture.rs` (captura + JPEG + fragmentação) e `receiver.rs` (remontagem). |
| `src-tauri/src/stream_server.rs` | Servidor MJPEG local que entrega o vídeo recebido ao webview. |
| `src-tauri/src/network/tailscale.rs` | Descoberta do IP Tailscale / IP local preferido. |
| `src/api.ts` | **Única** porta de entrada do frontend para `invoke` e `listen`. |
| `src/types.ts` | Tipos TS espelhando os structs serializados pelo Rust. |
| `src/App.tsx`, `src/components/` | Estado da sala e as telas (rail, sidebar, canvas, diálogos). |

## 4. Contratos que não podem quebrar

**Fronteira Rust ↔ TypeScript.** Um comando existe em quatro lugares; mudou um, mude todos na
mesma tarefa:

1. a função `#[tauri::command]` em `commands.rs`;
2. a lista `generate_handler![…]` em `lib.rs`;
3. o wrapper em `src/api.ts`;
4. os tipos em `src/types.ts`.

- Argumentos: Rust em `snake_case`, JS em `camelCase` (`monitor_index` ↔ `monitorIndex`).
- Campos de payload/retorno ficam em `snake_case` nos dois lados (`is_speaking`, `stream_port`).
- Componentes React nunca chamam `invoke`/`listen` direto: sempre via `api`.
- Eventos existentes: `room-state`, `room-error`, `disconnected`. Todo `listen` precisa do
  `unlisten` no cleanup do `useEffect`.

**Protocolo de rede.** Host e convidados rodam o mesmo binário, mas em máquinas diferentes:
qualquer mudança em `protocol.rs` precisa ser aplicada em `server.rs` **e** `client.rs`, e
quebra compatibilidade com builds antigos (diga isso na resposta final).

- TCP 9765: JSON delimitado por `\n`. UDP 9766: primeiro byte é a tag (`TAG_AUDIO` 0x01,
  `TAG_VIDEO` 0x02). MJPEG local 9768, apenas em `127.0.0.1`.
- Use as constantes de `protocol.rs` / `stream_server.rs`; não repita números de porta.
- Pacotes UDP devem caber no MTU do WireGuard: payload de vídeo ~1200 bytes.
- Áudio na rede: PCM 16 kHz mono, frames de 20 ms.

## 5. Regras de Rust

- Comandos retornam `Result<T, String>` com mensagem legível; a UI mostra esse texto.
- Nada de `unwrap()`/`expect()` novos em caminho de runtime (rede, dispositivo de áudio, captura).
  Dispositivo ausente, porta ocupada e peer desconectado são casos normais: trate e reporte.
- **Nunca segure um `std::sync::Mutex` através de um `.await`.** Copie/clone o que precisa, solte
  o guard e só então aguarde.
- Não bloqueie o runtime Tokio: nada de `std::thread::sleep`, I/O síncrono pesado ou encode de
  imagem dentro de task async. Use `tokio::time`, `spawn_blocking` ou thread dedicada.
- Callbacks de áudio do `cpal` são tempo real: sem alocação grande, sem lock demorado, sem I/O,
  sem `println!` por frame.
- Toda task/thread criada ao entrar na sala precisa ser encerrada no `leave_room` e em
  desconexão. Entrar e sair várias vezes não pode vazar socket, stream de áudio nem captura.
- Nunca confie em dado vindo da rede: valide tamanho antes de indexar, limite buffers de
  remontagem e descarte pacote malformado sem derrubar a sessão.
- Dependência nova só com justificativa; adicione em `src-tauri/Cargo.toml`, nunca no da raiz.

## 6. Regras de frontend

- TypeScript estrito: sem `any`, sem `@ts-ignore`, sem `as` para calar erro.
- Componentes de função com hooks. Estado da sala vive em `App.tsx` e desce por props; não crie
  store global nem biblioteca de estado sem pedir.
- Estilo com classes Tailwind CSS 4 e os tokens de `@theme` em `src/index.css`. Combine classes com
  `clsx` + `tailwind-merge`.
- Ícones: `lucide-react`. Animação: `framer-motion`. Não adicione outra lib para isso.
- Toda ação assíncrona precisa de três estados visíveis: carregando, erro e sucesso. Botão
  desabilitado enquanto o `invoke` está pendente.
- Estado da UI segue o backend: use o valor retornado pelo comando (ex.: `toggle_mic` retorna o
  novo estado) em vez de inverter um booleano local.
- Texto de interface em português do Brasil.
- O app roda dentro do webview do Tauri. Abrir `http://localhost:5173` no navegador **não** tem
  `invoke`; isso não prova que uma funcionalidade funciona.

## 7. Comandos

Rode a partir da raiz do repositório (PowerShell).

```powershell
# Frontend: tipos + build
npx tsc --noEmit
npm run build

# Rust: sempre apontando para o crate certo
cargo check  --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo fmt    --manifest-path src-tauri/Cargo.toml --check
cargo test   --manifest-path src-tauri/Cargo.toml

# App completo (abre janela e fica rodando; não use como verificação automática)
npm run tauri dev
```

## 8. Verificação obrigatória antes de dizer "pronto"

| Você mexeu em | Precisa passar |
| --- | --- |
| Qualquer `.ts` / `.tsx` / CSS | `npx tsc --noEmit` e `npm run build` |
| Qualquer `.rs` ou `Cargo.toml` | `cargo check`, `cargo clippy` e `cargo test` (do crate `src-tauri`) |
| Comando, evento ou payload | Os dois blocos acima, mais conferir os 4 pontos da seção 4 |

- Se um comando falhar, cole o erro real e corrija a causa. Não silencie com `#[allow]`,
  `any`, `// @ts-ignore`, nem removendo o teste.
- Se não conseguiu rodar uma verificação, diga exatamente qual e por quê.
- Áudio, captura de tela e conexão entre duas máquinas **não são verificáveis por você**. Nesses
  casos diga "compila e passa nos testes; falta teste manual de X" e liste os passos para o
  usuário testar. Nunca afirme que áudio/vídeo "está funcionando".

## 9. Testes

- Lógica pura deve ser testável sem hardware: serialização do protocolo, fragmentação e
  remontagem de frames, VAD, resample, jitter buffer, parsing de endereço. Ao mexer nelas,
  adicione ou atualize um teste.
- Se a lógica está presa dentro de um callback de dispositivo, extraia uma função pura e teste
  a função; não tente simular o dispositivo.
- Teste de rede usa `127.0.0.1` com **porta 0** (o SO escolhe), nunca 9765/9766 fixas, e sempre
  com `tokio::time::timeout` para não travar.
- Teste não pode depender de Tailscale instalado, microfone, monitor ou internet.
- Correção de bug vem com um teste que falhava antes e passa depois, quando a lógica for
  testável.
- Um teste precisa conseguir falhar: verifique valores concretos, não apenas "não deu panic".

## 10. Skills

As skills ficam em `C:\Users\tz\Documents\PROJETOSGIT\ai-agent-skills` e estão linkadas em
`~/.gemini/config/skills`. Se a skill não aparecer na sua lista, **abra o `SKILL.md` pelo caminho
abaixo com a ferramenta de leitura de arquivo e siga-o** — isso tem o mesmo efeito.

Leia a skill **antes** de começar o tipo de tarefa correspondente:

| Quando | Skill | Arquivo |
| --- | --- | --- |
| Criar, alterar ou revisar qualquer tela/componente | `ui-design` | `…\ai-agent-skills\design\ui-design\SKILL.md` |
| Algo quebrado, travando, com erro ou lento | `diagnosing-bugs` | `…\ai-agent-skills\shared\diagnosing-bugs\SKILL.md` |
| Criar ou mudar a fronteira de um módulo/API | `codebase-design` | `…\ai-agent-skills\shared\codebase-design\SKILL.md` |
| Decidir o que e como testar | `engineering-testing-strategy` | `…\ai-agent-skills\anthropic\engineering-testing-strategy\SKILL.md` |
| Revisar mudanças antes de entregar | `code-review` | `…\ai-agent-skills\shared\code-review\SKILL.md` |
| Escolher entre abordagens de arquitetura | `engineering-architecture` | `…\ai-agent-skills\anthropic\engineering-architecture\SKILL.md` |

(`…` = `C:\Users\tz\Documents\PROJETOSGIT`.) Uma skill pode apontar para arquivos em
`references/`; leia só os que ela mandar para a tarefa atual.

Diga na resposta final qual skill você usou. Não use skills de Flutter, Firebase, Python, Qt,
Blender, marketing ou produto neste projeto. Para CSS/HTML moderno, a skill embutida
`modern-web-guidance` também se aplica.

## 11. Como trabalhar aqui

1. Leia os arquivos da seção 3 que a tarefa toca **inteiros** antes de editar (são pequenos,
   nenhum passa de ~320 linhas).
2. Para tarefa que cruza Rust e React, escreva um plano de 3 a 6 passos e siga-o.
3. Faça a mudança de ponta a ponta: backend, `api.ts`, `types.ts` e a tela. Nada de função
   vazia, `todo!()`, dado falso/mock ou "implementar depois" deixado no código.
4. Edite o arquivo existente. Não crie `*_v2`, `*_new`, cópias de backup nem scripts soltos.
5. Rode a verificação da seção 8 e leia a saída.
6. Releia o próprio diff procurando: import não usado, log de debug, código morto, contrato da
   seção 4 pela metade.

Pare e pergunte antes de: trocar framework ou biblioteca central, mudar o formato do protocolo
de rede, apagar arquivos que você não criou, mexer nos itens legados da seção 2, ou fazer
qualquer operação git que altere histórico.
