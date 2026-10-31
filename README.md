# Sharezynho 2 (Rust Edition) 🦀

Aplicativo desktop ultraleve e de alta performance para compartilhamento de tela e chat de voz em tempo real, utilizando a rede privada do **Tailscale** como servidor local mesh P2P.

Backend em **Rust** e interface em **Tauri 2 + React**, sem dependências externas pesadas (sem Python, sem PySide6/Qt, sem download de binários externos de LiveKit).

---

## ✨ Principais Diferenciais em Relação à Versão Anterior

| Aspecto | Versão Anterior (Python) | Sharezynho 2 (Rust) |
| :--- | :--- | :--- |
| **Executável** | Múltiplas pastas/venv (~400 MB) | **Binário único nativo (~21 MB)** |
| **Consumo de Memória (RAM)** | ~250 MB – 350 MB | **~25 MB – 45 MB** |
| **Inicialização** | 3 – 5 segundos | **Instantânea (< 0.1s)** |
| **Servidor de Mídia** | Processo Go externo (`livekit-server.exe` ~60MB) | **Servidor embutido Tokio (TCP 9765 + UDP 9766)** |
| **Captura de Tela** | Python DXcam com conversões pesadas | **Windows Graphics Capture (DirectX hardware accelerated)** |
| **Áudio / Voz** | sounddevice + libwebrtc C++ bindings | **WASAPI nativo via `cpal` com eco cancelado, redução de ruído e VAD (`sonora`)** |
| **Interface Gráfica** | PySide6 / Qt6 (pesado, lento) | **Tauri 2 + React 19 + Tailwind CSS v4** |

---

## 🚀 Como Usar

### 1. Compilar / Executar

No diretório do projeto:

```powershell
# Instala dependências e abre em modo de desenvolvimento
npm install
npm run tauri dev

# Gera o executável de produção
npm run tauri build
```

---

### 2. Quem Cria a Sala (Host)
1. Abra o aplicativo.
2. Clique em **"Entrar ou criar sala"**.
3. Selecione **"Criar Sala (Host)"**, informe seu nome e código da sala (ou use o código aleatório gerado).
4. Clique em **"Criar Sala"**.
5. O aplicativo detecta seu IP do Tailscale (ex: `100.125.168.118`) e inicia o servidor local embutido em segundo plano.
6. Clique no botão **"🔗 Convidar"** no topo da tela para copiar um convite único e enviar aos seus amigos.

---

### 3. Quem Entra na Sala (Convidado)
1. Abra o aplicativo no outro computador conectado à mesma rede Tailscale.
2. Clique em **"Entrar ou criar sala"**.
3. Selecione **"Entrar em Sala"**.
4. Informe seu nome e cole o convite recebido (ex: `sharezynho://100.125.168.118:9765/A3F91C`).
5. Clique em **"Entrar"**.

---

## 🛠️ Controles na Chamada (Dock Inferior)
- 🎤 / 🔇 **Microfone**: Ativa ou silencia seu microfone com detecção de voz (VAD) automática.
- 🎧 / 🔇 **Áudio / Deafen**: Silencia o áudio recebido dos outros participantes.
- 🖥️ **Compartilhar Tela**: Transmite seu monitor selecionado via hardware acceleration DirectX em até 30 FPS, limitado a 1920 px de largura e cerca de 4 MB/s.
- 📞 **Sair da Sala**: Encerra a sessão e o servidor local de forma limpa.

---

## 📦 Gerar e Publicar uma Versão

```powershell
# Gera, assina e publica; quem tem o app instalado atualiza ao abrir
.\scripts\release.ps1 -Version 0.2.0 -Notes "O que mudou"

# Apenas gera o instalador em src-tauri/target/release/bundle/nsis
.\scripts\release.ps1 -Version 0.2.0 -BuildOnly
```

A chave privada e a senha ficam em `~/.tauri/sharezynho.key` e `.password`. Sem ela, nenhuma atualização poderá ser publicada para quem já instalou.

---

## 🧪 Testes

Para rodar a suíte de testes automatizados (unitários e ponta a ponta):

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

## macOS

Requer macOS 12.3 ou mais novo. O `.dmg` universal (Intel e Apple Silicon) é gerado pelo workflow **Build macOS** e fica na página de Releases.

O app não tem assinatura da Apple, então libere-o uma vez depois de copiá-lo para Aplicativos:

```bash
xattr -cr /Applications/Sharezynho.app
```

Permita o **Microfone** na primeira chamada. Ao compartilhar a tela, permita **Gravação de Tela** em Ajustes do Sistema > Privacidade e Segurança e reabra o app.

Para compilar no próprio Mac, instale Node.js, Rust e as ferramentas do Xcode, e rode:

```bash
npm ci
```

```bash
npm run tauri build -- --bundles app,dmg --config '{"bundle":{"createUpdaterArtifacts":false}}'
```
