import { create } from 'zustand'
import {
  isTauri,
  tauri,
  type WorkspaceAuditEventView,
  type WorkspaceListItemView,
  type WorkspaceRuntimeLog,
  type WorkspaceSessionView,
  type WorkspaceTrustView,
  type TemporaryRouteView,
  type WebhookEventView,
} from '@/lib/tauriApi'

type WorkspaceState = {
  workspaces: WorkspaceListItemView[]
  selected: WorkspaceTrustView | null
  session: WorkspaceSessionView | null
  logs: WorkspaceRuntimeLog[]
  audit: WorkspaceAuditEventView[]
  temporaryRoutes: TemporaryRouteView[]
  webhookEvents: WebhookEventView[]
  busy: boolean
  error: string | null
  pipelineStage: 'validate' | 'runtime' | 'readiness' | 'tunnel' | 'routes' | 'healthy' | 'stopped'
  pipelineFailed: boolean
  refresh: () => Promise<void>
  inspect: (path: string) => Promise<void>
  approve: () => Promise<void>
  select: (workspace: WorkspaceListItemView) => Promise<void>
  start: () => Promise<void>
  stop: () => Promise<void>
  refreshDetails: () => Promise<void>
  replayWebhook: (routeId: string, eventId: string) => Promise<number>
  reconcileTemporaryRoutes: () => Promise<void>
}

export const useWorkspaceStore = create<WorkspaceState>((set, get) => {
  const run = async (operation: () => Promise<void>) => {
    set({ busy: true, error: null })
    try {
      await operation()
    } catch (error) {
      set({ error: String(error) })
      throw error
    } finally {
      set({ busy: false })
    }
  }

  return {
    workspaces: [],
    selected: null,
    session: null,
    logs: [],
    audit: [],
    temporaryRoutes: [],
    webhookEvents: [],
    busy: false,
    error: null,
    pipelineStage: 'stopped',
    pipelineFailed: false,
    refresh: async () => {
      if (!isTauri()) return
      await run(async () => set({ workspaces: await tauri.workspaceList() }))
    },
    inspect: async (path) => {
      await run(async () => {
        const selected = await tauri.workspaceInspect(path)
        const session = await tauri.workspaceSessionStatus(selected.workspaceId)
        set({
          selected,
          session,
          logs: [],
          audit: [],
          temporaryRoutes: [],
          webhookEvents: [],
          pipelineStage: session?.state === 'healthy' ? 'healthy' : 'validate',
          pipelineFailed: false,
        })
      })
    },
    approve: async () => {
      const selected = get().selected
      if (!selected) return
      await run(async () => {
        set({ selected: await tauri.workspaceApprove(selected.root) })
        set({ workspaces: await tauri.workspaceList() })
      })
    },
    select: async (workspace) => {
      await get().inspect(workspace.root)
      await get().refreshDetails()
    },
    start: async () => {
      const selected = get().selected
      if (!selected) return
      set({ pipelineStage: 'runtime', pipelineFailed: false })
      try {
        await run(async () => {
          set({ session: await tauri.workspaceSessionStart(selected.workspaceId), pipelineStage: 'healthy' })
        })
      } catch (error) {
        const message = String(error).toLowerCase()
        const pipelineStage: WorkspaceState['pipelineStage'] = message.includes('readiness')
          ? 'readiness'
          : message.includes('tunnel')
            ? 'tunnel'
            : message.includes('route')
              ? 'routes'
              : message.includes('trust') || message.includes('approval') || message.includes('manifest')
                ? 'validate'
                : 'runtime'
        set({ pipelineStage, pipelineFailed: true })
        throw error
      }
      await get().refreshDetails()
      await get().refresh()
    },
    stop: async () => {
      const session = get().session
      if (!session) return
      await run(async () => set({ session: await tauri.workspaceSessionStop(session.id), pipelineStage: 'stopped', pipelineFailed: false }))
      await get().refreshDetails()
      await get().refresh()
    },
    refreshDetails: async () => {
      const selected = get().selected
      if (!selected) return
      const session = await tauri.workspaceSessionStatus(selected.workspaceId)
      const [logs, audit, temporaryRoutes] = await Promise.all([
        session ? tauri.workspaceSessionLogs(session.id, 100) : Promise.resolve([]),
        tauri.workspaceAudit(selected.workspaceId),
        session ? tauri.workspaceTemporaryRoutes(session.id) : Promise.resolve([]),
      ])
      const webhookEvents = (await Promise.all(
        temporaryRoutes.map((route) => tauri.workspaceWebhookEvents(route.id, 100)),
      )).flat().sort((left, right) => right.timestamp.localeCompare(left.timestamp))
      set({ session, logs, audit, temporaryRoutes, webhookEvents })
    },
    replayWebhook: async (routeId, eventId) => {
      let status = 0
      await run(async () => { status = await tauri.workspaceWebhookReplay(routeId, eventId) })
      await get().refreshDetails()
      return status
    },
    reconcileTemporaryRoutes: async () => {
      await run(async () => { await tauri.workspaceTemporaryRoutesReconcile() })
      await get().refreshDetails()
    },
  }
})
