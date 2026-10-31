import { MonitorUp } from 'lucide-react';
import { useRoomStore } from '../store/roomStore';

export const RoomsSidebar: React.FC = () => {
  const participants = useRoomStore((s) => s.participants);

  return (
    <aside className="w-60 h-full shrink-0 border-r border-border-subtle bg-panel flex flex-col p-4 gap-4">
      <span className="font-display text-base font-bold text-white">Sharezynho</span>

      <div className="min-h-0 flex-1 flex flex-col gap-2">
        <h2 className="text-xs font-semibold text-slate-400">
          Na sala · {participants.length}
        </h2>
        <ul className="flex flex-col gap-1 overflow-y-auto">
          {participants.map((p) => (
            <li key={p.id} className="flex items-center gap-2.5 rounded-lg p-1.5">
              <span
                className={`w-8 h-8 shrink-0 rounded-full bg-control flex items-center justify-center text-xs font-bold text-white ${
                  p.is_speaking ? 'ring-2 ring-speaking ring-offset-2 ring-offset-panel' : ''
                }`}
              >
                {(p.name[0] || '?').toUpperCase()}
              </span>
              <span className="min-w-0 flex-1">
                <span className="block text-sm font-medium text-slate-100 truncate">{p.name}</span>
                <span className="block text-[11px] text-slate-400">
                  {p.is_speaking ? 'Falando' : p.is_host ? 'Host' : 'Conectado'}
                </span>
              </span>
              {p.is_sharing_screen && (
                <MonitorUp className="w-4 h-4 shrink-0 text-brand" aria-label="Compartilhando tela" />
              )}
            </li>
          ))}
        </ul>
      </div>
    </aside>
  );
};
