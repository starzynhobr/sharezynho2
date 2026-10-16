export interface NetworkInfo {
  preferred_ip: string;
  is_tailscale: boolean;
  tcp_port: number;
  udp_port: number;
  stream_port: number;
}

export interface ParticipantInfo {
  id: string;
  name: string;
  is_speaking: boolean;
  is_host: boolean;
  is_sharing_screen: boolean;
}

export interface RoomJoinedInfo {
  room: string;
  display_name: string;
  is_host: boolean;
  host_address: string;
}

export interface RoomStatePayload {
  room: string;
  participants: ParticipantInfo[];
  screen_presenter: string | null;
}
