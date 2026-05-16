import { invoke } from '@tauri-apps/api/core'
import type {
  CloudflaredConfig,
  IngressRule,
  Profile,
} from '@/store/app-store'

export type CloudflaredInfo = {
  installed: boolean
  path?: string | null
  version?: string | null
}

export type AuthStatus = {
  authenticated: boolean
  certPath?: string | null
}

export type TunnelStatus = {
  profileId: string
  running: boolean
  pid?: number | null
}

export type ConfigPayload = {
  path: string
  raw: string
  parsed: CloudflaredConfig | null
}

export type DnsLookupResult = {
  resolved: boolean
  addresses: string[]
}

export type TunnelListEntry = {
  id: string
  name: string
  createdAt?: string
}

export type CreatedTunnel = {
  uuid: string
  name: string
  credentialsFile: string
}

export type ProfilePatch = {
  name?: string
  wslHost?: boolean
}

export type AppPrefs = {
  minimizeToTray: boolean
  trayHintShown: boolean
  closeChoiceMade: boolean
}

export type ProfileIndex = {
  profiles: Profile[]
  activeProfileId: string | null
}

export const isTauri = (): boolean =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export const tauri = {
  appVersion: () => invoke<string>('app_version'),

  cloudflaredCheck: () => invoke<CloudflaredInfo>('cloudflared_check'),

  tunnelStatus: (profileId: string) =>
    invoke<TunnelStatus>('tunnel_status', { profileId }),
  tunnelStart: (profileId: string) =>
    invoke<TunnelStatus>('tunnel_start', { profileId }),
  tunnelStop: (profileId: string) =>
    invoke<TunnelStatus>('tunnel_stop', { profileId }),
  tunnelRestart: (profileId: string) =>
    invoke<TunnelStatus>('tunnel_restart', { profileId }),
  tunnelList: () => invoke<TunnelListEntry[]>('tunnel_list'),
  tunnelRouteDns: (tunnelName: string, hostname: string) =>
    invoke<void>('tunnel_route_dns', { tunnelName, hostname }),
  tunnelCreate: (name: string) =>
    invoke<CreatedTunnel>('tunnel_create', { name }),

  authCheck: () => invoke<AuthStatus>('auth_check'),
  authLogin: () => invoke<void>('auth_login'),
  authLogout: () => invoke<void>('auth_logout'),

  configGet: (profileId: string) =>
    invoke<ConfigPayload>('config_get', { profileId }),
  configSave: (profileId: string, raw: string) =>
    invoke<ConfigPayload>('config_save', { profileId, raw }),

  networkCheckPort: (host: string, port: number) =>
    invoke<boolean>('network_check_port', { host, port }),

  dnsCheck: (hostname: string) =>
    invoke<DnsLookupResult>('dns_check', { hostname }),

  shellOpenExternal: (url: string) =>
    invoke<void>('shell_open_external', { url }),
  shellOpenPath: (path: string) => invoke<void>('shell_open_path', { path }),

  profilesList: () => invoke<ProfileIndex>('profiles_list'),
  profilesCreate: (
    name: string,
    tunnelName: string,
    wslHost: boolean,
    createTunnel: boolean,
  ) =>
    invoke<Profile>('profiles_create', {
      name,
      tunnelName,
      wslHost,
      createTunnel,
    }),
  profilesUpdate: (id: string, patch: ProfilePatch) =>
    invoke<Profile>('profiles_update', { id, patch }),
  profilesDelete: (id: string) =>
    invoke<ProfileIndex>('profiles_delete', { id }),
  profilesSetActive: (id: string) =>
    invoke<ProfileIndex>('profiles_set_active', { id }),

  wslHostIp: () => invoke<string | null>('wsl_host_ip'),

  prefsGet: () => invoke<AppPrefs>('prefs_get'),
  prefsSetMinimizeToTray: (minimizeToTray: boolean) =>
    invoke<AppPrefs>('prefs_set_minimize_to_tray', { minimizeToTray }),
  prefsMarkTrayHintShown: () => invoke<AppPrefs>('prefs_mark_tray_hint_shown'),
  prefsSetCloseChoice: (minimizeToTray: boolean) =>
    invoke<AppPrefs>('prefs_set_close_choice', { minimizeToTray }),
}

export type { IngressRule }
