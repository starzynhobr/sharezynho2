import { useState } from 'react';
import {
  ArrowRight,
  Check,
  Copy,
  Headphones,
  Mic,
  MicOff,
  MonitorUp,
  PhoneOff,
  Share2,
  VolumeX,
} from 'lucide-react';
import { useRoomStore } from '../store/roomStore';

const dockButton =
  'h-11 min-w-11 px-3 rounded-xl flex items-center justify-center gap-2 text-xs font-semibold transition-colors cursor-pointer';
const dockIdle = 'bg-control hover:bg-control-hover text-slate-200';

const HomeView: React.FC = () => {
  const networkInfo = useRoomStore((s) => s.networkInfo);
  const statusText = useRoomStore((s) => s.statusText);
  const openJoinCreate = useRoomStore((s) => s.openJoinCreate);
  const [copied, setCopied] = useState(false);

  const ip = networkInfo?.preferred_ip || '127.0.0.1';

  const handleCopyIp = () => {
    navigator.clipboard.writeText(ip);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <main className="flex-1 h-full flex flex-col justify-between p-10 overflow-y-auto">
      <span className="font-display text-lg font-bold text-white">Sharezynho</span>

      <div className="max-w-3xl py-8">
        <h1 className="font-display text-5xl lg:text-6xl font-extrabold leading-[1.02] text-white text-balance">
          Abra a sala. Mostre a tela. Fale com a galera.
        </h1>
        <p className="mt-5 text-base text-slate-400 max-w-xl leading-relaxed">
          Voz e compartilhamento de tela direto entre vocês, pela sua rede Tailscale.
        </p>
        <div className="mt-8 flex flex-wrap items-center gap-3">
          <button
            onClick={() => openJoinCreate(true)}
            className="h-12 px-6 rounded-xl bg-brand hover:bg-brand-hover text-root text-sm font-bold transition-colors cursor-pointer"
          >
            Criar sala
          </button>
          <button
            onClick={() => openJoinCreate(false)}
            className="h-12 px-5 rounded-xl text-sm font-semibold text-slate-200 hover:text-white hover:bg-control flex items-center gap-2 transition-colors cursor-pointer"
          >
            Entrar com um convite
            <ArrowRight className="w-4 h-4" />
          </button>
        </div>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-3 border-t border-border-subtle pt-4 text-xs text-slate-400">
        <button
          onClick={handleCopyIp}
          title="Copiar meu endereço"
          className="flex items-center gap-2 rounded-lg hover:text-white transition-colors cursor-pointer"
        >
          <span
            className={`w-2 h-2 rounded-full ${networkInfo?.is_tailscale ? 'bg-speaking' : 'bg-slate-500'}`}
          />
          {networkInfo?.is_tailscale ? 'Tailscale' : 'Rede local'}
          <span className="font-mono text-slate-200">{ip}</span>
          {copied ? <Check className="w-3.5 h-3.5 text-speaking" /> : <Copy className="w-3.5 h-3.5" />}
        </button>
        <span role="status">{statusText}</span>
      </div>
    </main>
  );
};

