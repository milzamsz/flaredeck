import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { isTauri, tauri } from '@/lib/tauriApi'
import {
  ingressToProxyItems,
  parseConfigYaml,
  parseServiceUrl,
  serializeConfig,
} from '@/lib/yaml-helpers'

export type TunnelLifecycle =
  | 'idle'
  | 'starting'
  | 'running'
  | 'stopping'
  | 'stopped'
  | 'error'

export type ProxyItem = {
  id: string
  hostname: string
  service: string
  path?: string
  portStatus?: 'open' | 'closed' | 'unknown'
  dnsStatus?: 'resolved' | 'unresolved' | 'unknown'
}

export type Profile = {
  id: string
  name: string
  tunnelName: string
  configPath: string
  wslHost: boolean
}

export type LogEntry = {
  id: string
  ts: number
  level: 'info' | 'warn' | 'error'
  source: string
  message: string
}

export type IngressRule = {
  hostname?: string
  path?: string
  service: string
  originRequest?: {
    connectTimeout?: string
    noTLSVerify?: boolean
    httpHostHeader?: string
  }
}

export type CloudflaredConfig = {
  tunnel?: string
  'credentials-file'?: string
  ingress?: IngressRule[]
} & Record<string, unknown>

type PersistedSlice = {
  theme: 'light' | 'dark' | 'system'
  activeProfileId: string | null
}

type AppState = PersistedSlice & {
  profiles: Profile[]
  tunnelStatus: TunnelLifecycle
  tunnelPid: number | null
  isAuthenticated: boolean
  certPath: string | null
  cloudflaredPath: string | null
  cloudflaredInstalled: boolean
  cloudflaredVersion: string | null
  wslHostIp: string | null
  minimizeToTray: boolean
  trayHintShown: boolean
  closeChoiceMade: boolean
  config: CloudflaredConfig | null
  configRaw: string
  configPath: string | null
  proxyItems: ProxyItem[]
  logs: LogEntry[]

  setTheme: (theme: PersistedSlice['theme']) => void
  setActiveProfile: (id: string | null) => Promise<void>
  appendLog: (entry: Omit<LogEntry, 'id' | 'ts'>) => void
  clearLogs: () => void

  bootstrap: () => Promise<void>
  refreshCloudflared: () => Promise<void>
  refreshAuth: () => Promise<void>
  refreshProfiles: () => Promise<void>
  refreshTunnelStatus: () => Promise<void>
  refreshWslHostIp: () => Promise<void>
  refreshPrefs: () => Promise<void>
  setMinimizeToTray: (value: boolean) => Promise<void>
  markTrayHintShown: () => Promise<void>
  setCloseChoice: (minimizeToTray: boolean) => Promise<void>
  loadConfig: () => Promise<void>
  saveConfig: (raw: string) => Promise<void>
  saveProxyItems: () => Promise<void>
  startTunnel: () => Promise<void>
  stopTunnel: () => Promise<void>
  restartTunnel: () => Promise<void>
  checkAllProxyPorts: () => Promise<void>
  checkAllDnsStatus: () => Promise<void>
  setProxyItems: (items: ProxyItem[]) => void
  addProxyItem: (item: Omit<ProxyItem, 'id' | 'portStatus' | 'dnsStatus'>) => Promise<void>
  updateProxyItem: (id: string, patch: Partial<ProxyItem>) => Promise<void>
  removeProxyItem: (id: string) => Promise<void>
  createProfile: (
    name: string,
    tunnelName: string,
    wslHost: boolean,
    createTunnel: boolean,
  ) => Promise<void>
  updateActiveProfile: (patch: { name?: string; wslHost?: boolean }) => Promise<void>
  deleteProfile: (id: string) => Promise<void>
}

const newLogId = () =>
  `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`

