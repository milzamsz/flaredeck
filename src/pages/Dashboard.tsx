import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  ExternalLink,
  Eye,
  EyeOff,
  Play,
  Plus,
  RefreshCw,
  Square,
} from 'lucide-react'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { LogViewer } from '@/components/log-viewer'
import { ProxyFormDialog, type ProxyFormValues } from '@/components/proxy-form-dialog'
import { ProxyTable } from '@/components/proxy-table'
import { useAppStore, type ProxyItem, type TunnelLifecycle } from '@/store/app-store'
import { CF_TOKEN_CREATE_URL, routeDnsForProfile, tauri } from '@/lib/tauriApi'

const REUSE_TOKEN_NEW = '__new__'

function NewProfileDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (v: boolean) => void
}) {
  const { t } = useTranslation()
  const createProfileSimple = useAppStore((s) => s.createProfileSimple)
  const wslHostIp = useAppStore((s) => s.wslHostIp)
  const profiles = useAppStore((s) => s.profiles)
  const tokenSourceProfiles = profiles.filter((p) => p.hasApiToken)

  const [name, setName] = useState('')
  const [token, setToken] = useState('')
  const [showToken, setShowToken] = useState(false)
  const [reuseFrom, setReuseFrom] = useState<string>(REUSE_TOKEN_NEW)
  const [domain, setDomain] = useState('')
  const [wslHost, setWslHost] = useState(true)
  const [submitting, setSubmitting] = useState(false)

  const usingReuse = reuseFrom !== REUSE_TOKEN_NEW
  const canSubmit =
    name.trim() &&
    domain.trim() &&
    (usingReuse || token.trim()) &&
    !submitting

  const reset = () => {
    setName('')
    setToken('')
    setShowToken(false)
    setReuseFrom(REUSE_TOKEN_NEW)
    setDomain('')
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(v) => {
        if (!v) reset()
        onOpenChange(v)
      }}
    >
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
              autoFocus
            />
          </div>

          {tokenSourceProfiles.length > 0 && (
            <div className="space-y-2">
              <Label htmlFor="profile-reuse-token">
                {t('profile.tokenSource')}
              </Label>
              <Select value={reuseFrom} onValueChange={setReuseFrom}>
                <SelectTrigger id="profile-reuse-token">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value={REUSE_TOKEN_NEW}>
                    {t('profile.tokenSourceNew')}
                  </SelectItem>
                  {tokenSourceProfiles.map((p) => (
                    <SelectItem key={p.id} value={p.id}>
                      {t('profile.tokenSourceReuse', { name: p.name })}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}

          {!usingReuse && (
            <div className="space-y-1">
              <Label htmlFor="profile-token">{t('profile.token')}</Label>
              <div className="flex gap-2">
                <Input
                  id="profile-token"
                  type={showToken ? 'text' : 'password'}
                  autoComplete="off"
                  spellCheck={false}
                  value={token}
                  onChange={(e) => setToken(e.target.value)}
                  placeholder={t('profile.tokenPlaceholder')}
                />
                <Button
                  type="button"
                  variant="outline"
                  size="icon-sm"
                  aria-label={t(
                    showToken
                      ? 'settings.credsTokenHide'
                      : 'settings.credsTokenShow',
                  )}
                  onClick={() => setShowToken((v) => !v)}
                >
                  {showToken ? (
                    <EyeOff className="size-4" />
                  ) : (
                    <Eye className="size-4" />
                  )}
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">
                {t('profile.tokenHelp')}{' '}
                <button
                  type="button"
                  className="inline-flex items-center gap-1 text-primary hover:underline"
                  onClick={() => void tauri.shellOpenExternal(CF_TOKEN_CREATE_URL)}
                >
                  {t('profile.tokenCreateLink')}
                  <ExternalLink className="size-3" />
                </button>
              </p>
              <div className="rounded-md border border-border/60 bg-muted/40 p-2 text-xs text-muted-foreground">
                <div className="mb-1 font-medium text-foreground">
                  {t('profile.tokenScopesHeading')}
                </div>
                <ul className="ml-1 list-disc pl-4 space-y-0.5">
                  <li>{t('profile.tokenScopeTunnel')}</li>
                  <li>{t('profile.tokenScopeZoneRead')}</li>
                  <li>{t('profile.tokenScopeDnsEdit')}</li>
                </ul>
                <p className="mt-1">{t('profile.tokenScopesNote')}</p>
              </div>
            </div>
          )}

          <div className="space-y-1">
            <Label htmlFor="profile-domain">{t('profile.domain')}</Label>
            <Input
              id="profile-domain"
              value={domain}
              onChange={(e) => setDomain(e.target.value)}
              placeholder={t('profile.domainPlaceholder')}
              autoComplete="off"
              spellCheck={false}
            />
            <p className="text-xs text-muted-foreground">
              {t('profile.domainHelp')}
            </p>
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
            disabled={!canSubmit}
            onClick={async () => {
              setSubmitting(true)
              try {
                await createProfileSimple(
                  name.trim(),
                  usingReuse ? '' : token.trim(),
                  usingReuse ? reuseFrom : null,
                  domain.trim(),
                  wslHost,
                )
                reset()
                onOpenChange(false)
                toast.success(t('profile.created'))
              } catch (e) {
                toast.error(t('profile.createFailed', { message: String(e) }))
              } finally {
                setSubmitting(false)
              }
            }}
          >
            {submitting ? t('profile.creating') : t('profile.create')}
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
  const profileDialogOpen = useAppStore((s) => s.newProfileDialogOpen)
  const setProfileDialogOpen = useAppStore((s) => s.setNewProfileDialogOpen)

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
      if (config?.tunnel && activeProfile) {
        try {
          const { via } = await routeDnsForProfile(
            activeProfile,
            values.hostname,
            config.tunnel,
          )
          toast.message(t('proxy.dnsConfigured'), {
            description:
              via === 'api'
                ? t('proxy.dnsConfiguredViaApi', { hostname: values.hostname })
                : values.hostname,
          })
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
