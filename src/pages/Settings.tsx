import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useTheme } from 'next-themes'
import {
  Download,
  ExternalLink,
  Eye,
  EyeOff,
  RefreshCw,
  Trash2,
} from 'lucide-react'
import { toast } from 'sonner'

import { useUpdater } from '@/lib/updater'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useAppStore } from '@/store/app-store'
import {
  CF_TOKEN_CREATE_URL,
  CF_TOKEN_DOCS_URL,
  normaliseDomainInput,
  tauri,
} from '@/lib/tauriApi'

export default function SettingsPage() {
  const { t } = useTranslation()
  const { theme, setTheme: setNextTheme } = useTheme()
  const cloudflaredInstalled = useAppStore((s) => s.cloudflaredInstalled)
  const cloudflaredPath = useAppStore((s) => s.cloudflaredPath)
  const cloudflaredVersion = useAppStore((s) => s.cloudflaredVersion)
  const wslHostIp = useAppStore((s) => s.wslHostIp)
  const profiles = useAppStore((s) => s.profiles)
  const activeProfileId = useAppStore((s) => s.activeProfileId)
  const setStoreTheme = useAppStore((s) => s.setTheme)
  const refreshCloudflared = useAppStore((s) => s.refreshCloudflared)
  const [installingCloudflared, setInstallingCloudflared] = useState(false)
  const refreshWslHostIp = useAppStore((s) => s.refreshWslHostIp)
  const updateActiveProfile = useAppStore((s) => s.updateActiveProfile)
  const deleteProfile = useAppStore((s) => s.deleteProfile)
  const minimizeToTray = useAppStore((s) => s.minimizeToTray)
  const setMinimizeToTray = useAppStore((s) => s.setMinimizeToTray)

  const activeProfile = profiles.find((p) => p.id === activeProfileId) ?? null

  return (
    <main className="space-y-6 p-6">
      <h2 className="text-xl font-semibold">{t('nav.settings')}</h2>

      <UpdateCard />

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.cloudflaredTitle')}</CardTitle>
          <CardDescription>{t('settings.cloudflaredDescription')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <div className="flex items-center justify-between">
            <span>{t('settings.status')}</span>
            <Badge variant={cloudflaredInstalled ? 'default' : 'destructive'}>
              {t(cloudflaredInstalled ? 'status.found' : 'status.missing')}
            </Badge>
          </div>
          {cloudflaredPath && (
            <div className="flex items-center justify-between gap-2">
              <span className="text-muted-foreground">{t('settings.path')}</span>
              <code className="rounded bg-muted px-1 text-xs">{cloudflaredPath}</code>
            </div>
          )}
          {cloudflaredVersion && (
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground">{t('settings.version')}</span>
              <code className="rounded bg-muted px-1 text-xs">{cloudflaredVersion}</code>
            </div>
          )}
          <div className="flex flex-wrap gap-2 pt-2">
            <Button variant="outline" size="sm" onClick={() => void refreshCloudflared()}>
              {t('settings.recheck')}
            </Button>
            {!cloudflaredInstalled && (
              <Button
                size="sm"
                disabled={installingCloudflared}
                onClick={async () => {
                  setInstallingCloudflared(true)
                  try {
                    await tauri.cloudflaredInstall()
                    await refreshCloudflared()
                    toast.success(t('settings.installSuccess'))
                  } catch (e) {
                    toast.error(t('settings.installFailed', { message: String(e) }))
                  } finally {
                    setInstallingCloudflared(false)
                  }
                }}
              >
                <Download className="size-4" /> {t('settings.install')}
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.wslTitle')}</CardTitle>
          <CardDescription>{t('settings.wslDescription')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <div className="flex items-center justify-between">
            <span>{t('settings.wslIp')}</span>
            <code className="rounded bg-muted px-1 text-xs">
              {wslHostIp ?? t('settings.wslNotDetected')}
            </code>
          </div>
          {activeProfile && (
            <div className="flex items-start justify-between gap-2 pt-2">
              <div className="flex flex-col">
                <Label
                  htmlFor="settings-wsl-toggle"
                  className="cursor-pointer leading-tight"
                >
                  {t('settings.activeProfileWslHost', {
                    name: activeProfile.name,
                  })}
                </Label>
                <span className="text-xs text-muted-foreground">
                  {t('profile.wslHostHelp')}
                </span>
              </div>
              <input
                id="settings-wsl-toggle"
                type="checkbox"
                className="mt-1"
                checked={activeProfile.wslHost}
                onChange={(e) =>
                  void updateActiveProfile({ wslHost: e.target.checked })
                }
              />
            </div>
          )}
          <div className="pt-2">
            <Button variant="outline" size="sm" onClick={() => void refreshWslHostIp()}>
              {t('settings.wslRedetect')}
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.profilesTitle')}</CardTitle>
          <CardDescription>{t('settings.profilesDescription')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          {profiles.length === 0 ? (
            <p className="text-muted-foreground">{t('settings.profilesEmpty')}</p>
          ) : (
            <ul className="divide-y rounded-md border">
              {profiles.map((p) => (
                <li
                  key={p.id}
                  className="flex items-center justify-between gap-3 px-3 py-2"
                >
                  <div className="min-w-0">
                    <div className="font-medium">{p.name}</div>
                    <div className="truncate text-xs text-muted-foreground">
                      {p.tunnelName} · {p.configPath}
                    </div>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    aria-label={t('settings.profileDelete')}
                    onClick={async () => {
                      const ok = window.confirm(
                        t('settings.profileDeleteConfirm', { name: p.name }),
                      )
                      if (!ok) return
                      try {
                        await deleteProfile(p.id)
                        toast.success(t('settings.profileDeleted'))
                      } catch (e) {
                        toast.error(
                          t('settings.profileDeleteFailed', {
                            message: String(e),
                          }),
                        )
                      }
                    }}
                  >
                    <Trash2 className="size-4" />
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </CardContent>
      </Card>

      {activeProfile && (
        <CredentialsCard key={activeProfile.id} profile={activeProfile} />
      )}


      <Card>
        <CardHeader>
          <CardTitle>{t('settings.windowTitle')}</CardTitle>
          <CardDescription>{t('settings.windowDescription')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 text-sm">
          <div className="flex items-start justify-between gap-3">
            <div className="flex flex-col">
              <Label
                htmlFor="settings-minimize-to-tray"
                className="cursor-pointer leading-tight"
              >
                {t('settings.minimizeToTray')}
              </Label>
              <span className="text-xs text-muted-foreground">
                {t('settings.minimizeToTrayHelp')}
              </span>
            </div>
            <input
              id="settings-minimize-to-tray"
              type="checkbox"
              className="mt-1"
              checked={minimizeToTray}
              onChange={(e) => void setMinimizeToTray(e.target.checked)}
            />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.appearanceTitle')}</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="max-w-sm space-y-2">
            <Label>{t('settings.theme')}</Label>
            <Select
              value={theme ?? 'system'}
              onValueChange={(v) => {
                setNextTheme(v)
                setStoreTheme(v as 'light' | 'dark' | 'system')
              }}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="system">{t('settings.themeSystem')}</SelectItem>
                <SelectItem value="light">{t('settings.themeLight')}</SelectItem>
                <SelectItem value="dark">{t('settings.themeDark')}</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>
    </main>
  )
}

type CredentialsCardProps = {
  profile: NonNullable<ReturnType<typeof useAppStore.getState>['profiles']>[number]
}

function CredentialsCard({ profile }: CredentialsCardProps) {
  const { t } = useTranslation()
  const updateActiveProfile = useAppStore((s) => s.updateActiveProfile)
  const refreshProfiles = useAppStore((s) => s.refreshProfiles)

  const [tokenDraft, setTokenDraft] = useState('')
  const [showToken, setShowToken] = useState(false)
  const [domainDraft, setDomainDraft] = useState(profile.zoneName ?? '')
  const [saving, setSaving] = useState(false)

  // The card shows a single status line that summarises everything the
  // user cares about. The two inputs (token, domain) are the only
  // editable fields; Account ID and Zone ID happen invisibly on Save.
  const connected = profile.hasApiToken && !!profile.zoneId
  const hasPendingChanges =
    tokenDraft.trim().length > 0 || domainDraft.trim() !== (profile.zoneName ?? '')

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          {t('settings.credsTitle', { name: profile.name })}
          <Badge variant={connected ? 'default' : 'outline'}>
            {connected
              ? t('settings.credsConnected', {
                  zone: profile.zoneName ?? profile.zoneId,
                })
              : t('settings.credsNotConnected')}
          </Badge>
        </CardTitle>
        <CardDescription>{t('settings.credsDescription')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3 text-sm">
        <div className="space-y-1">
          <Label htmlFor="cf-token">
            {profile.hasApiToken
              ? t('settings.credsTokenRotate')
              : t('settings.credsTokenAdd')}
          </Label>
          <div className="flex gap-2">
            <Input
              id="cf-token"
              type={showToken ? 'text' : 'password'}
              autoComplete="off"
              spellCheck={false}
              placeholder={
                profile.hasApiToken
                  ? t('settings.credsTokenPlaceholderRotate')
                  : t('settings.credsTokenPlaceholder')
              }
              value={tokenDraft}
              onChange={(e) => setTokenDraft(e.target.value)}
            />
            <Button
              type="button"
              variant="outline"
              size="icon-sm"
              aria-label={t(
                showToken ? 'settings.credsTokenHide' : 'settings.credsTokenShow',
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
            {t('settings.credsTokenHelp')}{' '}
            <button
              type="button"
              className="inline-flex items-center gap-1 text-primary hover:underline"
              onClick={() => void tauri.shellOpenExternal(CF_TOKEN_CREATE_URL)}
            >
              {t('settings.credsTokenCreateLink')}
              <ExternalLink className="size-3" />
            </button>
            {' · '}
            <button
              type="button"
              className="inline-flex items-center gap-1 text-primary hover:underline"
              onClick={() => void tauri.shellOpenExternal(CF_TOKEN_DOCS_URL)}
            >
              {t('settings.credsTokenDocsLink')}
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

        <div className="space-y-1">
          <Label htmlFor="cf-domain">{t('settings.credsDomain')}</Label>
          <Input
            id="cf-domain"
            autoComplete="off"
            spellCheck={false}
            placeholder={t('settings.credsDomainPlaceholder')}
            value={domainDraft}
            onChange={(e) => setDomainDraft(e.target.value)}
          />
          <p className="text-xs text-muted-foreground">
            {t('settings.credsDomainHelp')}
          </p>
        </div>

        <div className="flex flex-wrap gap-2 pt-1">
          <Button
            size="sm"
            disabled={saving || !hasPendingChanges}
            onClick={async () => {
              setSaving(true)
              try {
                // 1) Save token first so the lookup that follows has
                //    something to authenticate with.
                if (tokenDraft.trim()) {
                  await tauri.profilesSetToken(profile.id, tokenDraft.trim())
                }
                // 2) If the domain changed, resolve and persist the new
                //    account/zone IDs.
                const cleaned = normaliseDomainInput(domainDraft)
                const domainChanged = (cleaned ?? '') !== (profile.zoneName ?? '')
                if (domainChanged && cleaned) {
                  const r = await tauri.cfLookupZone(profile.id, cleaned)
                  await updateActiveProfile({
                    accountId: r.accountId,
                    zoneId: r.zoneId,
                    zoneName: r.zoneName,
                  })
                  toast.success(
                    t('settings.credsLookupOk', {
                      zone: r.zoneName,
                      account: r.accountName ?? r.accountId,
                    }),
                  )
                } else if (tokenDraft.trim()) {
                  toast.success(t('settings.credsSaved'))
                }
                setTokenDraft('')
                setShowToken(false)
                await refreshProfiles()
              } catch (e) {
                toast.error(
                  t('settings.credsSaveFailed', { message: String(e) }),
                )
              } finally {
                setSaving(false)
              }
            }}
          >
            {saving ? t('settings.credsSaving') : t('settings.credsSave')}
          </Button>
          {profile.hasApiToken && (
            <Button
              size="sm"
              variant="ghost"
              onClick={async () => {
                const ok = window.confirm(t('settings.credsDisconnectConfirm'))
                if (!ok) return
                try {
                  await tauri.profilesClearToken(profile.id)
                  await updateActiveProfile({
                    accountId: '',
                    zoneId: '',
                    zoneName: '',
                  })
                  await refreshProfiles()
                  setDomainDraft('')
                  toast.success(t('settings.credsDisconnected'))
                } catch (e) {
                  toast.error(
                    t('settings.credsDisconnectFailed', {
                      message: String(e),
                    }),
                  )
                }
              }}
            >
              {t('settings.credsDisconnect')}
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

function UpdateCard() {
  const { t } = useTranslation()
  const { state, checkOnce, downloadAndInstall, restartNow } = useUpdater()

  const fmtBytes = (n: number) => {
    if (n < 1024) return `${n} B`
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`
    return `${(n / 1024 / 1024).toFixed(1)} MB`
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          {t('settings.updateTitle')}
          {state.kind === 'available' && (
            <Badge className="bg-green-600 text-white">
              {t('settings.updateAvailable')}
            </Badge>
          )}
          {state.kind === 'upToDate' && (
            <Badge variant="outline">{t('settings.updateUpToDate')}</Badge>
          )}
        </CardTitle>
        <CardDescription>{t('settings.updateDescription')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3 text-sm">
        {state.kind === 'checking' && (
          <p className="text-muted-foreground">{t('settings.updateChecking')}</p>
        )}
        {state.kind === 'error' && (
          <p className="text-destructive">{state.message}</p>
        )}
        {state.kind === 'available' && (
          <div className="space-y-2">
            <p>
              <span className="font-medium">
                {t('settings.updateNewVersion', {
                  version: state.update.version,
                })}
              </span>
              {state.update.date && (
                <span className="text-muted-foreground">
                  {' '}
                  · {state.update.date}
                </span>
              )}
            </p>
            {state.update.body && (
              <pre className="max-h-40 overflow-auto whitespace-pre-wrap rounded border bg-muted/40 p-2 text-xs">
                {state.update.body}
              </pre>
            )}
          </div>
        )}
        {state.kind === 'downloading' && (
          <div className="space-y-1">
            <p className="text-muted-foreground">
              {t('settings.updateDownloading', {
                downloaded: fmtBytes(state.downloaded),
                total: state.total ? fmtBytes(state.total) : '?',
              })}
            </p>
            {state.total !== null && (
              <div className="h-1.5 w-full overflow-hidden rounded bg-muted">
                <div
                  className="h-full bg-primary transition-all"
                  style={{
                    width: `${Math.min(100, (state.downloaded / state.total) * 100)}%`,
                  }}
                />
              </div>
            )}
          </div>
        )}
        {state.kind === 'ready' && (
          <p className="text-muted-foreground">{t('settings.updateReady')}</p>
        )}

        <div className="flex flex-wrap gap-2 pt-1">
          {state.kind === 'available' && (
            <Button
              size="sm"
              onClick={() => {
                void downloadAndInstall()
              }}
            >
              <Download className="size-4" />
              {t('settings.updateInstall')}
            </Button>
          )}
          {state.kind === 'ready' && (
            <Button
              size="sm"
              onClick={() => {
                void restartNow()
              }}
            >
              {t('settings.updateRestart')}
            </Button>
          )}
          <Button
            size="sm"
            variant="outline"
            disabled={state.kind === 'checking' || state.kind === 'downloading'}
            onClick={() => {
              void checkOnce()
            }}
          >
            <RefreshCw className="size-4" />
            {t('settings.updateRecheck')}
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
