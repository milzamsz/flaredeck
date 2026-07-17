import { useEffect, useState } from 'react'
import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  Clipboard,
  FolderGit2,
  Play,
  RefreshCw,
  RotateCcw,
  ShieldCheck,
  Square,
  TerminalSquare,
  Webhook,
} from 'lucide-react'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useWorkspaceStore } from '@/store/workspace-store'

const stages = ['Validate', 'Start runtime', 'Readiness', 'Tunnel', 'Routes', 'Healthy']

export default function WorkspacesPage() {
  const store = useWorkspaceStore()
  const refresh = store.refresh
  const [path, setPath] = useState('')
  const [reviewOpen, setReviewOpen] = useState(false)

  useEffect(() => {
    void refresh()
  }, [refresh])

  const inspect = async () => {
    try {
      await store.inspect(path)
      setReviewOpen(true)
    } catch (error) {
      toast.error(`Could not inspect workspace: ${String(error)}`)
    }
  }

  return (
    <main className="space-y-6 p-4 sm:p-6">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <h1 className="text-xl font-semibold">Workspaces</h1>
          <p className="text-sm text-muted-foreground">Trusted development runtimes, tunnels, and public routes.</p>
        </div>
        <div className="flex w-full max-w-xl gap-2">
          <Input aria-label="Workspace directory" value={path} onChange={(event) => setPath(event.target.value)} placeholder="Workspace directory" />
          <Button disabled={!path.trim() || store.busy} onClick={() => void inspect()}>Review</Button>
        </div>
      </div>

      <div aria-live="polite" className="sr-only">{store.busy ? 'Workspace operation in progress' : store.error ?? 'Workspace operation complete'}</div>
      {store.error && <p role="alert" className="rounded-md border border-destructive/40 bg-destructive/10 p-3 text-sm">{store.error}</p>}

      <div className="grid gap-4 xl:grid-cols-[minmax(16rem,22rem)_1fr]">
        <section aria-label="Registered workspaces" className="space-y-2">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-medium">Registered</h2>
            <Button size="icon-sm" variant="ghost" aria-label="Refresh workspaces" onClick={() => void store.refresh()}><RefreshCw className="size-4" /></Button>
          </div>
          {store.workspaces.length === 0 ? (
            <Card><CardContent className="py-8 text-center text-sm text-muted-foreground">Review a repository with a <code>.flaredeck/project.yaml</code> manifest.</CardContent></Card>
          ) : store.workspaces.map((workspace) => (
            <button key={workspace.workspaceId} type="button" onClick={() => void store.select(workspace)} className="w-full rounded-lg border p-3 text-left transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
              <div className="flex items-start justify-between gap-2">
                <span className="font-medium">{workspace.projectName}</span>
                <StatusBadge label={workspace.validationState === 'invalid' ? 'Invalid manifest' : workspace.activeSession?.state ?? workspace.approvalState} />
              </div>
              <p className="mt-1 truncate text-xs text-muted-foreground">{workspace.root}</p>
              <p className="mt-2 text-xs">Profile: {workspace.profile || 'Unavailable'}</p>
            </button>
          ))}
        </section>

        {store.selected ? (
          <WorkspaceDetail onReview={() => setReviewOpen(true)} />
        ) : (
          <Card className="min-h-72"><CardContent className="flex h-full min-h-72 items-center justify-center text-sm text-muted-foreground">Select a workspace to inspect its runtime and session.</CardContent></Card>
        )}
      </div>

      <TrustReviewDialog open={reviewOpen} onOpenChange={setReviewOpen} />
    </main>
  )
}

