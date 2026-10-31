import { create } from 'zustand';
import { api } from '../api';
import type { NetworkInfo, ParticipantInfo } from '../types';

export interface RoomStoreState {
  // Network
  networkInfo: NetworkInfo | null;
  hostAddress: string;

  // Session
  currentRoom: string | null;
  participants: ParticipantInfo[];
  screenPresenter: string | null;
  userName: string;
  statusText: string;

  // Media
  isMicOn: boolean;
  isDeafened: boolean;
  isSharingScreen: boolean;

  // Modals
  isJoinCreateOpen: boolean;
  joinCreateIsModeCreate: boolean;
  isInviteOpen: boolean;
  isMonitorPickerOpen: boolean;
  monitors: string[];

  // Actions
  init: () => () => void;
  openJoinCreate: (isCreate?: boolean) => void;
  closeJoinCreate: () => void;
  openInvite: () => void;
  closeInvite: () => void;
  closeMonitorPicker: () => void;

  joinCreateSubmit: (
    isCreate: boolean,
    name: string,
    room: string,
    address: string
  ) => Promise<void>;
  leaveRoom: () => Promise<void>;
  toggleMic: () => Promise<void>;
  toggleDeafen: () => Promise<void>;
  startShareClick: () => Promise<void>;
  selectMonitor: (index: number) => Promise<void>;
  stopShareClick: () => Promise<void>;
}

export const useRoomStore = create<RoomStoreState>((set, get) => ({
  networkInfo: null,
  hostAddress: '',
  currentRoom: null,
  participants: [],
  screenPresenter: null,
  userName: 'Você',
  statusText: 'Pronto para conectar',

  isMicOn: true,
  isDeafened: false,
  isSharingScreen: false,

  isJoinCreateOpen: false,
  joinCreateIsModeCreate: true,
  isInviteOpen: false,
  isMonitorPickerOpen: false,
  monitors: [],

  init: () => {
    let isCancelled = false;
    let unlistenRoom: (() => void) | undefined;
    let unlistenError: (() => void) | undefined;
    let unlistenDisc: (() => void) | undefined;
    let unlistenReconnected: (() => void) | undefined;

    // 1. Network info
    api
      .getNetworkInfo()
      .then((info) => {
        if (!isCancelled) {
          set({
            networkInfo: info,
            hostAddress: `${info.preferred_ip}:${info.tcp_port}`,
          });
        }
      })
      .catch((err: unknown) => {
        console.error('Falha ao obter info de rede:', err);
      });

    // 2. Room State listener
    api
      .onRoomState((payload) => {
        if (!isCancelled) {
          set({
            currentRoom: payload.room,
            participants: payload.participants,
            screenPresenter: payload.screen_presenter,
            statusText: `Conectado à sala #${payload.room}`,
          });
        }
      })
      .then((un) => {
        if (isCancelled) {
          un();
        } else {
          unlistenRoom = un;
        }
      })
      .catch((err: unknown) => {
        console.error('Falha ao registrar onRoomState:', err);
      });

    // 3. Room Error listener
    api
      .onRoomError((err) => {
        if (!isCancelled) {
          set({ statusText: `Erro: ${err}` });
        }
      })
      .then((un) => {
        if (isCancelled) {
          un();
        } else {
          unlistenError = un;
        }
      })
      .catch((err: unknown) => {
        console.error('Falha ao registrar onRoomError:', err);
      });

    // 4. Disconnected listener
    api
      .onDisconnected(() => {
        if (!isCancelled) {
          set({
            currentRoom: null,
            participants: [],
            screenPresenter: null,
            isSharingScreen: false,
            statusText: 'Desconectado da sala',
          });
        }
      })
      .then((un) => {
        if (isCancelled) {
          un();
        } else {
          unlistenDisc = un;
        }
      })
      .catch((err: unknown) => {
        console.error('Falha ao registrar onDisconnected:', err);
      });

    // 5. Reconnected listener (screen share does not survive a reconnect)
    api
      .onReconnected(() => {
        if (!isCancelled) {
          set({ isSharingScreen: false, statusText: 'Reconectado à sala' });
        }
      })
      .then((un) => {
        if (isCancelled) {
          un();
        } else {
          unlistenReconnected = un;
        }
      })
      .catch((err: unknown) => {
        console.error('Falha ao registrar onReconnected:', err);
      });

    return () => {
      isCancelled = true;
      unlistenReconnected?.();
      unlistenRoom?.();
      unlistenError?.();
      unlistenDisc?.();
    };
  },

  openJoinCreate: (isCreate = true) => {
    set({
      joinCreateIsModeCreate: isCreate,
      isJoinCreateOpen: true,
    });
  },

  closeJoinCreate: () => {
    set({ isJoinCreateOpen: false });
  },

  openInvite: () => {
    set({ isInviteOpen: true });
  },

  closeInvite: () => {
    set({ isInviteOpen: false });
  },

  closeMonitorPicker: () => {
    set({ isMonitorPickerOpen: false });
  },

  joinCreateSubmit: async (
    isCreate: boolean,
    name: string,
    room: string,
    address: string
  ) => {
    set({
      isJoinCreateOpen: false,
      userName: name,
      statusText: isCreate ? 'Iniciando servidor local...' : `Conectando a ${address}...`,
    });

    try {
      if (isCreate) {
        const info = await api.createRoom(name, room);
        set({
          currentRoom: info.room,
          hostAddress: info.host_address,
          statusText: `Conectado como Host na sala #${info.room}`,
        });
      } else {
        const info = await api.joinRoom(address, room, name);
        set({
          currentRoom: info.room,
          hostAddress: info.host_address,
          statusText: `Conectado à sala #${info.room}`,
        });
      }
    } catch (e: unknown) {
      set({ statusText: `Falha ao conectar: ${String(e)}` });
    }
  },

  leaveRoom: async () => {
    try {
      await api.leaveRoom();
      set({
        currentRoom: null,
        participants: [],
        screenPresenter: null,
        isSharingScreen: false,
        statusText: 'Você saiu da sala',
      });
    } catch (e: unknown) {
      set({ statusText: `Erro ao sair: ${String(e)}` });
    }
  },

  toggleMic: async () => {
    try {
      const state = await api.toggleMic();
      set({ isMicOn: state });
    } catch {}
  },

  toggleDeafen: async () => {
    try {
      const state = await api.toggleDeafen();
      set({ isDeafened: state });
    } catch {}
  },

  startShareClick: async () => {
    try {
      const list = await api.getMonitors();
      if (list.length === 0) {
        set({ statusText: 'Nenhum monitor compatível encontrado' });
      } else if (list.length === 1) {
        await get().selectMonitor(0);
      } else {
        set({
          monitors: list,
          isMonitorPickerOpen: true,
        });
      }
    } catch (e: unknown) {
      set({ statusText: `Erro ao listar monitores: ${String(e)}` });
    }
  },

  selectMonitor: async (index: number) => {
    set({ isMonitorPickerOpen: false });
    try {
      await api.startScreenShare(index);
      set({
        isSharingScreen: true,
        statusText: 'Compartilhamento de tela ativo',
      });
    } catch (e: unknown) {
      set({ statusText: `Falha no compartilhamento: ${String(e)}` });
    }
  },

  stopShareClick: async () => {
    try {
      await api.stopScreenShare();
      set({
        isSharingScreen: false,
        statusText: 'Compartilhamento de tela encerrado',
      });
    } catch {}
  },
}));