export const useAppStore = create<AppState>()(
  persist(
    (set, get) => ({
      theme: 'system',
      activeProfileId: null,

      profiles: [],
      tunnelStatus: 'idle',
      tunnelPid: null,
      isAuthenticated: false,
      certPath: null,
      cloudflaredPath: null,
      cloudflaredInstalled: false,
      cloudflaredVersion: null,
      wslHostIp: null,
      minimizeToTray: false,
      trayHintShown: false,
      closeChoiceMade: false,
      config: null,
      configRaw: '',
      configPath: null,
      proxyItems: [],
      logs: [],

      setTheme: (theme) => set({ theme }),

      setActiveProfile: async (activeProfileId) => {
        set({ activeProfileId })
        if (activeProfileId && isTauri()) {
          try {
            await tauri.profilesSetActive(activeProfileId)
          } catch (e) {
            get().appendLog({
              level: 'warn',
              source: 'profiles',
              message: `Failed to persist active profile: ${String(e)}`,
            })
          }
          await get().loadConfig()
          await get().refreshTunnelStatus()
        }
      },

      appendLog: (entry) =>
        set((state) => ({
          logs: [
            ...state.logs.slice(-199),
            { id: newLogId(), ts: Date.now(), ...entry },
          ],
        })),
      clearLogs: () => set({ logs: [] }),

      bootstrap: async () => {
        if (!isTauri()) return
        await get().refreshPrefs()
        await get().refreshCloudflared()
        await get().refreshAuth()
        await get().refreshWslHostIp()
        await get().refreshProfiles()
        const id = get().activeProfileId
        if (id) {
          await get().loadConfig()
          await get().refreshTunnelStatus()
        }
      },

      refreshPrefs: async () => {
        if (!isTauri()) return
        try {
          const prefs = await tauri.prefsGet()
          set({
            minimizeToTray: prefs.minimizeToTray,
            trayHintShown: prefs.trayHintShown,
            closeChoiceMade: prefs.closeChoiceMade,
          })
        } catch (e) {
          get().appendLog({
            level: 'warn',
            source: 'prefs',
            message: `prefs_get failed: ${String(e)}`,
          })
        }
      },

      setMinimizeToTray: async (value) => {
        if (!isTauri()) {
          set({ minimizeToTray: value })
          return
        }
        try {
          const prefs = await tauri.prefsSetMinimizeToTray(value)
          set({
            minimizeToTray: prefs.minimizeToTray,
            trayHintShown: prefs.trayHintShown,
            closeChoiceMade: prefs.closeChoiceMade,
          })
        } catch (e) {
          get().appendLog({
            level: 'error',
            source: 'prefs',
            message: `prefs_set_minimize_to_tray failed: ${String(e)}`,
          })
          throw e
        }
      },

      setCloseChoice: async (minimizeToTray) => {
        if (!isTauri()) {
          set({ minimizeToTray, closeChoiceMade: true })
          return
        }
        try {
          const prefs = await tauri.prefsSetCloseChoice(minimizeToTray)
          set({
            minimizeToTray: prefs.minimizeToTray,
            trayHintShown: prefs.trayHintShown,
            closeChoiceMade: prefs.closeChoiceMade,
          })
        } catch (e) {
          get().appendLog({
            level: 'error',
            source: 'prefs',
            message: `prefs_set_close_choice failed: ${String(e)}`,
          })
          throw e
        }
      },

      markTrayHintShown: async () => {
        if (!isTauri()) return
        try {
          const prefs = await tauri.prefsMarkTrayHintShown()
          set({ trayHintShown: prefs.trayHintShown })
        } catch {
          set({ trayHintShown: true })
        }
      },

      refreshWslHostIp: async () => {
        if (!isTauri()) return
        try {
          const ip = await tauri.wslHostIp()
          set({ wslHostIp: ip ?? null })
        } catch (e) {
          get().appendLog({
            level: 'warn',
            source: 'wsl',
            message: `wsl_host_ip failed: ${String(e)}`,
          })
        }
      },

      refreshCloudflared: async () => {
        if (!isTauri()) return
        try {
          const info = await tauri.cloudflaredCheck()
          set({
            cloudflaredInstalled: info.installed,
            cloudflaredPath: info.path ?? null,
            cloudflaredVersion: info.version ?? null,
          })
        } catch (e) {
          get().appendLog({
            level: 'error',
            source: 'cloudflared',
            message: `cloudflared_check failed: ${String(e)}`,
          })
        }
      },

      refreshAuth: async () => {
        if (!isTauri()) return
        try {
          const status = await tauri.authCheck()
          set({
            isAuthenticated: status.authenticated,
            certPath: status.certPath ?? null,
          })
        } catch (e) {
          get().appendLog({
            level: 'error',
            source: 'auth',
            message: `auth_check failed: ${String(e)}`,
          })
        }
      },

      refreshProfiles: async () => {
        if (!isTauri()) return
        try {
          const index = await tauri.profilesList()
          const current = get().activeProfileId
          const active =
            (current && index.profiles.some((p) => p.id === current) && current) ||
            index.activeProfileId ||
            index.profiles[0]?.id ||
            null
          set({ profiles: index.profiles, activeProfileId: active })
        } catch (e) {
          get().appendLog({
            level: 'error',
            source: 'profiles',
            message: `profiles_list failed: ${String(e)}`,
          })
        }
      },

      refreshTunnelStatus: async () => {
        const id = get().activeProfileId
        if (!id || !isTauri()) {
          set({ tunnelStatus: 'idle', tunnelPid: null })
          return
        }
        try {
          const status = await tauri.tunnelStatus(id)
          set({
            tunnelStatus: status.running ? 'running' : 'stopped',
            tunnelPid: status.pid ?? null,
          })
        } catch (e) {
          set({ tunnelStatus: 'error' })
          get().appendLog({
            level: 'error',
            source: 'tunnel',
            message: `tunnel_status failed: ${String(e)}`,
          })
        }
      },

      loadConfig: async () => {
        const id = get().activeProfileId
        if (!id || !isTauri()) return
        try {
          const payload = await tauri.configGet(id)
          const parsed = payload.parsed ?? parseConfigYaml(payload.raw)
          set({
            config: parsed,
            configRaw: payload.raw,
            configPath: payload.path,
            proxyItems: ingressToProxyItems(parsed?.ingress),
          })
        } catch (e) {
          get().appendLog({
            level: 'error',
            source: 'config',
            message: `config_get failed: ${String(e)}`,
          })
        }
      },

      saveConfig: async (raw) => {
        const id = get().activeProfileId
        if (!id || !isTauri()) return
        try {
          const payload = await tauri.configSave(id, raw)
          const parsed = payload.parsed ?? parseConfigYaml(payload.raw)
          set({
            config: parsed,
            configRaw: payload.raw,
            configPath: payload.path,
            proxyItems: ingressToProxyItems(parsed?.ingress),
          })
          get().appendLog({
            level: 'info',
            source: 'config',
            message: `Saved ${payload.path}`,
          })
        } catch (e) {
          get().appendLog({
            level: 'error',
            source: 'config',
            message: `config_save failed: ${String(e)}`,
          })
          throw e
        }
      },

      saveProxyItems: async () => {
        const {
          config,
          proxyItems,
          saveConfig,
          activeProfileId,
          profiles,
          wslHostIp,
        } = get()
        const active = profiles.find((p) => p.id === activeProfileId)
        const hostRewrite =
          active?.wslHost && wslHostIp ? wslHostIp : undefined
        const base = config ?? {}
        const raw = serializeConfig(base, proxyItems, { hostRewrite })
        await saveConfig(raw)
      },

      startTunnel: async () => {
        const id = get().activeProfileId
        if (!id || !isTauri()) return
        set({ tunnelStatus: 'starting' })
        try {
          const status = await tauri.tunnelStart(id)
          set({
            tunnelStatus: status.running ? 'running' : 'stopped',
            tunnelPid: status.pid ?? null,
          })
          get().appendLog({
            level: 'info',
            source: 'tunnel',
            message: `Tunnel started (pid ${status.pid ?? '?'})`,
          })
        } catch (e) {
          set({ tunnelStatus: 'error' })
          get().appendLog({
            level: 'error',
            source: 'tunnel',
            message: `tunnel_start failed: ${String(e)}`,
          })
          throw e
        }
      },

      stopTunnel: async () => {
        const id = get().activeProfileId
        if (!id || !isTauri()) return
        set({ tunnelStatus: 'stopping' })
        try {
          await tauri.tunnelStop(id)
          set({ tunnelStatus: 'stopped', tunnelPid: null })
          get().appendLog({
            level: 'info',
            source: 'tunnel',
            message: 'Tunnel stopped',
          })
        } catch (e) {
          set({ tunnelStatus: 'error' })
          get().appendLog({
            level: 'error',
            source: 'tunnel',
            message: `tunnel_stop failed: ${String(e)}`,
          })
          throw e
        }
      },

      restartTunnel: async () => {
        const id = get().activeProfileId
        if (!id || !isTauri()) return
        set({ tunnelStatus: 'starting' })
        try {
          const status = await tauri.tunnelRestart(id)
          set({
            tunnelStatus: status.running ? 'running' : 'stopped',
            tunnelPid: status.pid ?? null,
          })
          get().appendLog({
            level: 'info',
            source: 'tunnel',
            message: `Tunnel restarted (pid ${status.pid ?? '?'})`,
          })
        } catch (e) {
          set({ tunnelStatus: 'error' })
          get().appendLog({
            level: 'error',
            source: 'tunnel',
            message: `tunnel_restart failed: ${String(e)}`,
          })
          throw e
        }
      },

      checkAllProxyPorts: async () => {
        if (!isTauri()) return
        const {
          proxyItems: items,
          activeProfileId,
          profiles,
          wslHostIp,
        } = get()
        const active = profiles.find((p) => p.id === activeProfileId)
        const rewriteHost =
          active?.wslHost && wslHostIp ? wslHostIp : null
        const results = await Promise.all(
          items.map(async (item) => {
            const { host, port } = parseServiceUrl(item.service)
            const effectiveHost =
              rewriteHost &&
              (host === 'localhost' || host === '127.0.0.1' || host === '0.0.0.0')
                ? rewriteHost
                : host
            try {
              const open = await tauri.networkCheckPort(effectiveHost, port)
              return { id: item.id, status: open ? 'open' : 'closed' } as const
            } catch {
              return { id: item.id, status: 'unknown' } as const
            }
          }),
        )
        set({
          proxyItems: get().proxyItems.map((item) => {
            const found = results.find((r) => r.id === item.id)
            return found ? { ...item, portStatus: found.status } : item
          }),
        })
      },

      checkAllDnsStatus: async () => {
        if (!isTauri()) return
        const items = get().proxyItems
        const results = await Promise.all(
          items.map(async (item) => {
            try {
              const r = await tauri.dnsCheck(item.hostname)
              return { id: item.id, status: r.resolved ? 'resolved' : 'unresolved' } as const
            } catch {
              return { id: item.id, status: 'unknown' } as const
            }
          }),
        )
        set({
          proxyItems: get().proxyItems.map((item) => {
            const found = results.find((r) => r.id === item.id)
            return found ? { ...item, dnsStatus: found.status } : item
          }),
        })
      },

      setProxyItems: (items) => set({ proxyItems: items }),

      addProxyItem: async (item) => {
        const id = `ingress-new-${Date.now()}`
        set((state) => ({
          proxyItems: [
            ...state.proxyItems,
            { ...item, id, portStatus: 'unknown', dnsStatus: 'unknown' },
          ],
        }))
        await get().saveProxyItems()
      },

      updateProxyItem: async (id, patch) => {
        set((state) => ({
          proxyItems: state.proxyItems.map((p) =>
            p.id === id ? { ...p, ...patch } : p,
          ),
        }))
        await get().saveProxyItems()
      },

      removeProxyItem: async (id) => {
        set((state) => ({
          proxyItems: state.proxyItems.filter((p) => p.id !== id),
        }))
        await get().saveProxyItems()
      },

      createProfile: async (name, tunnelName, wslHost, createTunnel) => {
        if (!isTauri()) return
        try {
          const profile = await tauri.profilesCreate(
            name,
            tunnelName,
            wslHost,
            createTunnel,
          )
          await get().refreshProfiles()
          await get().setActiveProfile(profile.id)
        } catch (e) {
          get().appendLog({
            level: 'error',
            source: 'profiles',
            message: `profiles_create failed: ${String(e)}`,
          })
          throw e
        }
      },

      updateActiveProfile: async (patch) => {
        const id = get().activeProfileId
        if (!id || !isTauri()) return
        try {
          await tauri.profilesUpdate(id, patch)
          await get().refreshProfiles()
        } catch (e) {
          get().appendLog({
            level: 'error',
            source: 'profiles',
            message: `profiles_update failed: ${String(e)}`,
          })
          throw e
        }
      },

      deleteProfile: async (id) => {
        if (!isTauri()) return
        try {
          await tauri.profilesDelete(id)
          if (get().activeProfileId === id) {
            set({
              config: null,
              configRaw: '',
              configPath: null,
              proxyItems: [],
              tunnelStatus: 'idle',
              tunnelPid: null,
              activeProfileId: null,
            })
          }
          await get().refreshProfiles()
          const next = get().activeProfileId
          if (next) {
            await get().loadConfig()
            await get().refreshTunnelStatus()
          }
        } catch (e) {
          get().appendLog({
            level: 'error',
            source: 'profiles',
            message: `profiles_delete failed: ${String(e)}`,
          })
          throw e
        }
      },
    }),
    {
      name: 'flaredeck.app-store',
      partialize: (state): PersistedSlice => ({
        theme: state.theme,
        activeProfileId: state.activeProfileId,
      }),
    },
  ),
)
