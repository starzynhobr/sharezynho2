import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { NetworkInfo, RoomJoinedInfo, RoomStatePayload } from './types';

export const api = {
  showWindow: async (): Promise<void> => {
    try {
      await getCurrentWindow().show();
    } catch {}
  },
  getNetworkInfo: async (): Promise<NetworkInfo> => {
    return invoke('get_network_info');
  },

  createRoom: async (name: string, room: string): Promise<RoomJoinedInfo> => {
    return invoke('create_room', { name, room });
  },

  joinRoom: async (address: string, room: string, name: string): Promise<RoomJoinedInfo> => {
    return invoke('join_room', { address, room, name });
  },

  leaveRoom: async (): Promise<void> => {
    return invoke('leave_room');
  },

  toggleMic: async (): Promise<boolean> => {
    return invoke('toggle_mic');
  },

  toggleDeafen: async (): Promise<boolean> => {
    return invoke('toggle_deafen');
  },

  getMonitors: async (): Promise<string[]> => {
    return invoke('get_monitors');
  },

  startScreenShare: async (monitorIndex: number): Promise<void> => {
    return invoke('start_screen_share', { monitorIndex });
  },

  stopScreenShare: async (): Promise<void> => {
    return invoke('stop_screen_share');
  },

  onRoomState: (callback: (payload: RoomStatePayload) => void): Promise<UnlistenFn> => {
    return listen<RoomStatePayload>('room-state', (event) => callback(event.payload));
  },

  onRoomError: (callback: (error: string) => void): Promise<UnlistenFn> => {
    return listen<string>('room-error', (event) => callback(event.payload));
  },

  onDisconnected: (callback: () => void): Promise<UnlistenFn> => {
    return listen('disconnected', () => callback());
  },
};