function WorkspaceDetail({ onReview }: { onReview: () => void }) {
  const { selected, session, logs, audit, temporaryRoutes, webhookEvents, busy, pipelineStage, pipelineFailed, start, stop, refreshDetails, replayWebhook, reconcileTemporaryRoutes } = useWorkspaceStore()
  const [replay, setReplay] = useState<{ routeId: string; eventId: string; label: string; origin: string } | null>(null)
  if (!selected) return null
  const active = session && !['stopped', 'failed'].includes(session.state)

  return (
    <section className="min-w-0 space-y-4" aria-label={`${selected.projectName} workspace detail`}>
      <Card>
        <CardHeader className="gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <CardTitle className="flex items-center gap-2"><FolderGit2 className="size-5" />{selected.projectName}</CardTitle>
            <CardDescription className="mt-1 break-all">{selected.root}</CardDescription>
          </div>
          <div className="flex flex-wrap gap-2">
            <StatusBadge label={session?.state ?? selected.approvalState} />
            {!selected.trusted && <Button variant="outline" onClick={onReview}><ShieldCheck className="size-4" />Review trust</Button>}
            {selected.trusted && !active && <Button disabled={busy} onClick={() => void start().catch((error) => toast.error(`Could not start session: ${String(error)}`))}><Play className="size-4" />Start</Button>}
            {active && <Button variant="destructive" disabled={busy} onClick={() => void stop().catch((error) => toast.error(`Could not stop session: ${String(error)}`))}><Square className="size-4" />Stop</Button>}
            <Button size="icon-sm" variant="ghost" aria-label="Refresh session details" onClick={() => void refreshDetails()}><RefreshCw className="size-4" /></Button>
          </div>
        </CardHeader>
        <CardContent>
          <SessionPipeline stage={selected.trusted ? pipelineStage : 'validate'} failed={pipelineFailed || selected.approvalState !== 'trusted'} />
        </CardContent>
      </Card>

      <Tabs defaultValue="overview">
        <TabsList className="max-w-full overflow-x-auto">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="logs">Logs</TabsTrigger>
          <TabsTrigger value="health">Health</TabsTrigger>
          <TabsTrigger value="routes">Routes</TabsTrigger>
          <TabsTrigger value="webhooks">Webhooks</TabsTrigger>
          <TabsTrigger value="audit">Audit</TabsTrigger>
        </TabsList>
        <TabsContent value="overview" className="grid gap-4 md:grid-cols-2">
          <InfoCard title="Runtime" icon={<TerminalSquare className="size-4" />} rows={[
            ['Executable', selected.executable],
            ['Arguments', selected.args.join(' ') || 'None'],
            ['Working directory', selected.workingDirectory],
            ['Ownership', session?.runtimeOwnership ?? 'Not running'],
          ]} />
          <InfoCard title="Readiness and tunnel" icon={<Activity className="size-4" />} rows={[
            ['Readiness', selected.readiness],
            ['Profile', selected.profile],
            ['Tunnel ownership', session?.tunnelOwnership ?? 'Not running'],
            ['Session', session ? session.id.slice(0, 16) : 'None'],
          ]} />
          <Card className="md:col-span-2"><CardHeader><CardTitle className="text-base">Public URLs</CardTitle></CardHeader><CardContent className="space-y-2">
            {session?.publicUrls.length ? session.publicUrls.map((url) => <div key={url} className="flex items-center justify-between gap-2 rounded-md border p-2"><a className="min-w-0 truncate text-sm underline" href={url} target="_blank" rel="noreferrer">{url}</a><Button size="icon-sm" variant="ghost" aria-label={`Copy ${url}`} onClick={() => void navigator.clipboard.writeText(url).then(() => toast.success('Public URL copied'))}><Clipboard className="size-4" /></Button></div>) : <p className="text-sm text-muted-foreground">No active public URL.</p>}
          </CardContent></Card>
        </TabsContent>
        <TabsContent value="logs"><Card><CardHeader><CardTitle className="text-base">Combined logs</CardTitle><CardDescription>Runtime output is bounded and redacted. Unredacted output is never available.</CardDescription></CardHeader><CardContent><ScrollArea tabIndex={0} className="h-72 rounded-md border bg-zinc-950 p-3 font-mono text-xs text-zinc-100">{logs.length ? logs.map((entry, index) => <div key={`${index}-${entry.stream}`}><span className="text-zinc-400">[{entry.stream}]</span> {entry.line}</div>) : <p className="text-zinc-400">No log entries.</p>}</ScrollArea></CardContent></Card></TabsContent>
        <TabsContent value="health"><InfoCard title="Health observations" icon={<Activity className="size-4" />} rows={[
          ['Runtime', session ? (session.runtimeOwnership === 'session' ? 'Running · session owned' : 'Ready · external') : 'Stopped'],
          ['Readiness', session?.state === 'healthy' ? 'Healthy' : 'Not healthy'],
          ['Tunnel', session ? session.tunnelOwnership : 'Not observed'],
          ['Routes', session?.publicUrls.length ? 'Configured' : 'Not active'],
        ]} /></TabsContent>
        <TabsContent value="routes"><Card><CardHeader><CardTitle className="text-base">Exposure routes</CardTitle></CardHeader><CardContent className="space-y-2">{selected.routes.map((route) => <div key={`${route.hostname}${route.path ?? ''}`} className="rounded-md border p-3 text-sm"><div className="flex justify-between gap-2"><strong>{route.hostname}</strong><Badge variant="outline">{route.mode}</Badge></div><p className="mt-1 break-all text-xs text-muted-foreground">{route.path ?? '/'} → {route.origin}</p></div>)}</CardContent></Card></TabsContent>
        <TabsContent value="webhooks"><Card><CardHeader className="gap-2 sm:flex-row sm:items-start sm:justify-between"><div><CardTitle className="flex items-center gap-2 text-base"><Webhook className="size-4" />Temporary webhook captures</CardTitle><CardDescription>At most 100 redacted events per temporary route. Secret headers and fields cannot be recovered.</CardDescription></div><Button size="sm" variant="outline" disabled={busy} onClick={() => void reconcileTemporaryRoutes().then(() => toast.success('Temporary route cleanup reconciled')).catch((error) => toast.error(`Could not reconcile cleanup: ${String(error)}`))}><RefreshCw className="size-3" />Retry cleanup</Button></CardHeader><CardContent className="space-y-4">
          {temporaryRoutes.map((route) => <section key={route.id} className="space-y-2 rounded-md border p-3"><div className="flex flex-wrap items-center justify-between gap-2"><div><strong className="text-sm">{route.hostname}{route.path ?? '/'}</strong><p className="text-xs text-muted-foreground">Expires {new Date(route.expiresAt).toLocaleString()}</p></div><StatusBadge label={route.state} /></div>
            {webhookEvents.filter((event) => event.routeId === route.id).map((event) => <div key={event.id} className="rounded-md border bg-muted/20 p-3 text-xs"><div className="flex flex-wrap items-center justify-between gap-2"><div><Badge variant="outline">{event.method}</Badge> <code className="break-all">{event.path}</code></div><Button size="sm" variant="outline" disabled={route.state !== 'active'} onClick={() => setReplay({ routeId: route.id, eventId: event.id, label: `${event.method} ${event.path}`, origin: route.origin })}><RotateCcw className="size-3" />Replay</Button></div><p className="mt-2 text-muted-foreground">{new Date(event.timestamp).toLocaleString()} · response {event.responseStatus ?? 'unknown'} · {event.bodyState} · redaction v{event.redactionVersion}</p>{event.body && <pre className="mt-2 max-h-36 overflow-auto whitespace-pre-wrap break-all rounded bg-zinc-950 p-2 text-zinc-100">{event.body}</pre>}</div>)}
            {!webhookEvents.some((event) => event.routeId === route.id) && <p className="text-sm text-muted-foreground">No captured events.</p>}
          </section>)}
          {!temporaryRoutes.length && <p className="text-sm text-muted-foreground">No temporary routes in this session.</p>}
        </CardContent></Card></TabsContent>
        <TabsContent value="audit"><Card><CardHeader><CardTitle className="text-base">Recent audit events</CardTitle></CardHeader><CardContent className="space-y-2">{audit.length ? audit.map((event) => <div key={`${event.timestamp}-${event.correlationId}`} className="rounded-md border p-3 text-xs"><div className="flex justify-between gap-2"><strong>{event.operation}</strong><StatusBadge label={event.result} /></div><p className="mt-1 text-muted-foreground">{new Date(event.timestamp).toLocaleString()} · {event.correlationId}</p></div>) : <p className="text-sm text-muted-foreground">No audit events.</p>}</CardContent></Card></TabsContent>
      </Tabs>
      <Dialog open={replay !== null} onOpenChange={(open) => { if (!open) setReplay(null) }}>
        <DialogContent><DialogHeader><DialogTitle>Replay this redacted webhook?</DialogTitle><DialogDescription>This may repeat side effects in your local application. The stored redacted method, path, headers, and body will be sent only to the original loopback origin; removed secrets are not replayed.</DialogDescription></DialogHeader><div className="space-y-2 rounded-md border p-3 text-sm"><code className="block break-all">{replay?.label}</code><code className="block break-all text-muted-foreground">Target: {replay?.origin}</code></div><DialogFooter><Button variant="outline" onClick={() => setReplay(null)}>Cancel</Button><Button disabled={busy || !replay} onClick={() => { if (!replay) return; void replayWebhook(replay.routeId, replay.eventId).then((status) => { toast.success(`Webhook replay returned ${status}`); setReplay(null) }).catch((error) => toast.error(`Could not replay webhook: ${String(error)}`)) }}><RotateCcw className="size-4" />Replay redacted event</Button></DialogFooter></DialogContent>
      </Dialog>
    </section>
  )
}

function TrustReviewDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  const { selected, busy, approve } = useWorkspaceStore()
  if (!selected) return null
  const accept = async () => {
    try {
      await approve()
      onOpenChange(false)
      toast.success('Workspace configuration trusted until it changes')
    } catch (error) {
      toast.error(`Could not approve workspace: ${String(error)}`)
    }
  }
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] max-w-3xl overflow-y-auto">
        <DialogHeader><DialogTitle>Trust this workspace configuration?</DialogTitle><DialogDescription>Review every behavior FlareDeck may execute. A behavior change invalidates this approval.</DialogDescription></DialogHeader>
        {selected.approvalState === 'changed' && <div className="flex gap-2 rounded-md border border-amber-500/50 bg-amber-500/10 p-3 text-sm"><AlertTriangle className="mt-0.5 size-4 shrink-0" />The manifest differs from the previously approved fingerprint. Review the current behavior below.</div>}
        <div className="grid gap-4 text-sm sm:grid-cols-2">
          <ReviewSection title="Runtime"><code className="break-all">{selected.executable}</code>{selected.args.map((arg, index) => <code key={`${index}-${arg}`} className="block break-all text-xs">arg {index + 1}: {arg}</code>)}<p>Directory: <code>{selected.workingDirectory}</code></p></ReviewSection>
          <ReviewSection title="Readiness and profile"><p>{selected.readiness}</p><p>Profile: <strong>{selected.profile}</strong></p></ReviewSection>
          <ReviewSection title="Environment"><p>Names: {selected.environmentNames.join(', ') || 'None'}</p>{selected.environmentValues.map((entry) => <p key={entry.name}><code>{entry.name}={entry.value}</code> (committed literal)</p>)}</ReviewSection>
          <ReviewSection title="Lifecycle">{selected.lifecycle.map((line) => <p key={line}>{line}</p>)}</ReviewSection>
          <ReviewSection title="Exposure routes">{selected.routes.map((route) => <p key={`${route.hostname}${route.path ?? ''}`} className="break-all"><strong>{route.hostname}</strong> {route.path ?? '/'} → {route.origin} ({route.mode})</p>)}</ReviewSection>
          <ReviewSection title="Capabilities">{selected.capabilities.map((capability) => <p key={capability}>• {capability}</p>)}</ReviewSection>
        </div>
        <p className="break-all text-xs text-muted-foreground">Fingerprint: {selected.fingerprint}</p>
        <DialogFooter><Button variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button><Button disabled={busy} onClick={() => void accept()}><ShieldCheck className="size-4" />Trust until manifest changes</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function ReviewSection({ title, children }: { title: string; children: React.ReactNode }) {
  return <section className="space-y-1 rounded-md border p-3"><h3 className="font-medium">{title}</h3>{children}</section>
}

