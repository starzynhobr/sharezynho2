import {
  Monitor,
  Mic,
  MicOff,
  Headphones,
  VolumeX,
  MonitorUp,
  PhoneOff,
  Share2,
  Zap,
  Shield,
  Cpu,
  Radio,
  Link2,
  Play,
  ArrowRight,
} from 'lucide-react';
import { useRoomStore } from '../store/roomStore';

export const MainCanvas: React.FC = () => {
  const currentRoom = useRoomStore((s) => s.currentRoom);
  const participants = useRoomStore((s) => s.participants);
  const screenPresenter = useRoomStore((s) => s.screenPresenter);
  const networkInfo = useRoomStore((s) => s.networkInfo);
  const statusText = useRoomStore((s) => s.statusText);
  const isMicOn = useRoomStore((s) => s.isMicOn);
  const isDeafened = useRoomStore((s) => s.isDeafened);
  const isSharingScreen = useRoomStore((s) => s.isSharingScreen);

  const onOpenJoinCreate = useRoomStore((s) => s.openJoinCreate);
  const onOpenInvite = useRoomStore((s) => s.openInvite);
  const onStartShareClick = useRoomStore((s) => s.startShareClick);
  const onStopShareClick = useRoomStore((s) => s.stopShareClick);
  const onToggleMic = useRoomStore((s) => s.toggleMic);
  const onToggleDeafen = useRoomStore((s) => s.toggleDeafen);
  const onLeaveRoom = useRoomStore((s) => s.leaveRoom);
  return (
    <main className="flex-1 h-full bg-main border border-border-subtle rounded-2xl flex flex-col p-6 overflow-hidden select-none shadow-xl justify-between">
      {currentRoom ? (
        /* --- MODE 2: ACTIVE ROOM VIEW --- */
        <div className="flex flex-col h-full justify-between gap-4">
          {/* Header Row */}
          <div className="flex items-center justify-between border-b border-border-subtle/50 pb-4">
            <div>
              <div className="flex items-center gap-2.5">
                <span className="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse" />
                <h1 className="text-xl font-extrabold text-white tracking-wide">
                  #{currentRoom.toUpperCase()}
                </h1>
              </div>
              <p className="text-xs text-slate-400 mt-0.5">
                {participants.length} participante{participants.length === 1 ? '' : 's'} conectados
              </p>
            </div>

            <div className="flex items-center gap-3">
              {/* Participant Avatars */}
              <div className="flex items-center -space-x-2 overflow-hidden mr-1">
                {participants.slice(0, 5).map((p, idx) => (
                  <div
                    key={p.id || idx}
                    title={p.name}
                    className={`inline-block h-8 w-8 rounded-full ring-2 ring-main bg-linear-to-tr from-blue-600 to-indigo-600 flex items-center justify-center text-xs font-bold text-white shadow-xs ${
                      p.is_speaking ? 'ring-emerald-400 shadow-glow-success' : ''
                    }`}
                  >
                    {(p.name[0] || '?').toUpperCase()}
                  </div>
                ))}
              </div>

              <button
                onClick={onOpenInvite}
                className="py-1.5 px-3 rounded-xl bg-control hover:bg-control-hover border border-border-subtle text-xs font-semibold text-white flex items-center gap-1.5 transition-all cursor-pointer shadow-xs"
              >
                <Share2 className="w-3.5 h-3.5 text-brand" />
                Convidar
              </button>
            </div>
          </div>

          {/* Central Stage */}
          <div className="flex-1 bg-root/70 border border-border-subtle rounded-2xl p-4 flex items-center justify-center relative overflow-hidden min-h-[300px]">
            {screenPresenter ? (
              /* Live Hardware Accelerated Video Stream via MJPEG */
              <div className="w-full h-full flex flex-col items-center justify-center relative">
                <img
                  src={`http://127.0.0.1:${networkInfo?.stream_port || 9768}/video`}
                  alt="Transmissão ao vivo"
                  className="max-h-full max-w-full rounded-xl object-contain shadow-2xl border border-border-subtle/80 bg-black"
                />
                <div className="absolute top-3 left-3 bg-root/90 backdrop-blur-md border border-border-subtle px-3 py-1.5 rounded-lg text-xs font-semibold text-brand flex items-center gap-2 shadow-lg">
                  <span className="w-2 h-2 rounded-full bg-brand animate-ping" />
                  Transmissão ativa: {screenPresenter}
                </div>
              </div>
            ) : (
              /* Stage Empty Hero */
              <div className="flex flex-col items-center text-center max-w-md p-6">
                <div className="w-16 h-16 rounded-2xl bg-card border border-border-subtle flex items-center justify-center text-slate-400 mb-4 shadow-md">
                  <Monitor className="w-8 h-8 text-brand" />
                </div>
                <h3 className="text-lg font-bold text-white">Compartilhe sua tela</h3>
                <p className="text-xs text-slate-400 mt-1.5 leading-relaxed">
                  Mostre seu jogo, aplicativo ou apresentação para a sala em tempo real com DirectX 60 FPS.
                </p>
                <button
                  onClick={onStartShareClick}
                  className="mt-5 py-2.5 px-5 rounded-xl bg-brand hover:bg-brand-hover text-white text-xs font-bold flex items-center gap-2 shadow-lg shadow-brand/25 transition-all hover:scale-105 active:scale-95 cursor-pointer"
                >
                  <MonitorUp className="w-4 h-4" />
                  Compartilhar tela
                </button>
              </div>
            )}
          </div>

          {/* Floating Call Dock */}
          <div className="flex flex-col items-center gap-2">
            <div className="bg-card/90 backdrop-blur-md border border-border-subtle rounded-full px-6 py-2.5 flex items-center gap-3 shadow-2xl">
              {/* Mic Toggle */}
              <button
                onClick={onToggleMic}
                title={isMicOn ? 'Desativar Microfone' : 'Ativar Microfone'}
                className={`w-11 h-11 rounded-full flex items-center justify-center transition-all cursor-pointer shadow-md ${
                  isMicOn
                    ? 'bg-emerald-500 hover:bg-emerald-400 text-white shadow-glow-success'
                    : 'bg-control hover:bg-control-hover text-rose-400'
                }`}
              >
                {isMicOn ? <Mic className="w-5 h-5" /> : <MicOff className="w-5 h-5" />}
              </button>

              {/* Deafen Toggle */}
              <button
                onClick={onToggleDeafen}
                title={isDeafened ? 'Ouvir Áudio' : 'Silenciar Áudio'}
                className={`w-11 h-11 rounded-full flex items-center justify-center transition-all cursor-pointer shadow-md ${
                  isDeafened
                    ? 'bg-rose-500 hover:bg-rose-400 text-white shadow-glow-danger'
                    : 'bg-control hover:bg-control-hover text-slate-300'
                }`}
              >
                {isDeafened ? <VolumeX className="w-5 h-5" /> : <Headphones className="w-5 h-5" />}
              </button>

              <div className="w-px h-6 bg-border-subtle mx-1" />

              {/* Screen Share Toggle */}
              <button
                onClick={isSharingScreen ? onStopShareClick : onStartShareClick}
                title={isSharingScreen ? 'Parar Compartilhamento' : 'Compartilhar Tela'}
                className={`w-11 h-11 rounded-full flex items-center justify-center transition-all cursor-pointer shadow-md ${
                  isSharingScreen
                    ? 'bg-brand hover:bg-brand-hover text-white shadow-glow-brand'
                    : 'bg-control hover:bg-control-hover text-slate-300'
                }`}
              >
                <MonitorUp className="w-5 h-5" />
              </button>

              {/* Leave Call Button */}
              <button
                onClick={onLeaveRoom}
                title="Sair da Sala"
                className="w-11 h-11 rounded-full bg-rose-500 hover:bg-rose-600 text-white flex items-center justify-center transition-all cursor-pointer shadow-lg shadow-rose-500/30 hover:scale-105 active:scale-95"
              >
                <PhoneOff className="w-5 h-5" />
              </button>
            </div>

            {/* Bottom Status text */}
            <span className="text-[11px] text-slate-400 font-medium">
              {statusText}
            </span>
          </div>
        </div>
      ) : (
        /* --- MODE 1: WELCOME / HOME DASHBOARD --- */
        <div className="flex flex-col h-full justify-between gap-6">
          {/* Header Row */}
          <div className="flex items-center justify-between border-b border-border-subtle/50 pb-5">
            <div>
              <h1 className="text-2xl font-black text-white tracking-tight flex items-center gap-2">
                Sharezynho
              </h1>
              <p className="text-xs text-slate-400 mt-1">
                Compartilhamento de tela e áudio peer-to-peer de alto desempenho via Tailscale Mesh
              </p>
            </div>

            <div className="bg-card border border-border-subtle px-3.5 py-1.5 rounded-xl flex items-center gap-2 shadow-xs">
              <span
                className={`w-2 h-2 rounded-full ${
                  networkInfo?.is_tailscale ? 'bg-emerald-400' : 'bg-slate-400'
                }`}
              />
              <span className="text-xs font-mono font-bold text-slate-200">
                {networkInfo?.is_tailscale ? 'Tailscale: ' : 'LAN: '}
                {networkInfo?.preferred_ip || '127.0.0.1'}
              </span>
            </div>
          </div>

          {/* Action Cards (Responsive Grid) */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-5 flex-1 items-center">
            {/* Card 1: Criar Sala (Host) */}
            <div
              onClick={() => onOpenJoinCreate(true)}
              className="bg-card border border-border-subtle hover:border-brand/60 rounded-2xl p-6 flex flex-col justify-between h-[250px] shadow-lg transition-all duration-200 hover:-translate-y-1 hover:shadow-glow-brand cursor-pointer group"
            >
              <div className="flex flex-col gap-3">
                <div className="flex items-center justify-between">
                  <div className="w-12 h-12 rounded-xl bg-brand/10 border border-brand/20 flex items-center justify-center text-brand group-hover:scale-110 transition-transform">
                    <Radio className="w-6 h-6 text-brand" />
                  </div>
                  <span className="text-[10px] font-extrabold uppercase tracking-wider bg-brand/10 text-brand px-2.5 py-1 rounded-md border border-brand/20">
                    Host Local
                  </span>
                </div>

                <div>
                  <h3 className="text-lg font-bold text-white group-hover:text-brand transition-colors">
                    Criar Nova Sala
                  </h3>
                  <p className="text-xs text-slate-400 mt-1.5 leading-relaxed">
                    Inicie uma sala na sua máquina como host. Amigos na sua rede Tailscale poderão entrar para conversar e assistir sua transmissão.
                  </p>
                </div>
              </div>

              <button className="py-2.5 px-4 rounded-xl bg-brand group-hover:bg-brand-hover text-white text-xs font-bold flex items-center justify-center gap-2 shadow-md shadow-brand/20 transition-all">
                <Play className="w-3.5 h-3.5 fill-current" />
                Iniciar como Host
              </button>
            </div>

            {/* Card 2: Entrar em Sala (Cliente) */}
            <div
              onClick={() => onOpenJoinCreate(false)}
              className="bg-card border border-border-subtle hover:border-emerald-500/60 rounded-2xl p-6 flex flex-col justify-between h-[250px] shadow-lg transition-all duration-200 hover:-translate-y-1 hover:shadow-glow-success cursor-pointer group"
            >
              <div className="flex flex-col gap-3">
                <div className="flex items-center justify-between">
                  <div className="w-12 h-12 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400 group-hover:scale-110 transition-transform">
                    <Link2 className="w-6 h-6 text-emerald-400" />
                  </div>
                  <span className="text-[10px] font-extrabold uppercase tracking-wider bg-emerald-500/10 text-emerald-400 px-2.5 py-1 rounded-md border border-emerald-500/20">
                    Cliente P2P
                  </span>
                </div>

                <div>
                  <h3 className="text-lg font-bold text-white group-hover:text-emerald-400 transition-colors">
                    Entrar em uma Sala
                  </h3>
                  <p className="text-xs text-slate-400 mt-1.5 leading-relaxed">
                    Conecte-se diretamente a uma transmissão iniciada por um amigo. Basta inserir o IP do Tailscale dele e o código da sala.
                  </p>
                </div>
              </div>

              <button className="py-2.5 px-4 rounded-xl bg-control group-hover:bg-control-hover border border-border-subtle text-slate-200 group-hover:text-white text-xs font-bold flex items-center justify-center gap-2 transition-all">
                Conectar via IP
                <ArrowRight className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>

          {/* Spec Badges Row */}
          <div className="flex flex-wrap items-center justify-center gap-3 pt-2">
            <div className="bg-card/70 border border-border-subtle px-3.5 py-2 rounded-xl flex items-center gap-2 text-xs font-medium text-slate-300">
              <Zap className="w-4 h-4 text-amber-400" />
              Ultraleve (~30MB RAM)
            </div>
            <div className="bg-card/70 border border-border-subtle px-3.5 py-2 rounded-xl flex items-center gap-2 text-xs font-medium text-slate-300">
              <Shield className="w-4 h-4 text-emerald-400" />
              P2P WireGuard Mesh
            </div>
            <div className="bg-card/70 border border-border-subtle px-3.5 py-2 rounded-xl flex items-center gap-2 text-xs font-medium text-slate-300">
              <Cpu className="w-4 h-4 text-brand" />
              DirectX GPU 60 FPS
            </div>
          </div>

          {/* Bottom Bar */}
          <div className="text-center text-[11px] text-slate-400 border-t border-border-subtle/50 pt-3">
            {statusText} • Portas TCP {networkInfo?.tcp_port || 9765} / UDP {networkInfo?.udp_port || 9766}
          </div>
        </div>
      )}
    </main>
  );
};