const CallView: React.FC = () => {
  const currentRoom = useRoomStore((s) => s.currentRoom);
  const participants = useRoomStore((s) => s.participants);
  const screenPresenter = useRoomStore((s) => s.screenPresenter);
  const networkInfo = useRoomStore((s) => s.networkInfo);
  const statusText = useRoomStore((s) => s.statusText);
  const isMicOn = useRoomStore((s) => s.isMicOn);
  const isDeafened = useRoomStore((s) => s.isDeafened);
  const isSharingScreen = useRoomStore((s) => s.isSharingScreen);
  const openInvite = useRoomStore((s) => s.openInvite);
  const startShareClick = useRoomStore((s) => s.startShareClick);
  const stopShareClick = useRoomStore((s) => s.stopShareClick);
  const toggleMic = useRoomStore((s) => s.toggleMic);
  const toggleDeafen = useRoomStore((s) => s.toggleDeafen);
  const leaveRoom = useRoomStore((s) => s.leaveRoom);

  const presenterName = participants.find((p) => p.id === screenPresenter)?.name;

  return (
    <main className="flex-1 min-w-0 h-full flex flex-col gap-3 p-3">
      <header className="flex items-center justify-between gap-3 px-2 pt-1">
        <div className="min-w-0 flex items-baseline gap-3">
          <h1 className="font-display text-xl font-bold text-white truncate">
            Sala <span className="font-mono text-base text-brand">{currentRoom}</span>
          </h1>
          <span role="status" className="text-xs text-slate-400 truncate">
            {statusText}
          </span>
        </div>
        <button
          onClick={openInvite}
          className="h-9 px-3 shrink-0 rounded-lg bg-control hover:bg-control-hover text-xs font-semibold text-white flex items-center gap-2 transition-colors cursor-pointer"
        >
          <Share2 className="w-3.5 h-3.5" />
          Convidar
        </button>
      </header>

      <section className="relative flex-1 min-h-0 rounded-2xl bg-black/40 border border-border-subtle overflow-hidden flex items-center justify-center">
        {screenPresenter ? (
          <>
            <img
              src={`http://127.0.0.1:${networkInfo?.stream_port || 9768}/video`}
              alt={`Tela de ${presenterName || 'um participante'}`}
              className="w-full h-full object-contain"
            />
            <span className="absolute top-3 left-3 rounded-lg bg-root/85 px-2.5 py-1.5 text-xs font-semibold text-white">
              Tela de {presenterName || 'um participante'}
            </span>
          </>
        ) : (
          <div className="text-center px-6">
            <p className="font-display text-2xl font-bold text-white">Ninguém compartilhando</p>
            <p className="mt-2 text-sm text-slate-400">A tela de quem transmitir aparece aqui.</p>
            <button
              onClick={startShareClick}
              className="mt-5 h-11 px-5 rounded-xl bg-brand hover:bg-brand-hover text-root text-sm font-bold inline-flex items-center gap-2 transition-colors cursor-pointer"
            >
              <MonitorUp className="w-4 h-4" />
              Compartilhar minha tela
            </button>
          </div>
        )}
      </section>

      <div className="flex items-center justify-center gap-2 pb-1">
        <button
          onClick={toggleMic}
          aria-pressed={!isMicOn}
          title={isMicOn ? 'Silenciar microfone' : 'Ativar microfone'}
          className={`${dockButton} ${isMicOn ? dockIdle : 'bg-danger/20 text-danger hover:bg-danger/30'}`}
        >
          {isMicOn ? <Mic className="w-5 h-5" /> : <MicOff className="w-5 h-5" />}
          {!isMicOn && 'Mudo'}
        </button>
        <button
          onClick={toggleDeafen}
          aria-pressed={isDeafened}
          title={isDeafened ? 'Ouvir a sala' : 'Silenciar a sala'}
          className={`${dockButton} ${isDeafened ? 'bg-danger/20 text-danger hover:bg-danger/30' : dockIdle}`}
        >
          {isDeafened ? <VolumeX className="w-5 h-5" /> : <Headphones className="w-5 h-5" />}
          {isDeafened && 'Sem som'}
        </button>
        <button
          onClick={isSharingScreen ? stopShareClick : startShareClick}
          aria-pressed={isSharingScreen}
          title={isSharingScreen ? 'Parar de compartilhar' : 'Compartilhar tela'}
          className={`${dockButton} ${isSharingScreen ? 'bg-brand hover:bg-brand-hover text-root' : dockIdle}`}
        >
          <MonitorUp className="w-5 h-5" />
          {isSharingScreen && 'Parar'}
        </button>
        <div className="w-px h-6 bg-border-subtle mx-1" />
        <button
          onClick={leaveRoom}
          title="Sair da sala"
          className={`${dockButton} bg-danger hover:bg-danger-hover text-white`}
        >
          <PhoneOff className="w-5 h-5" />
          Sair
        </button>
      </div>
    </main>
  );
};

export const MainCanvas: React.FC = () => {
  const currentRoom = useRoomStore((s) => s.currentRoom);
  return currentRoom ? <CallView /> : <HomeView />;
};
