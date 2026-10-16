# Arquitetura — Sharezynho 2 (Rust)

## Visão Geral

O Sharezynho 2 substitui toda a pilha pesada anterior (PySide6, LiveKit Server Go, libwebrtc C++ e dxcam em Python) por uma implementação nativa em **Rust** autossuficiente e ultraleve.

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        Rede Privada Tailscale                          │
│                      (100.64.0.0/10 WireGuard Mesh)                    │
└──────────────────────────────────┬─────────────────────────────────────┘
                                   │
      ┌────────────────────────────┴─────────────────────────────┐
      │                                                          │
┌─────▼──────────────────────────────┐          ┌────────────────▼────────────────────┐
│      Participante A (Host)         │          │       Participante B (Cliente)      │
│ ┌────────────────────────────────┐ │          │ ┌─────────────────────────────────┐ │
│ │ Servidor Embutido Tokio:       │ │          │ │ Cliente Tokio:                  │ │
│ │ • TCP 9765 (Sinalização/Salas) │ │◄──TCP────│ │ • Sinalização / Handshake       │ │
│ │ • UDP 9766 (Relay Áudio/Vídeo) │ │◄──UDP────│ │ • Streaming de Microfone e Tela │ │
│ └────────────────────────────────┘ │          │ └─────────────────────────────────┘ │
│ ┌────────────────────────────────┐ │          │ ┌─────────────────────────────────┐ │
│ │ WASAPI Audio + WinCapture (GPU)│ │          │ │ WASAPI Audio + Player egui GPU  │ │
│ └────────────────────────────────┘ │          │ └─────────────────────────────────┘ │
└────────────────────────────────────┘          └─────────────────────────────────────┘
```

---

## 1. Rede e Descoberta (Tailscale Local Server)

- O módulo [`src/network/tailscale.rs`](src/network/tailscale.rs) consulta a interface nativa do Tailscale via `tailscale ip -4` ou adaptadores locais.
- Ao criar a sala, o próprio aplicativo do Host escuta em `0.0.0.0:9765` (TCP) e `0.0.0.0:9766` (UDP).
- O Tailscale cria um túnel WireGuard criptografado ponto a ponto entre os computadores.
- Não há necessidade de servidores STUN/TURN em nuvem, contas ou portas abertas no roteador: o tráfego transita direto pela VPN do Tailscale.

---

## 2. Protocolo de Comunicação ([`src/protocol.rs`](src/protocol.rs))

- **TCP (Porta 9765)**: Linhas JSON delimitadas por quebra de linha (`\n`):
  - `Join`: Envia nome, sala e porta UDP local.
  - `Admitted`: Servidor confirma admissão e informa a porta UDP do relay.
  - `RoomState`: Sincroniza participantes, status de quem está falando (`is_speaking`) e apresentador de tela atual (`screen_presenter`).
  - `Speaking`: Notifica quando o usuário começou ou parou de falar.
  - `ScreenShareStarted` / `ScreenShareStopped`: Notifica início e término da transmissão de tela.
- **UDP (Porta 9766)**: Pacotes binários em tempo real:
  - `TAG_AUDIO (0x01)`: Amostras PCM 16kHz mono de 20ms com detecção de fala (VAD).
  - `TAG_VIDEO (0x02)`: Chunks de ~1200 bytes contendo frames JPEG comprimidos a 30 FPS.

---

## 3. Pipeline de Áudio ([`src/audio/`](src/audio/))

- **Captura (`capture.rs`)**:
  - Captura via WASAPI nativo com a crate `cpal`.
  - Converte canais para mono e efetua downsampling limpo de 48kHz para 16kHz.
  - **VAD (Voice Activity Detection)**: Calcula a energia RMS do frame de 20ms. Silêncio não consome banda (zero pacotes enviados). Quando o usuário fala, o avatar ganha contorno azul luminoso em tempo real.
- **Reprodução (`playback.rs`)**:
  - Buffer de áudio com controle de jitter (máximo 150ms para evitar acúmulo de atraso).
  - Upsampling de 16kHz para 48kHz estéreo para a saída padrão do sistema.
  - Suporte a *Deafen* (silenciamento imediato).

---

## 4. Pipeline de Captura e Vídeo ([`src/screen/`](src/screen/))

- **Captura (`capture.rs`)**:
  - Utiliza `windows-capture`, que usa a API oficial **Windows Graphics Capture (DirectX 11 / DXGI)**.
  - Aceleração por GPU, captura do cursor do mouse e enumeração de múltiplos monitores.
  - Compressão rápida com `jpeg-encoder` em qualidade ajustada para 30 FPS fluidos.
  - Fragmentação em pacotes UDP de tamanho compatível com o MTU do WireGuard (~1200 bytes).
- **Recepção (`receiver.rs`)**:
  - Remontagem de pacotes UDP indexados por `frame_id`.
  - Decodificação rápida de JPEG para `egui::ColorImage`.
  - Atualização direta de textura GPU no egui via `request_repaint()`.

---

## 5. Interface Gráfica ([`src/ui/`](src/ui/))

- Desenvolvida em `eframe` / `egui` (Immediate Mode GUI nativo).
- Consumo mínimo de memória (~30 MB de RAM em repouso) e zero vazamentos de ciclo de vida.
- Layout de 3 colunas com dock inferior flutuante idêntico à identidade visual desejada.