function SessionPipeline({ stage, failed }: { stage: string; failed: boolean }) {
  const active = Math.max(0, ['validate', 'runtime', 'readiness', 'tunnel', 'routes', 'healthy'].indexOf(stage))
  return <ol className="grid gap-2 text-xs sm:grid-cols-3 lg:grid-cols-6" aria-label="Session progress">{stages.map((label, index) => <li key={label} aria-current={index === active ? 'step' : undefined} className="flex items-center gap-2 rounded-md border p-2">{index < active || stage === 'healthy' ? <CheckCircle2 className="size-4 text-green-600" /> : index === active && failed ? <AlertTriangle className="size-4 text-destructive" /> : <span className="size-2 rounded-full bg-muted-foreground" />}<span>{label}{index === active && failed ? ' · blocked' : ''}</span></li>)}</ol>
}

function StatusBadge({ label }: { label: string }) {
  const text = label.replaceAll('_', ' ')
  const variant = ['failed', 'invalid', 'cleanup_incomplete'].some((value) => label.includes(value)) ? 'destructive' : label === 'healthy' || label === 'trusted' || label === 'success' ? 'default' : 'secondary'
  return <Badge variant={variant} className="capitalize">{text}</Badge>
}

function InfoCard({ title, icon, rows }: { title: string; icon: React.ReactNode; rows: [string, string][] }) {
  return <Card><CardHeader><CardTitle className="flex items-center gap-2 text-base">{icon}{title}</CardTitle></CardHeader><CardContent><dl className="space-y-2 text-sm">{rows.map(([label, value]) => <div key={label} className="grid gap-1 sm:grid-cols-[9rem_1fr]"><dt className="text-muted-foreground">{label}</dt><dd className="break-all">{value}</dd></div>)}</dl></CardContent></Card>
}
