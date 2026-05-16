import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Outlet } from 'react-router-dom'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { toast } from 'sonner'

import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { SidebarInset, SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import { AppSidebar } from '@/components/app-sidebar'
import { useAppStore } from '@/store/app-store'
import { isTauri } from '@/lib/tauriApi'

type TunnelLogEvent = {
  profileId: string
  stream: 'stdout' | 'stderr'
  line: string
}

export function AppLayout() {
  const { t } = useTranslation()
  const bootstrap = useAppStore((s) => s.bootstrap)
  const setCloseChoice = useAppStore((s) => s.setCloseChoice)
  const [closePromptOpen, setClosePromptOpen] = useState(false)

  useEffect(() => {
    if (!isTauri()) return
    void bootstrap()
    const interval = setInterval(() => {
      void useAppStore.getState().refreshTunnelStatus()
    }, 5000)

    let unlistenLog: (() => void) | null = null
    let unlistenHidden: (() => void) | null = null
    let unlistenFirstClose: (() => void) | null = null

    listen<TunnelLogEvent>('tunnel:log', (event) => {
      const { stream, line } = event.payload
      if (!line.trim()) return
      useAppStore.getState().appendLog({
        level: stream === 'stderr' ? 'warn' : 'info',
        source: 'cloudflared',
        message: line,
      })
    }).then((fn) => {
      unlistenLog = fn
    })

    listen('window:hidden-to-tray', () => {
      const state = useAppStore.getState()
      if (state.trayHintShown) return
      toast.message(t('tray.hiddenTitle'), {
        description: t('tray.hiddenBody'),
        duration: 8000,
      })
      void state.markTrayHintShown()
    }).then((fn) => {
      unlistenHidden = fn
    })

    listen('window:first-close-prompt', () => {
      setClosePromptOpen(true)
    }).then((fn) => {
      unlistenFirstClose = fn
    })

    return () => {
      clearInterval(interval)
      unlistenLog?.()
      unlistenHidden?.()
      unlistenFirstClose?.()
    }
  }, [bootstrap, t])

  const handleKeepRunning = async () => {
    try {
      await setCloseChoice(true)
      setClosePromptOpen(false)
      await getCurrentWindow().hide()
    } catch (e) {
      toast.error(String(e))
    }
  }

  const handleQuit = async () => {
    try {
      await setCloseChoice(false)
      setClosePromptOpen(false)
      await getCurrentWindow().close()
    } catch (e) {
      toast.error(String(e))
    }
  }

  return (
    <>
      <SidebarProvider>
        <AppSidebar />
        <SidebarInset>
          <header className="flex h-12 shrink-0 items-center gap-2 border-b px-4">
            <SidebarTrigger />
          </header>
          <div className="flex-1 overflow-auto">
            <Outlet />
          </div>
        </SidebarInset>
      </SidebarProvider>

      <AlertDialog open={closePromptOpen} onOpenChange={setClosePromptOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('closePrompt.title')}</AlertDialogTitle>
            <AlertDialogDescription>{t('closePrompt.body')}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <Button variant="ghost" onClick={() => setClosePromptOpen(false)}>
              {t('closePrompt.cancel')}
            </Button>
            <Button variant="outline" onClick={() => void handleQuit()}>
              {t('closePrompt.quit')}
            </Button>
            <Button onClick={() => void handleKeepRunning()}>
              {t('closePrompt.keepRunning')}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}
