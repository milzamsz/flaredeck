import { useTranslation } from 'react-i18next'
import { NavLink } from 'react-router-dom'
import {
  CheckCircle2,
  Cog,
  FileText,
  KeyRound,
  LayoutDashboard,
  User,
  Wifi,
  WifiOff,
  XCircle,
} from 'lucide-react'

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar'
import { Badge } from '@/components/ui/badge'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { useAppStore } from '@/store/app-store'
import { cn } from '@/lib/utils'

type Item = {
  to: string
  label: string
  Icon: typeof LayoutDashboard
}

export function AppSidebar() {
  const { t } = useTranslation()
  const tunnelStatus = useAppStore((s) => s.tunnelStatus)
  const cloudflaredInstalled = useAppStore((s) => s.cloudflaredInstalled)
  const isAuthenticated = useAppStore((s) => s.isAuthenticated)
  const profiles = useAppStore((s) => s.profiles)
  const activeProfileId = useAppStore((s) => s.activeProfileId)
  const setActiveProfile = useAppStore((s) => s.setActiveProfile)

  const items: Item[] = [
    { to: '/', label: t('nav.dashboard'), Icon: LayoutDashboard },
    { to: '/config', label: t('nav.config'), Icon: FileText },
    { to: '/settings', label: t('nav.settings'), Icon: Cog },
  ]

  const statusLabel = t(
    tunnelStatus === 'running'
      ? 'status.running'
      : tunnelStatus === 'starting'
        ? 'status.starting'
        : tunnelStatus === 'stopping'
          ? 'status.stopping'
          : tunnelStatus === 'error'
            ? 'status.error'
            : 'status.stopped',
  )

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <div className="flex items-center gap-2 px-2 py-1">
          <img
            src="/favicon.svg"
            alt={t('app.name')}
            className="size-6 shrink-0 rounded"
          />
          <span className="text-sm font-semibold group-data-[collapsible=icon]:hidden">
            {t('app.name')}
          </span>
        </div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>{t('nav.navigation')}</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map(({ to, label, Icon }) => (
                <SidebarMenuItem key={to}>
                  <NavLink to={to} end={to === '/'}>
                    {({ isActive }) => (
                      <SidebarMenuButton isActive={isActive} tooltip={label}>
                        <Icon className="size-4" />
                        <span>{label}</span>
                      </SidebarMenuButton>
                    )}
                  </NavLink>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        {profiles.length > 0 && (
          <SidebarGroup>
            <SidebarGroupLabel>{t('nav.profiles')}</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {profiles.map((p) => (
                  <SidebarMenuItem key={p.id}>
                    <SidebarMenuButton
                      isActive={p.id === activeProfileId}
                      onClick={() => void setActiveProfile(p.id)}
                      tooltip={p.name}
                    >
                      <User className="size-4" />
                      <span className="truncate">{p.name}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        )}
      </SidebarContent>

      <SidebarFooter>
        {/* Expanded layout: full rows with labels + badges. */}
        <div className="flex flex-col gap-2 px-2 py-1 group-data-[collapsible=icon]:hidden">
          <div className="flex items-center justify-between gap-2 text-xs">
            <span className="text-muted-foreground">{t('status.tunnel')}</span>
            <Badge
              className={cn(
                'gap-1',
                tunnelStatus === 'running'
                  ? 'bg-green-600 text-white hover:bg-green-600'
                  : tunnelStatus === 'starting' || tunnelStatus === 'stopping'
                    ? 'bg-yellow-500 text-white hover:bg-yellow-500'
                    : tunnelStatus === 'error'
                      ? 'bg-red-600 text-white hover:bg-red-600'
                      : 'bg-zinc-500 text-white hover:bg-zinc-500',
              )}
            >
              {tunnelStatus === 'running' ? (
                <Wifi className="size-3" />
              ) : (
                <WifiOff className="size-3" />
              )}
              {statusLabel}
            </Badge>
          </div>
          <div className="flex items-center justify-between gap-2 text-xs">
            <span className="text-muted-foreground">{t('status.cloudflared')}</span>
            <Badge
              className={cn(
                'gap-1',
                cloudflaredInstalled
                  ? 'bg-green-600 text-white hover:bg-green-600'
                  : 'bg-red-600 text-white hover:bg-red-600',
              )}
            >
              {cloudflaredInstalled ? (
                <CheckCircle2 className="size-3" />
              ) : (
                <XCircle className="size-3" />
              )}
              {t(cloudflaredInstalled ? 'status.found' : 'status.missing')}
            </Badge>
          </div>
          <div className="flex items-center justify-between gap-2 text-xs">
            <span className="text-muted-foreground">{t('status.auth')}</span>
            <Badge
              className={cn(
                'gap-1',
                isAuthenticated
                  ? 'bg-green-600 text-white hover:bg-green-600'
                  : 'bg-zinc-500 text-white hover:bg-zinc-500',
              )}
            >
              <KeyRound className="size-3" />
              {t(isAuthenticated ? 'status.signedIn' : 'status.notSignedIn')}
            </Badge>
          </div>
        </div>

        {/* Collapsed (icon rail) layout: stacked icon dots with tooltips. */}
        <div className="hidden flex-col items-center gap-2 py-2 group-data-[collapsible=icon]:flex">
          <TooltipProvider delayDuration={150}>
            <Tooltip>
              <TooltipTrigger asChild>
                <div
                  className={cn(
                    'flex size-7 items-center justify-center rounded-md text-white',
                    tunnelStatus === 'running'
                      ? 'bg-green-600'
                      : tunnelStatus === 'starting' || tunnelStatus === 'stopping'
                        ? 'bg-yellow-500'
                        : tunnelStatus === 'error'
                          ? 'bg-red-600'
                          : 'bg-zinc-500',
                  )}
                >
                  {tunnelStatus === 'running' ? (
                    <Wifi className="size-4" />
                  ) : (
                    <WifiOff className="size-4" />
                  )}
                </div>
              </TooltipTrigger>
              <TooltipContent side="right">
                {t('status.tunnel')}: {statusLabel}
              </TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <div
                  className={cn(
                    'flex size-7 items-center justify-center rounded-md text-white',
                    cloudflaredInstalled ? 'bg-green-600' : 'bg-red-600',
                  )}
                >
                  {cloudflaredInstalled ? (
                    <CheckCircle2 className="size-4" />
                  ) : (
                    <XCircle className="size-4" />
                  )}
                </div>
              </TooltipTrigger>
              <TooltipContent side="right">
                {t('status.cloudflared')}:{' '}
                {t(cloudflaredInstalled ? 'status.found' : 'status.missing')}
              </TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger asChild>
                <div
                  className={cn(
                    'flex size-7 items-center justify-center rounded-md text-white',
                    isAuthenticated ? 'bg-green-600' : 'bg-zinc-500',
                  )}
                >
                  <KeyRound className="size-4" />
                </div>
              </TooltipTrigger>
              <TooltipContent side="right">
                {t('status.auth')}:{' '}
                {t(isAuthenticated ? 'status.signedIn' : 'status.notSignedIn')}
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      </SidebarFooter>
    </Sidebar>
  )
}
