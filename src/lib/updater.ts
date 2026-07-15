import { useCallback, useEffect, useState } from 'react'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

import { isTauri } from '@/lib/tauriApi'

export type UpdaterState =
  | { kind: 'idle' }
  | { kind: 'checking' }
  | { kind: 'upToDate'; checkedAt: number }
  | { kind: 'available'; update: Update }
  | { kind: 'downloading'; downloaded: number; total: number | null }
  | { kind: 'ready' }
  | { kind: 'error'; message: string }

/**
 * Tauri updater hook. Polls once on mount, exposes manual recheck +
 * install actions, and tracks download progress for the UI.
 *
 * The plugin reads its endpoints + pubkey from `tauri.conf.json` —
 * we never pass URLs from JS land (they're trust-locked at build).
 */
export function useUpdater() {
  const [state, setState] = useState<UpdaterState>({ kind: 'idle' })

  const checkOnce = useCallback(async () => {
    if (!isTauri()) {
      setState({ kind: 'error', message: 'Updater is only available in the desktop app.' })
      return
    }
    setState({ kind: 'checking' })
    try {
      const update = await check()
      if (update) {
        setState({ kind: 'available', update })
      } else {
        setState({ kind: 'upToDate', checkedAt: Date.now() })
      }
    } catch (e) {
      setState({ kind: 'error', message: String(e) })
    }
  }, [])

  useEffect(() => {
    if (!isTauri()) return
    const timer = window.setTimeout(() => void checkOnce(), 0)
    return () => window.clearTimeout(timer)
  }, [checkOnce])

  const downloadAndInstall = async () => {
    if (state.kind !== 'available') return
    const { update } = state
    let downloaded = 0
    let total: number | null = null
    setState({ kind: 'downloading', downloaded, total })
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          total = event.data.contentLength ?? null
          setState({ kind: 'downloading', downloaded: 0, total })
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength
          setState({ kind: 'downloading', downloaded, total })
        } else if (event.event === 'Finished') {
          setState({ kind: 'ready' })
        }
      })
    } catch (e) {
      setState({ kind: 'error', message: String(e) })
    }
  }

  const restartNow = async () => {
    try {
      await relaunch()
    } catch (e) {
      setState({ kind: 'error', message: `Restart failed: ${String(e)}` })
    }
  }

  return { state, checkOnce, downloadAndInstall, restartNow }
}
