import { invoke, isTauri } from '@tauri-apps/api/core';
export interface Room {
  id: string;
  context: string;
  name: string;
  description: string;
  owner: string;
  archived: boolean;
  muted: boolean;
  unread: number;
}
export interface Message {
  id: string;
  room: string;
  author: string;
  profile: string;
  content: string;
  mode: 'text' | 'code';
  logical: number;
  timestamp: number;
  delivery: 'waiting' | 'shared' | 'received';
}
export interface NetworkContext {
  id: string;
  label: string;
  current: boolean;
}
export interface Peer {
  id: string;
  name: string;
  platform: string;
  status: 'connected' | 'reconnecting' | 'unavailable';
}
export type Theme = 'system' | 'light' | 'dark';
export type StartupSnapshot = { theme: Theme | null } & (
  | { status: 'loading' }
  | { status: 'ready' }
  | { status: 'failed'; message: string }
);
export interface Snapshot {
  identity: string;
  profile: string;
  theme: Theme;
  notifications: boolean;
  context: string;
  contexts: NetworkContext[];
  rooms: Room[];
  peers: Peer[];
  network_status: 'discovering' | 'connected' | 'reconnecting' | 'unavailable';
  network_detail: string;
}
export interface Draft {
  content: string;
  code: boolean;
}
export interface SearchResult {
  excerpt: string;
  message: Message;
  context: string;
  room_name: string;
  current: boolean;
}
export type UpdateChannel = 'stable' | 'beta';
type UpdateRelease = { version: string; notes: string };
export type UpdateSnapshot = {
  version: string;
  channel: UpdateChannel;
  automatic: boolean;
  last_checked: number;
  dismissed: string;
  in_app: boolean;
} & (
  | { status: 'idle' | 'checking' | 'current' | 'installing' }
  | { status: 'available' | 'ready'; release: UpdateRelease }
  | {
      status: 'downloading';
      release: UpdateRelease;
      downloaded: number;
      total: number | null;
    }
  | { status: 'failed'; message: string }
);

export const desktop = isTauri();
export const api = {
  updateStatus: () => invoke<UpdateSnapshot>('update_status'),
  checkUpdates: () => invoke<void>('check_updates'),
  downloadUpdate: () => invoke<void>('download_update'),
  installUpdate: () => invoke<void>('install_update'),
  updatePreferences: (automatic: boolean, channel: UpdateChannel) =>
    invoke<void>('update_preferences', { automatic, channel }),
  dismissUpdate: () => invoke<void>('dismiss_update'),
  openUpdateDownload: () => invoke<void>('open_update_download'),
  startup: () => invoke<StartupSnapshot>('startup_status'),
  retryStartup: () => invoke<void>('retry_startup'),
  showWindow: () => invoke<void>('show_window'),
  snapshot: () => invoke<Snapshot>('snapshot'),
  reconnect: () => invoke<void>('reconnect'),
  rooms: (context: string) => invoke<Room[]>('rooms', { context }),
  messages: (
    context: string,
    room: string,
    before: string | null = null,
    around: string | null = null,
  ) =>
    invoke<{ items: Message[]; has_older: boolean; has_newer: boolean }>(
      'messages',
      { context, room, before, around },
    ),
  send: (context: string, room: string, content: string, code: boolean) =>
    invoke<string>('send_message', { context, room, content, code }),
  manage: (
    context: string,
    action: 'create' | 'rename' | 'archive' | 'restore',
    room: string | null,
    name = '',
    description = '',
  ) =>
    invoke<string>('manage_room', { context, action, room, name, description }),
  draft: (context: string, room: string) =>
    invoke<Draft>('draft', { context, room }),
  saveDraft: (context: string, room: string, content: string, code: boolean) =>
    invoke<void>('save_draft', { context, room, content, code }),
  local: (
    context: string,
    room: string,
    muted: boolean | null = null,
    read: number | null = null,
    scroll: number | null = null,
  ) => invoke<number>('room_local', { context, room, muted, read, scroll }),
  view: (context: string, room: string) =>
    invoke<{ anchor: string; offset: number } | null>('view_state', {
      context,
      room,
    }),
  saveView: (context: string, room: string, anchor: string, offset: number) =>
    invoke<void>('save_view', { context, room, anchor, offset }),
  search: (query: string, room: string | null, context: string) =>
    invoke<SearchResult[]>('search', { query, room, context }),
  preferences: (profile: string, theme: Theme, notifications: boolean) =>
    invoke<void>('preferences', { profile, theme, notifications }),
  recordRoomSwitch: (milliseconds: number) =>
    invoke<void>('record_room_switch', { milliseconds }),
  ready: () => invoke<void>('ready'),
};
