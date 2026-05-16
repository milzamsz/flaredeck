import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Play, Plus, RefreshCw, Square } from 'lucide-react'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { LogViewer } from '@/components/log-viewer'
import { ProxyFormDialog, type ProxyFormValues } from '@/components/proxy-form-dialog'
import { ProxyTable } from '@/components/proxy-table'
import { useAppStore, type ProxyItem, type TunnelLifecycle } from '@/store/app-store'
import { tauri } from '@/lib/tauriApi'

function NewProfileDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (v: boolean) => void
}) {
  const { t } = useTranslation()
  const createProfile = useAppStore((s) => s.createProfile)
  const isAuthenticated = useAppStore((s) => s.isAuthenticated)
  const wslHostIp = useAppStore((s) => s.wslHostIp)
  const [name, setName] = useState('')
  const [tunnel, setTunnel] = useState('')
  const [wslHost, setWslHost] = useState(true)
  const [createTunnel, setCreateTunnel] = useState(true)
  const [submitting, setSubmitting] = useState(false)

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('profile.newTitle')}</DialogTitle>
          <DialogDescription>{t('profile.newDescription')}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="profile-name">{t('profile.displayName')}</Label>
            <Input
              id="profile-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t('profile.displayNamePlaceholder')}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="tunnel-name">{t('profile.tunnelName')}</Label>
            <Input
              id="tunnel-name"
              value={tunnel}
              onChange={(e) => setTunnel(e.target.value)}
              placeholder={t('profile.tunnelNamePlaceholder')}
            />
          </div>
          <div className="flex items-start gap-2">
            <input
              id="profile-create-tunnel"
              type="checkbox"
              className="mt-1"
              checked={createTunnel}
              onChange={(e) => setCreateTunnel(e.target.checked)}
            />
            <Label htmlFor="profile-create-tunnel" className="leading-tight">
              <span className="block">{t('profile.createTunnel')}</span>
              <span className="block text-xs font-normal text-muted-foreground">
                {t('profile.createTunnelHelp')}
              </span>
            </Label>
          </div>
          <div className="flex items-start gap-2">
            <input
              id="profile-wsl-host"
              type="checkbox"
              className="mt-1"
              checked={wslHost}
              onChange={(e) => setWslHost(e.target.checked)}
            />
            <Label htmlFor="profile-wsl-host" className="leading-tight">
              <span className="block">{t('profile.wslHost')}</span>
              <span className="block text-xs font-normal text-muted-foreground">
                {wslHostIp
                  ? t('profile.wslHostHelpDetected', { ip: wslHostIp })
                  : t('profile.wslHostHelp')}
              </span>
            </Label>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('profile.cancel')}
          </Button>
          <Button
            disabled={!name.trim() || !tunnel.trim() || submitting}
            onClick={async () => {
              if (createTunnel && !isAuthenticated) {
                toast.error(t('profile.signInRequired'))
                return
              }
              setSubmitting(true)
              try {
                await createProfile(
                  name.trim(),
                  tunnel.trim(),
                  wslHost,
                  createTunnel,
                )
                setName('')
                setTunnel('')
                onOpenChange(false)
                toast.success(t('profile.created'))
              } catch (e) {
                toast.error(t('profile.createFailed', { message: String(e) }))
              } finally {
                setSubmitting(false)
              }
            }}
          >
            {t('profile.create')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function StatusBadge({ status }: { status: TunnelLifecycle }) {
  const { t } = useTranslation()
  if (status === 'running') return <Badge className="bg-green-600">{t('status.running')}</Badge>
  if (status === 'starting') return <Badge variant="secondary">{t('status.starting')}</Badge>
  if (status === 'stopping') return <Badge variant="secondary">{t('status.stopping')}</Badge>
  if (status === 'error') return <Badge variant="destructive">{t('status.error')}</Badge>
  return <Badge variant="outline">{t('status.stopped')}</Badge>
}

export default function Dashboard() {
  const { t } = useTranslation()
  const tunnelStatus = useAppStore((s) => s.tunnelStatus)
  const activeProfileId = useAppStore((s) => s.activeProfileId)
  const profiles = useAppStore((s) => s.profiles)
  const proxyItems = useAppStore((s) => s.proxyItems)
  const config = useAppStore((s) => s.config)
  const startTunnel = useAppStore((s) => s.startTunnel)
  const stopTunnel = useAppStore((s) => s.stopTunnel)
  const checkAllProxyPorts = useAppStore((s) => s.checkAllProxyPorts)
  const checkAllDnsStatus = useAppStore((s) => s.checkAllDnsStatus)
  const addProxyItem = useAppStore((s) => s.addProxyItem)
  const updateProxyItem = useAppStore((s) => s.updateProxyItem)
  const restartTunnel = useAppStore((s) => s.restartTunnel)

  const [editTarget, setEditTarget] = useState<ProxyItem | null>(null)
  const [proxyFormOpen, setProxyFormOpen] = useState(false)
  const [profileDialogOpen, setProfileDialogOpen] = useState(false)

  const activeProfile = profiles.find((p) => p.id === activeProfileId) ?? null

  useEffect(() => {
    if (!activeProfileId) return
    void checkAllProxyPorts()
    void checkAllDnsStatus()
  }, [activeProfileId, proxyItems.length, checkAllProxyPorts, checkAllDnsStatus])

  const handleProxySubmit = async (values: ProxyFormValues) => {
    try {
      if (editTarget) {
        await updateProxyItem(editTarget.id, {
          hostname: values.hostname,
          service: values.service,
          path: values.path,
        })
        toast.success(t('proxy.routeUpdated'))
      } else {
        await addProxyItem({
          hostname: values.hostname,
          service: values.service,
          path: values.path,
        })
        toast.success(t('proxy.routeAdded'))
      }
      if (config?.tunnel) {
        try {
          await tauri.tunnelRouteDns(config.tunnel, values.hostname)
          toast.message(t('proxy.dnsConfigured'), { description: values.hostname })
        } catch (e) {
          toast.error(t('proxy.dnsRouteFailed', { message: String(e) }))
        }
      }
      if (tunnelStatus === 'running') {
        try {
          await restartTunnel()
          toast.message(t('dashboard.tunnelRestarted'))
        } catch (e) {
          toast.error(t('dashboard.restartFailed', { message: String(e) }))
        }
      }
      setProxyFormOpen(false)
      setEditTarget(null)
    } catch (e) {
      toast.error(t('proxy.saveFailed', { message: String(e) }))
    }
  }

  if (!activeProfile) {
    return (
      <main className="p-6">
        <div className="mx-auto flex max-w-3xl flex-col items-start gap-4 rounded-md border bg-card p-8">
          <h2 className="text-xl font-semibold">{t('dashboard.emptyTitle')}</h2>
          <p className="text-sm text-muted-foreground">{t('dashboard.emptyBody')}</p>
          <Button onClick={() => setProfileDialogOpen(true)}>
            <Plus className="size-4" /> {t('dashboard.newProfile')}
          </Button>
        </div>
        <NewProfileDialog open={profileDialogOpen} onOpenChange={setProfileDialogOpen} />
      </main>
    )
  }

  return (
    <main className="space-y-6 p-6">
      <header className="flex flex-wrap items-center justify-between gap-2">
        <div className="space-y-1">
          <h2 className="text-xl font-semibold">{activeProfile.name}</h2>
          <p className="text-xs text-muted-foreground">
            {t('dashboard.tunnelHeader')}{' '}
            <code className="rounded bg-muted px-1">{activeProfile.tunnelName}</code> ·{' '}
            {activeProfile.configPath}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <StatusBadge status={tunnelStatus} />
          {tunnelStatus === 'running' ? (
            <Button
              variant="outline"
              onClick={async () => {
                try {
                  await stopTunnel()
                  toast.success(t('dashboard.tunnelStopped'))
                } catch (e) {
                  toast.error(t('dashboard.stopFailed', { message: String(e) }))
                }
              }}
            >
              <Square className="size-4" /> {t('dashboard.stop')}
            </Button>
          ) : (
            <Button
              onClick={async () => {
                try {
                  await startTunnel()
                  toast.success(t('dashboard.tunnelStarted'))
                } catch (e) {
                  toast.error(t('dashboard.startFailed', { message: String(e) }))
                }
              }}
              disabled={tunnelStatus === 'starting'}
            >
              <Play className="size-4" /> {t('dashboard.start')}
            </Button>
          )}
          <Button
            variant="outline"
            size="icon"
            onClick={async () => {
              await Promise.all([checkAllProxyPorts(), checkAllDnsStatus()])
              toast.message(t('dashboard.refreshed'))
            }}
          >
            <RefreshCw className="size-4" />
          </Button>
        </div>
      </header>

      <Separator />

      <section className="space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold">{t('dashboard.ingressRules')}</h3>
          <div className="flex gap-2">
            <Button size="sm" variant="outline" onClick={() => setProfileDialogOpen(true)}>
              <Plus className="size-4" /> {t('dashboard.newProfile')}
            </Button>
            <Button
              size="sm"
              onClick={() => {
                setEditTarget(null)
                setProxyFormOpen(true)
              }}
            >
              <Plus className="size-4" /> {t('dashboard.addRoute')}
            </Button>
          </div>
        </div>
        <ProxyTable
          onEdit={(item) => {
            setEditTarget(item)
            setProxyFormOpen(true)
          }}
        />
      </section>

      <Separator />

      <LogViewer />

      <ProxyFormDialog
        open={proxyFormOpen}
        title={t(editTarget ? 'proxy.routeEditTitle' : 'proxy.routeAddTitle')}
        initial={
          editTarget
            ? {
                hostname: editTarget.hostname,
                service: editTarget.service,
                path: editTarget.path,
              }
            : undefined
        }
        onOpenChange={(v) => {
          setProxyFormOpen(v)
          if (!v) setEditTarget(null)
        }}
        onSubmit={handleProxySubmit}
      />

      <NewProfileDialog open={profileDialogOpen} onOpenChange={setProfileDialogOpen} />
    </main>
  )
}
