import { useState } from 'react';
import { Users, ShieldCheck, Globe, Check } from 'lucide-react';
import { useRoomStore } from '../store/roomStore';

export const NavigationRail: React.FC = () => {
  const networkInfo = useRoomStore((s) => s.networkInfo);
  const userName = useRoomStore((s) => s.userName);
  const openJoinCreate = useRoomStore((s) => s.openJoinCreate);
  const [copied, setCopied] = useState(false);

  const handleCopyIp = () => {
    if (networkInfo?.preferred_ip) {
      navigator.clipboard.writeText(networkInfo.preferred_ip);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const initial = (userName[0] || 'V').toUpperCase();

  return (
    <aside className="w-[70px] h-full bg-panel border border-border-subtle rounded-2xl flex flex-col items-center py-4 px-2 select-none justify-between shrink-0 shadow-lg">
      {/* Top Icons */}
      <div className="flex flex-col items-center gap-4 w-full">
        {/* App Logo */}
        <div
          title="Sharezynho P2P Mesh"
          className="w-11 h-11 rounded-xl bg-linear-to-tr from-brand/20 to-blue-500/10 border border-brand/30 flex items-center justify-center text-brand font-bold text-xl shadow-glow-brand transition-transform hover:scale-105 cursor-pointer"
        >
          〰
        </div>

        <div className="w-8 h-px bg-border-subtle my-1" />

        {/* Salas Button */}
        <button
          onClick={() => openJoinCreate(true)}
          title="Salas & Canais"
          className="w-12 h-12 rounded-xl bg-card border border-brand/50 text-white flex flex-col items-center justify-center gap-0.5 shadow-xs transition-all hover:bg-card-hover hover:border-brand cursor-pointer"
        >
          <Users className="w-4 h-4 text-brand" />
          <span className="text-[10px] font-medium text-slate-300">Salas</span>
        </button>

        {/* VPN / Tailscale Pill Button */}
        <button
          onClick={handleCopyIp}
          title={
            networkInfo?.is_tailscale
              ? `Tailscale Conectado: ${networkInfo.preferred_ip} (Clique para copiar)`
              : `Rede Local: ${networkInfo?.preferred_ip || '127.0.0.1'} (Clique para copiar)`
          }
          className={`w-12 h-12 rounded-xl border flex flex-col items-center justify-center gap-0.5 transition-all cursor-pointer ${
            networkInfo?.is_tailscale
              ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400 hover:bg-emerald-500/20'
              : 'bg-card border-border-subtle text-slate-400 hover:bg-card-hover'
          }`}
        >
          {copied ? (
            <Check className="w-4 h-4 text-emerald-400 animate-bounce" />
          ) : networkInfo?.is_tailscale ? (
            <ShieldCheck className="w-4 h-4 text-emerald-400" />
          ) : (
            <Globe className="w-4 h-4 text-slate-400" />
          )}
          <span className="text-[10px] font-semibold">
            {copied ? 'Copiado!' : networkInfo?.is_tailscale ? 'VPN' : 'LAN'}
          </span>
        </button>
      </div>

      {/* Bottom User Profile */}
      <div className="flex flex-col items-center gap-1.5 cursor-pointer group" title={`Perfil: ${userName}`}>
        <div className="w-10 h-10 rounded-full bg-linear-to-tr from-blue-600 to-indigo-500 flex items-center justify-center font-bold text-white text-sm shadow-md ring-2 ring-border-subtle group-hover:ring-brand transition-all">
          {initial}
        </div>
        <span className="text-[11px] font-medium text-slate-400 group-hover:text-slate-200 transition-colors">
          Você
        </span>
      </div>
    </aside>
  );
};
