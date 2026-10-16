import { useState } from 'react';
import { Plus, Radio, Share2, LogOut, Copy, Check, Mic, Volume2 } from 'lucide-react';
import { useRoomStore } from '../store/roomStore';

const AVATAR_COLORS = [
  'from-blue-600 to-indigo-500',
  'from-purple-600 to-pink-500',
  'from-emerald-600 to-teal-500',
  'from-amber-500 to-orange-500',
  'from-rose-500 to-red-600',
];

export const RoomsSidebar: React.FC = () => {
  const currentRoom = useRoomStore((s) => s.currentRoom);
  const participants = useRoomStore((s) => s.participants);
  const networkInfo = useRoomStore((s) => s.networkInfo);
  const openJoinCreate = useRoomStore((s) => s.openJoinCreate);
  const openInvite = useRoomStore((s) => s.openInvite);
  const leaveRoom = useRoomStore((s) => s.leaveRoom);
  const [copied, setCopied] = useState(false);

  const handleCopyIp = () => {
    if (networkInfo?.preferred_ip) {
      navigator.clipboard.writeText(networkInfo.preferred_ip);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <aside className="w-[280px] h-full bg-panel border border-border-subtle rounded-2xl flex flex-col p-4 select-none shrink-0 shadow-lg justify-between">
      {/* Top Section */}
      <div className="flex flex-col gap-4 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-bold tracking-tight text-white flex items-center gap-2">
            Salas
          </h2>
          <button
            onClick={() => openJoinCreate(true)}
            title="Criar ou Entrar em Sala"
            className="w-8 h-8 rounded-lg bg-control border border-border-subtle flex items-center justify-center text-slate-200 hover:text-white hover:bg-control-hover hover:border-brand/40 transition-all cursor-pointer shadow-xs"
          >
            <Plus className="w-4 h-4" />
          </button>
        </div>

        {/* Room State */}
        {currentRoom ? (
          <div className="flex flex-col gap-4">
            {/* Active Room Card */}
            <div className="bg-card border border-brand/40 rounded-xl p-3.5 shadow-md flex flex-col gap-2.5">
              <div className="flex items-center justify-between">
                <span className="flex items-center gap-1.5 text-[11px] font-bold text-emerald-400 uppercase tracking-wider">
                  <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
                  Sala Ativa
                </span>
                <span className="text-[11px] text-slate-400 font-medium">
                  {participants.length} participante{participants.length === 1 ? '' : 's'}
                </span>
              </div>

              <div className="text-base font-extrabold text-white tracking-wide truncate">
                #{currentRoom.toUpperCase()}
              </div>

              <div className="flex items-center gap-2 pt-1">
                <button
                  onClick={openInvite}
                  className="flex-1 py-1.5 px-2.5 rounded-lg bg-control hover:bg-control-hover border border-border-subtle text-xs font-semibold text-slate-200 hover:text-white flex items-center justify-center gap-1.5 transition-all cursor-pointer"
                >
                  <Share2 className="w-3.5 h-3.5 text-brand" />
                  Convidar
                </button>
                <button
                  onClick={leaveRoom}
                  className="py-1.5 px-2.5 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/30 text-xs font-semibold text-rose-400 hover:text-rose-300 flex items-center justify-center gap-1 transition-all cursor-pointer"
                  title="Sair da Sala"
                >
                  <LogOut className="w-3.5 h-3.5" />
                  Sair
                </button>
              </div>
            </div>

            {/* Participants Section */}
            <div className="flex flex-col gap-2">
              <span className="text-[11px] font-bold uppercase tracking-wider text-slate-400">
                Participantes ({participants.length})
              </span>

              <div className="flex flex-col gap-1.5 max-h-[220px] overflow-y-auto pr-1">
                {participants.map((p, idx) => {
                  const pInitial = (p.name[0] || '?').toUpperCase();
                  const colorClass = AVATAR_COLORS[idx % AVATAR_COLORS.length];

                  return (
                    <div
                      key={p.id || idx}
                      className="flex items-center justify-between p-2 rounded-lg bg-card/60 border border-border-subtle hover:bg-card transition-colors"
                    >
                      <div className="flex items-center gap-2.5 min-w-0">
                        <div
                          className={`w-7 h-7 rounded-full bg-linear-to-tr ${colorClass} flex items-center justify-center text-xs font-bold text-white shrink-0 shadow-xs ${
                            p.is_speaking ? 'ring-2 ring-emerald-400 shadow-glow-success' : ''
                          }`}
                        >
                          {pInitial}
                        </div>
                        <span className="text-xs font-medium text-slate-200 truncate">
                          {p.name}
                        </span>
                      </div>

                      <div className="shrink-0 flex items-center">
                        {p.is_speaking ? (
                          <Volume2 className="w-3.5 h-3.5 text-emerald-400 animate-pulse" />
                        ) : (
                          <Mic className="w-3.5 h-3.5 text-slate-500" />
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        ) : (
          /* Empty Room State */
          <div className="bg-card border border-border-subtle rounded-xl p-4 flex flex-col gap-3 shadow-md">
            <div className="w-10 h-10 rounded-xl bg-brand/10 border border-brand/20 flex items-center justify-center text-brand">
              <Radio className="w-5 h-5 text-brand" />
            </div>

            <div>
              <h3 className="text-sm font-bold text-white">Nenhuma sala ativa</h3>
              <p className="text-xs text-slate-400 mt-1 leading-relaxed">
                Crie uma sala ou conecte-se ao IP de um amigo na sua rede Tailscale.
              </p>
            </div>

            <button
              onClick={() => openJoinCreate(true)}
              className="w-full mt-1 py-2 px-3 rounded-lg bg-brand hover:bg-brand-hover text-white text-xs font-semibold flex items-center justify-center gap-1.5 shadow-md shadow-brand/20 transition-all hover:scale-[1.02] active:scale-[0.98] cursor-pointer"
            >
              <Plus className="w-3.5 h-3.5" />
              Entrar ou criar sala
            </button>
          </div>
        )}
      </div>

      {/* Bottom Pinned Network Connection Box */}
      <div className="bg-card border border-border-subtle rounded-xl p-3 flex flex-col gap-2 shadow-xs">
        <div className="flex items-center justify-between">
          <span className="flex items-center gap-1.5 text-[11px] font-bold text-slate-300">
            <span
              className={`w-2 h-2 rounded-full ${
                networkInfo?.is_tailscale ? 'bg-emerald-400' : 'bg-slate-400'
              }`}
            />
            {networkInfo?.is_tailscale ? 'Tailscale Ativo' : 'Rede Local (LAN)'}
          </span>
        </div>

        <div className="text-xs font-mono font-bold text-slate-100 bg-root/70 px-2 py-1 rounded-md border border-border-subtle truncate select-text">
          {networkInfo?.preferred_ip || '127.0.0.1'}
        </div>

        <button
          onClick={handleCopyIp}
          className="w-full py-1.5 px-2 rounded-lg bg-control hover:bg-control-hover border border-border-subtle text-[11px] font-semibold text-slate-300 hover:text-white flex items-center justify-center gap-1.5 transition-all cursor-pointer"
        >
          {copied ? (
            <>
              <Check className="w-3.5 h-3.5 text-emerald-400" />
              <span className="text-emerald-400">IP Copiado!</span>
            </>
          ) : (
            <>
              <Copy className="w-3.5 h-3.5 text-slate-400" />
              <span>Copiar meu IP</span>
            </>
          )}
        </button>
      </div>
    </aside>
  );
};
