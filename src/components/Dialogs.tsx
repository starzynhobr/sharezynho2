import { useEffect, useState, type FormEvent } from 'react';
import { X, Copy, Check, Monitor, ArrowRight, Play } from 'lucide-react';
import { api } from '../api';
import type { AudioDevices } from '../types';

interface JoinCreateModalProps {
  isOpen: boolean;
  initialModeIsCreate: boolean;
  detectedIp: string;
  defaultPort: number;
  onClose: () => void;
  onSubmit: (isCreate: boolean, name: string, room: string, address: string) => void;
}

export const JoinCreateModal: React.FC<JoinCreateModalProps> = ({
  isOpen,
  initialModeIsCreate,
  detectedIp,
  defaultPort,
  onClose,
  onSubmit,
}) => {
  const [isCreate, setIsCreate] = useState(initialModeIsCreate);
  const [name, setName] = useState('');
  const [room, setRoom] = useState(
    Math.floor(0x100000 + Math.random() * 0xefffff).toString(16).toUpperCase()
  );
  const [address, setAddress] = useState(`${detectedIp}:${defaultPort}`);
  const [devices, setDevices] = useState<AudioDevices>({ inputs: [], outputs: [] });
  const [inputDevice, setInputDevice] = useState('');
  const [outputDevice, setOutputDevice] = useState('');

  useEffect(() => {
    if (!isOpen) return;
    api.getAudioDevices().then(setDevices).catch((err: unknown) => {
      console.error('Falha ao listar dispositivos de áudio:', err);
    });
  }, [isOpen]);

  if (!isOpen) return null;

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !room.trim()) return;
    const submit = () => onSubmit(isCreate, name.trim(), room.trim().toUpperCase(), address.trim());
    api.setAudioDevices(inputDevice || null, outputDevice || null).finally(submit);
  };

  return (
    <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 select-none animate-in fade-in duration-200">
      <div className="bg-card border border-border-subtle rounded-2xl w-full max-w-md p-6 shadow-2xl flex flex-col gap-5">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-border-subtle/60 pb-3">
          <h2 className="text-base font-bold text-white">
            {isCreate ? 'Criar Nova Sala (Host)' : 'Entrar em uma Sala'}
          </h2>
          <button
            onClick={onClose}
            className="w-7 h-7 rounded-lg text-slate-400 hover:text-white hover:bg-control flex items-center justify-center transition-colors cursor-pointer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Tab switcher */}
        <div className="grid grid-cols-2 p-1 bg-root border border-border-subtle rounded-xl text-xs font-semibold">
          <button
            type="button"
            onClick={() => setIsCreate(true)}
            className={`py-1.5 rounded-lg transition-all cursor-pointer ${
              isCreate ? 'bg-brand text-white shadow-xs' : 'text-slate-400 hover:text-white'
            }`}
          >
            Criar Sala (Host)
          </button>
          <button
            type="button"
            onClick={() => setIsCreate(false)}
            className={`py-1.5 rounded-lg transition-all cursor-pointer ${
              !isCreate ? 'bg-brand text-white shadow-xs' : 'text-slate-400 hover:text-white'
            }`}
          >
            Entrar em Sala
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div>
            <label className="block text-xs font-semibold text-slate-300 mb-1.5">
              Seu Nome de Exibição
            </label>
            <input
              type="text"
              required
              placeholder="Ex: Pedro, Lucas..."
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full bg-root border border-border-subtle focus:border-brand rounded-xl px-3.5 py-2 text-sm text-white placeholder-slate-500 outline-hidden transition-colors"
            />
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-300 mb-1.5">
              Código da Sala
            </label>
            <input
              type="text"
              required
              value={room}
              onChange={(e) => setRoom(e.target.value.toUpperCase())}
              className="w-full bg-root border border-border-subtle focus:border-brand rounded-xl px-3.5 py-2 text-sm text-white font-mono tracking-wider outline-hidden transition-colors"
            />
          </div>

          {!isCreate ? (
            <div>
              <label className="block text-xs font-semibold text-slate-300 mb-1.5">
                Endereço do Host (IP ou nome Tailscale:{defaultPort})
              </label>
              <input
                type="text"
                required
                value={address}
                onChange={(e) => setAddress(e.target.value)}
                placeholder={`100.x.y.z:${defaultPort}`}
                className="w-full bg-root border border-border-subtle focus:border-brand rounded-xl px-3.5 py-2 text-sm text-white font-mono outline-hidden transition-colors"
              />
            </div>
          ) : (
            <div className="p-3 rounded-xl bg-root/60 border border-border-subtle text-xs text-slate-400 flex flex-col gap-1">
              <span className="font-semibold text-slate-300">Endereço Local:</span>
              <span className="font-mono text-brand font-medium">
                {detectedIp}:{defaultPort}
              </span>
            </div>
          )}

          <div className="grid grid-cols-2 gap-3">
            <div className="min-w-0">
              <label className="block text-xs font-semibold text-slate-300 mb-1.5">Microfone</label>
              <select
                value={inputDevice}
                onChange={(e) => setInputDevice(e.target.value)}
                className="w-full bg-root border border-border-subtle focus:border-brand rounded-xl px-2.5 py-2 text-xs text-white outline-hidden transition-colors"
              >
                <option value="">Padrão do sistema</option>
                {devices.inputs.map((device) => (
                  <option key={device} value={device}>{device}</option>
                ))}
              </select>
            </div>
            <div className="min-w-0">
              <label className="block text-xs font-semibold text-slate-300 mb-1.5">Saída de áudio</label>
              <select
                value={outputDevice}
                onChange={(e) => setOutputDevice(e.target.value)}
                className="w-full bg-root border border-border-subtle focus:border-brand rounded-xl px-2.5 py-2 text-xs text-white outline-hidden transition-colors"
              >
                <option value="">Padrão do sistema</option>
                {devices.outputs.map((device) => (
                  <option key={device} value={device}>{device}</option>
                ))}
              </select>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex items-center justify-end gap-2.5 pt-2 border-t border-border-subtle/60">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-300 hover:text-white hover:bg-control transition-colors cursor-pointer"
            >
              Cancelar
            </button>
            <button
              type="submit"
              className="px-5 py-2 rounded-xl bg-brand hover:bg-brand-hover text-white text-xs font-bold flex items-center gap-1.5 shadow-md shadow-brand/20 transition-all cursor-pointer"
            >
              {isCreate ? <Play className="w-3.5 h-3.5 fill-current" /> : <ArrowRight className="w-3.5 h-3.5" />}
              {isCreate ? 'Iniciar Sala' : 'Conectar'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

interface InviteModalProps {
  isOpen: boolean;
  hostAddress: string;
  room: string;
  onClose: () => void;
}

export const InviteModal: React.FC<InviteModalProps> = ({
  isOpen,
  hostAddress,
  room,
  onClose,
}) => {
  const [copied, setCopied] = useState(false);

  if (!isOpen) return null;

  const inviteText = `Sharezynho Convite:\nEndereço: ${hostAddress}\nSala: #${room}`;

  const handleCopy = () => {
    navigator.clipboard.writeText(inviteText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2500);
  };

  return (
    <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 select-none animate-in fade-in duration-200">
      <div className="bg-card border border-border-subtle rounded-2xl w-full max-w-md p-6 shadow-2xl flex flex-col gap-4">
        <div className="flex items-center justify-between border-b border-border-subtle/60 pb-3">
          <h2 className="text-base font-bold text-white">Convidar Amigos</h2>
          <button
            onClick={onClose}
            className="w-7 h-7 rounded-lg text-slate-400 hover:text-white hover:bg-control flex items-center justify-center transition-colors cursor-pointer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <p className="text-xs text-slate-400 leading-relaxed">
          Compartilhe os dados abaixo com quem você deseja convidar pela sua rede Tailscale:
        </p>

        <div className="bg-root border border-border-subtle rounded-xl p-3 flex flex-col gap-2 font-mono text-xs">
          <div>
            <span className="text-slate-500 block text-[10px] font-sans font-semibold">Endereço:</span>
            <span className="text-slate-200 select-text">{hostAddress}</span>
          </div>
          <div>
            <span className="text-slate-500 block text-[10px] font-sans font-semibold">Código da Sala:</span>
            <span className="text-brand font-bold text-sm select-text">#{room}</span>
          </div>
        </div>

        <button
          onClick={handleCopy}
          className="w-full py-2.5 px-4 rounded-xl bg-brand hover:bg-brand-hover text-white text-xs font-bold flex items-center justify-center gap-2 shadow-md shadow-brand/20 transition-all cursor-pointer"
        >
          {copied ? <Check className="w-4 h-4 text-emerald-300" /> : <Copy className="w-4 h-4" />}
          {copied ? 'Copiado para a área de transferência!' : 'Copiar dados do convite'}
        </button>
      </div>
    </div>
  );
};

interface MonitorPickerModalProps {
  isOpen: boolean;
  monitors: string[];
  onClose: () => void;
  onSelectMonitor: (index: number) => void;
}

export const MonitorPickerModal: React.FC<MonitorPickerModalProps> = ({
  isOpen,
  monitors,
  onClose,
  onSelectMonitor,
}) => {
  const [selectedIndex, setSelectedIndex] = useState(0);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 select-none animate-in fade-in duration-200">
      <div className="bg-card border border-border-subtle rounded-2xl w-full max-w-sm p-6 shadow-2xl flex flex-col gap-4">
        <div className="flex items-center justify-between border-b border-border-subtle/60 pb-3">
          <h2 className="text-base font-bold text-white">Escolher Tela</h2>
          <button
            onClick={onClose}
            className="w-7 h-7 rounded-lg text-slate-400 hover:text-white hover:bg-control flex items-center justify-center transition-colors cursor-pointer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <p className="text-xs text-slate-400">
          Qual monitor você gostaria de transmitir com GPU DirectX?
        </p>

        <div className="flex flex-col gap-2 max-h-[220px] overflow-y-auto">
          {monitors.map((mon, idx) => (
            <button
              key={idx}
              type="button"
              onClick={() => setSelectedIndex(idx)}
              className={`w-full p-3 rounded-xl border flex items-center gap-3 text-left transition-all cursor-pointer ${
                selectedIndex === idx
                  ? 'bg-brand/10 border-brand text-white shadow-xs'
                  : 'bg-root border-border-subtle text-slate-400 hover:text-white hover:border-slate-600'
              }`}
            >
              <Monitor className={`w-5 h-5 ${selectedIndex === idx ? 'text-brand' : 'text-slate-500'}`} />
              <div className="text-xs font-semibold truncate">
                Monitor {idx + 1}: {mon}
              </div>
            </button>
          ))}
        </div>

        <div className="flex items-center justify-end gap-2 pt-2 border-t border-border-subtle/60">
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-300 hover:text-white hover:bg-control transition-colors cursor-pointer"
          >
            Cancelar
          </button>
          <button
            onClick={() => onSelectMonitor(selectedIndex)}
            className="px-4 py-2 rounded-xl bg-brand hover:bg-brand-hover text-white text-xs font-bold transition-all cursor-pointer"
          >
            Iniciar Transmissão
          </button>
        </div>
      </div>
    </div>
  );
};
