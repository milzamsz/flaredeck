import { useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { Copy } from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useAppStore } from '@/store/app-store'
import { cn } from '@/lib/utils'

export function LogViewer() {
  const { t } = useTranslation()
  const logs = useAppStore((s) => s.logs)
  const clearLogs = useAppStore((s) => s.clearLogs)
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'auto' })
  }, [logs])

  const copyLogs = async () => {
    const text = logs
      .map(
        (entry) =>
          `${new Date(entry.ts).toLocaleTimeString()} [${entry.source}] ${entry.message}`,
      )
      .join('\n')
    try {
      await navigator.clipboard.writeText(text)
      toast.success(t('logs.copied'))
    } catch (e) {
      toast.error(t('logs.copyFailed', { message: String(e) }))
    }
  }

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold">{t('logs.title')}</h3>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => void copyLogs()}
            disabled={logs.length === 0}
            aria-label={t('logs.copy')}
          >
            <Copy className="size-4" /> {t('logs.copy')}
          </Button>
          <Button variant="ghost" size="sm" onClick={clearLogs} disabled={logs.length === 0}>
            {t('logs.clear')}
          </Button>
        </div>
      </div>
      <ScrollArea className="h-48 rounded-md border bg-black/90 p-3 font-mono text-xs text-zinc-200">
        {logs.length === 0 ? (
          <span className="text-zinc-500">{t('logs.empty')}</span>
        ) : (
          logs.map((entry) => (
            <div
              key={entry.id}
              className={cn(
                'border-b border-zinc-800 py-0.5 last:border-0',
                entry.level === 'error' && 'text-red-400',
                entry.level === 'warn' && 'text-yellow-400',
              )}
            >
              <span className="text-zinc-500">
                {new Date(entry.ts).toLocaleTimeString()}
              </span>{' '}
              <span className="text-zinc-400">[{entry.source}]</span>{' '}
              {entry.message}
            </div>
          ))
        )}
        <div ref={bottomRef} />
      </ScrollArea>
    </div>
  )
}
