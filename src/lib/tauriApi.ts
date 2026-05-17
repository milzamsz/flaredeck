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

export type ProfilePatch = {
  name?: string
  wslHost?: boolean
  accountId?: string | null
  zoneId?: string | null
  zoneName?: string | null
}

export type TokenInfo = {
  valid: boolean
  status?: string | null
  id?: string | null
  expiresOn?: string | null
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

export type ZoneLookup = {
  zoneId: string
  zoneName: string
  accountId: string
  accountName: string | null
}

export const isTauri = (): boolean =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

/**
 * Pre-scoped Cloudflare token creation URL. Three scopes required:
 *   - Account → Cloudflare Tunnel: Edit  (create the tunnel)
 *   - Zone → Zone: Read                   (look up zones by domain)
 *   - Zone → DNS: Edit                    (create CNAME records)
 * `dns:edit` does NOT imply `zone:read` — the lookup-by-name endpoint
 * needs the latter explicitly. Cloudflare's dashboard reads
 * `permissionGroupKeys` to pre-tick the scopes.
 */
export const CF_TOKEN_CREATE_URL =
  'https://dash.cloudflare.com/profile/api-tokens?' +
  'permissionGroupKeys=' +
  encodeURIComponent(
    JSON.stringify([
      { key: 'cfd_tunnel', type: 'edit' },
      { key: 'zone', type: 'read' },
      { key: 'dns', type: 'edit' },
    ]),
  ) +
  '&name=' +
  encodeURIComponent('FlareDeck')

export const CF_TOKEN_DOCS_URL =
  'https://developers.cloudflare.com/fundamentals/api/get-started/create-token/'

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
  /**
   * CLI fallback: only called by `routeDnsForProfile` when the active
   * profile has no API token configured. The API-token path is
   * `cfRouteDns` below.
   */
  tunnelRouteDns: (tunnelName: string, hostname: string) =>
    invoke<void>('tunnel_route_dns', { tunnelName, hostname }),

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
  profilesUpdate: (id: string, patch: ProfilePatch) =>
    invoke<Profile>('profiles_update', { id, patch }),
  profilesDelete: (id: string) =>
    invoke<ProfileIndex>('profiles_delete', { id }),
  profilesSetActive: (id: string) =>
    invoke<ProfileIndex>('profiles_set_active', { id }),
  profilesCreateSimple: (
    name: string,
    token: string,
    reuseTokenFromProfileId: string | null,
    domain: string,
    wslHost: boolean,
  ) =>
    invoke<Profile>('profiles_create_simple', {
      name,
      token,
      reuseTokenFromProfileId,
      domain,
      wslHost,
    }),
  profilesSetToken: (id: string, token: string) =>
    invoke<Profile>('profiles_set_token', { id, token }),
  profilesClearToken: (id: string) =>
    invoke<Profile>('profiles_clear_token', { id }),
  profilesVerifyToken: (id: string) =>
    invoke<TokenInfo>('profiles_verify_token', { id }),

  cfRouteDns: (profileId: string, hostname: string, tunnelId: string) =>
    invoke<string>('cf_route_dns', { profileId, hostname, tunnelId }),
  cfLookupZone: (profileId: string, domain: string) =>
    invoke<ZoneLookup>('cf_lookup_zone', { profileId, domain }),

  wslHostIp: () => invoke<string | null>('wsl_host_ip'),

  prefsGet: () => invoke<AppPrefs>('prefs_get'),
  prefsSetMinimizeToTray: (minimizeToTray: boolean) =>
    invoke<AppPrefs>('prefs_set_minimize_to_tray', { minimizeToTray }),
  prefsMarkTrayHintShown: () => invoke<AppPrefs>('prefs_mark_tray_hint_shown'),
  prefsSetCloseChoice: (minimizeToTray: boolean) =>
    invoke<AppPrefs>('prefs_set_close_choice', { minimizeToTray }),
}

/**
 * Strip scheme, path, port, leading "www.", trailing dot before sending a
 * user-typed domain to the backend. Returns the cleaned string or null
 * if nothing usable remains.
 */
export function normaliseDomainInput(input: string): string | null {
  let s = input.trim().toLowerCase()
  s = s.replace(/^https?:\/\//, '')
  s = s.split('/')[0] ?? ''
  s = s.split('?')[0] ?? ''
  s = s.split(':')[0] ?? ''
  s = s.replace(/\.$/, '')
  s = s.replace(/^www\./, '')
  if (!s || !s.includes('.')) return null
  if (!/^[a-z0-9.-]+$/.test(s)) return null
  return s
}

/**
 * Route a hostname to the profile's tunnel.
 * Uses the Cloudflare REST API when the profile has a token + zoneId,
 * otherwise falls back to `cloudflared tunnel route dns`.
 */
export async function routeDnsForProfile(
  profile: Profile,
  hostname: string,
  tunnelIdOrName: string,
): Promise<{ via: 'api' | 'cli' }> {
  if (profile.hasApiToken && profile.zoneId) {
    await tauri.cfRouteDns(profile.id, hostname, tunnelIdOrName)
    return { via: 'api' }
  }
  await tauri.tunnelRouteDns(tunnelIdOrName, hostname)
  return { via: 'cli' }
}

export type { IngressRule }
