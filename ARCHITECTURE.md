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
│ │ WASAPI Audio + WinCapture (GPU)│ │          │ │ WASAPI Audio + Player MJPEG local  │ │
│ └────────────────────────────────┘ │          │ └─────────────────────────────────┘ │
└────────────────────────────────────┘          └─────────────────────────────────────┘
```

---

## 1. Rede e Descoberta (Tailscale Local Server)

- O módulo [`src-tauri/src/network/tailscale.rs`](src-tauri/src/network/tailscale.rs) consulta a interface nativa do Tailscale via `tailscale ip -4` ou adaptadores locais.
- Ao criar a sala, o próprio aplicativo do Host escuta em `0.0.0.0:9765` (TCP) e `0.0.0.0:9766` (UDP).
- O Tailscale cria um túnel WireGuard criptografado ponto a ponto entre os computadores.
- Não há necessidade de servidores STUN/TURN em nuvem, contas ou portas abertas no roteador: o tráfego transita direto pela VPN do Tailscale.

---

## 2. Protocolo de Comunicação ([`src-tauri/src/protocol.rs`](src-tauri/src/protocol.rs))

- **TCP (Porta 9765)**: Linhas JSON delimitadas por quebra de linha (`\n`):
  - `Join`: Envia nome, sala e porta UDP local.
  - `Admitted`: Servidor confirma admissão e informa a porta UDP do relay.
  - `RoomState`: Sincroniza participantes, status de quem está falando (`is_speaking`) e apresentador de tela atual (`screen_presenter`).
  - `Speaking`: Notifica quando o usuário começou ou parou de falar.
  - `ScreenShareStarted` / `ScreenShareStopped`: Notifica início e término da transmissão de tela.
- **UDP (Porta 9766)**: Pacotes binários em tempo real:
  - `TAG_AUDIO (0x01)`: ID do remetente (u32) seguido de PCM 16kHz mono de 20ms, enviado só com fala detectada (VAD).
  - `TAG_VIDEO (0x02)`: ID da transmissão, ID do frame, índice e total de chunks, seguidos de ~1200 bytes de JPEG.
  - O relay só aceita pacotes de participantes admitidos.

---

## 3. Pipeline de Áudio ([`src-tauri/src/audio/`](src-tauri/src/audio/))

- **Captura (`capture.rs`)**:
  - Captura via WASAPI nativo com a crate `cpal`.
  - Converte canais para mono e reamostra a taxa do dispositivo para 16kHz.
  - **VAD (Voice Activity Detection)**: Calcula a energia RMS do frame de 20ms. Silêncio não consome banda (zero pacotes enviados). Quando o usuário fala, o avatar ganha contorno azul luminoso em tempo real.
- **Processamento (`processing.rs`)**:
  - `sonora` (porte Rust do WebRTC) aplica cancelamento de eco, supressão de ruído e controle de ganho.
- **Reprodução (`playback.rs`)**:
  - Uma fila por participante, mixadas na reprodução, com descarte acima de 300ms para evitar acúmulo de atraso.
  - Reamostragem de 16kHz para a taxa do dispositivo de saída escolhido.
  - Suporte a *Deafen* (silenciamento imediato).

---

## 4. Pipeline de Captura e Vídeo ([`src-tauri/src/screen/`](src-tauri/src/screen/))

- **Captura (`capture.rs`)**:
  - Utiliza `windows-capture`, que usa a API oficial **Windows Graphics Capture (DirectX 11 / DXGI)**.
  - Aceleração por GPU, captura do cursor do mouse e enumeração de múltiplos monitores.
  - Compressão com `jpeg-encoder`, largura máxima de 1920 px, qualidade adaptativa e limite de ~4 MB/s.
  - Fragmentação em pacotes UDP de tamanho compatível com o MTU do WireGuard (~1200 bytes).
- **Recepção (`receiver.rs`)**:
  - Remontagem de pacotes UDP indexados por `frame_id`.
  - O último JPEG completo é servido por MJPEG em `127.0.0.1:9768`.
  - Um frame com chunk perdido é descartado; não há retransmissão.

---

## 5. Interface Gráfica ([`src/`](src/))

- Desenvolvida em React 19, Tailwind CSS v4 e Zustand, dentro do Tauri 2.
- Layout de 3 colunas com dock inferior flutuante idêntico à identidade visual desejada.
