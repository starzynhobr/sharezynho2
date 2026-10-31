import { useEffect, useState, type FormEvent } from 'react';
import { X, Copy, Check, Monitor, ArrowRight, Play } from 'lucide-react';
import { api } from '../api';
import { Select } from './Select';
import { buildInvite, parseInvite } from '../invite';
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
  const [invite, setInvite] = useState('');
  const [inviteError, setInviteError] = useState(false);
  const [devices, setDevices] = useState<AudioDevices>({ inputs: [], outputs: [] });
  const [inputDevice, setInputDevice] = useState('');
  const [outputDevice, setOutputDevice] = useState('');

  useEffect(() => {
    if (!isOpen) return;
    setIsCreate(initialModeIsCreate);
    setInviteError(false);
    api.getAudioDevices().then(setDevices).catch((err: unknown) => {
      console.error('Falha ao listar dispositivos de áudio:', err);
    });
  }, [isOpen, initialModeIsCreate]);

  if (!isOpen) return null;

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    const parsed = isCreate ? { address: '', room: room.trim().toUpperCase() } : parseInvite(invite);
    if (!parsed) {
      setInviteError(true);
      return;
    }
    if (!name.trim() || !parsed.room) return;
    const submit = () => onSubmit(isCreate, name.trim(), parsed.room, parsed.address);
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
              isCreate ? 'bg-brand text-root shadow-xs' : 'text-slate-400 hover:text-white'
            }`}
          >
            Criar Sala (Host)
          </button>
          <button
            type="button"
            onClick={() => setIsCreate(false)}
            className={`py-1.5 rounded-lg transition-all cursor-pointer ${
              !isCreate ? 'bg-brand text-root shadow-xs' : 'text-slate-400 hover:text-white'
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

          {isCreate ? (
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
          ) : (
            <div>
              <label className="block text-xs font-semibold text-slate-300 mb-1.5">
                Convite
              </label>
              <input
                type="text"
                required
                autoFocus
                value={invite}
                onChange={(e) => {
                  setInvite(e.target.value);
                  setInviteError(false);
                }}
                placeholder="Cole aqui o convite recebido"
                aria-invalid={inviteError}
                className="w-full bg-root border border-border-subtle focus:border-brand rounded-xl px-3.5 py-2 text-sm text-white font-mono placeholder-slate-500 outline-hidden transition-colors"
              />
              {inviteError && (
                <p className="mt-1.5 text-xs text-danger">
                  Convite inválido. Copie novamente o convite completo.
                </p>
              )}
            </div>
          )}

          {isCreate && (
            <div className="p-3 rounded-xl bg-root/60 border border-border-subtle text-xs text-slate-400 flex flex-col gap-1">
              <span className="font-semibold text-slate-300">Endereço Local:</span>
              <span className="font-mono text-brand font-medium">
                {detectedIp}:{defaultPort}
              </span>
            </div>
          )}

          <div className="grid grid-cols-2 gap-3">
            <Select
              label="Microfone"
              value={inputDevice}
              defaultLabel="Padrão do sistema"
              options={devices.inputs}
              onChange={setInputDevice}
            />
            <Select
              label="Saída de áudio"
              value={outputDevice}
              defaultLabel="Padrão do sistema"
              options={devices.outputs}
              onChange={setOutputDevice}
            />
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
              className="px-5 py-2 rounded-xl bg-brand hover:bg-brand-hover text-root text-xs font-bold flex items-center gap-1.5 transition-all cursor-pointer"
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

  const inviteText = buildInvite(hostAddress, room);

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
          Envie este convite. A pessoa só precisa colar em “Entrar com um convite”.
        </p>

        <div className="bg-root border border-border-subtle rounded-xl p-3 font-mono text-xs text-slate-200 break-all select-text">
          {inviteText}
        </div>

        {hostAddress.startsWith('100.') ? null : (
          <p className="text-xs text-brand leading-relaxed">
            Tailscale desconectado: este convite só funciona na sua rede local.
          </p>
        )}

        <button
          onClick={handleCopy}
          className="w-full py-2.5 px-4 rounded-xl bg-brand hover:bg-brand-hover text-root text-xs font-bold flex items-center justify-center gap-2 transition-all cursor-pointer"
        >
          {copied ? <Check className="w-4 h-4 text-root" /> : <Copy className="w-4 h-4" />}
          {copied ? 'Convite copiado!' : 'Copiar convite'}
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
          Qual monitor você quer transmitir?
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
            className="px-4 py-2 rounded-xl bg-brand hover:bg-brand-hover text-root text-xs font-bold transition-all cursor-pointer"
          >
            Iniciar Transmissão
          </button>
        </div>
      </div>
    </div>
  );
};
