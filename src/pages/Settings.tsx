import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useTheme } from 'next-themes'
import { ExternalLink, LogIn, LogOut, Trash2 } from 'lucide-react'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useAppStore } from '@/store/app-store'
import { tauri } from '@/lib/tauriApi'

export default function SettingsPage() {
  const { t } = useTranslation()
  const { theme, setTheme: setNextTheme } = useTheme()
  const isAuthenticated = useAppStore((s) => s.isAuthenticated)
  const certPath = useAppStore((s) => s.certPath)
  const cloudflaredInstalled = useAppStore((s) => s.cloudflaredInstalled)
  const cloudflaredPath = useAppStore((s) => s.cloudflaredPath)
  const cloudflaredVersion = useAppStore((s) => s.cloudflaredVersion)
  const wslHostIp = useAppStore((s) => s.wslHostIp)
  const profiles = useAppStore((s) => s.profiles)
  const activeProfileId = useAppStore((s) => s.activeProfileId)
  const setStoreTheme = useAppStore((s) => s.setTheme)
  const refreshAuth = useAppStore((s) => s.refreshAuth)
  const refreshCloudflared = useAppStore((s) => s.refreshCloudflared)
  const refreshWslHostIp = useAppStore((s) => s.refreshWslHostIp)
  const updateActiveProfile = useAppStore((s) => s.updateActiveProfile)
  const deleteProfile = useAppStore((s) => s.deleteProfile)
  const minimizeToTray = useAppStore((s) => s.minimizeToTray)
  const setMinimizeToTray = useAppStore((s) => s.setMinimizeToTray)

  const activeProfile = profiles.find((p) => p.id === activeProfileId) ?? null

  const [pollingLogin, setPollingLogin] = useState(false)

  useEffect(() => {
    if (!pollingLogin) return
    const start = Date.now()
    const id = setInterval(async () => {
      await refreshAuth()
      const { isAuthenticated } = useAppStore.getState()
      if (isAuthenticated) {
        toast.success(t('settings.signedInToast'))
        clearInterval(id)
        setPollingLogin(false)
      } else if (Date.now() - start > 5 * 60 * 1000) {
        clearInterval(id)
        setPollingLogin(false)
      }
    }, 2000)
    return () => clearInterval(id)
  }, [pollingLogin, refreshAuth, t])

  return (
    <main className="space-y-6 p-6">
      <h2 className="text-xl font-semibold">{t('nav.settings')}</h2>

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
                onClick={() =>
                  void tauri.shellOpenExternal(
                    'https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/',
                  )
                }
              >
                <ExternalLink className="size-4" /> {t('settings.install')}
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

      <Card>
        <CardHeader>
          <CardTitle>{t('settings.accountTitle')}</CardTitle>
          <CardDescription>{t('settings.accountDescription')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <div className="flex items-center justify-between">
            <span>{t('settings.status')}</span>
            <Badge variant={isAuthenticated ? 'default' : 'outline'}>
              {t(isAuthenticated ? 'status.signedIn' : 'status.notSignedIn')}
            </Badge>
          </div>
          {certPath && (
            <div className="flex items-center justify-between gap-2">
              <span className="text-muted-foreground">{t('settings.certificate')}</span>
              <code className="rounded bg-muted px-1 text-xs">{certPath}</code>
            </div>
          )}
          <div className="flex flex-wrap gap-2 pt-2">
            {isAuthenticated ? (
              <Button
                variant="outline"
                size="sm"
                onClick={async () => {
                  try {
                    await tauri.authLogout()
                    await refreshAuth()
                    toast.success(t('settings.signedOut'))
                  } catch (e) {
                    toast.error(t('settings.signOutFailed', { message: String(e) }))
                  }
                }}
              >
                <LogOut className="size-4" /> {t('settings.signOut')}
              </Button>
            ) : (
              <Button
                size="sm"
                disabled={!cloudflaredInstalled || pollingLogin}
                onClick={async () => {
                  try {
                    await tauri.authLogin()
                    setPollingLogin(true)
                    toast.message(t('settings.browserOpened'), {
                      description: t('settings.browserHint'),
                    })
                  } catch (e) {
                    toast.error(t('settings.loginFailed', { message: String(e) }))
                  }
                }}
              >
                <LogIn className="size-4" />{' '}
                {pollingLogin ? t('settings.waitingBrowser') : t('settings.signIn')}
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

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
