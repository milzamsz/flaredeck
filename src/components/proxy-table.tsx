import { useTranslation } from 'react-i18next'
import { ExternalLink, Pencil, Trash2 } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { tauri } from '@/lib/tauriApi'
import { useAppStore, type ProxyItem } from '@/store/app-store'

function StatusPill({ kind, value }: { kind: 'port' | 'dns'; value?: string }) {
  const { t } = useTranslation()
  if (!value || value === 'unknown') {
    return <Badge variant="outline">{t('proxy.unknown')}</Badge>
  }
  const positive = kind === 'port' ? value === 'open' : value === 'resolved'
  return (
    <Badge variant={positive ? 'default' : 'destructive'}>{t(`proxy.${value}`)}</Badge>
  )
}

type Props = {
  onEdit: (item: ProxyItem) => void
}

export function ProxyTable({ onEdit }: Props) {
  const { t } = useTranslation()
  const items = useAppStore((s) => s.proxyItems)
  const removeProxyItem = useAppStore((s) => s.removeProxyItem)

  if (items.length === 0) {
    return (
      <div className="rounded-md border p-8 text-center text-sm text-muted-foreground">
        {t('proxy.noRoutes')}
      </div>
    )
  }

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>{t('proxy.hostname')}</TableHead>
          <TableHead>{t('proxy.service')}</TableHead>
          <TableHead>{t('proxy.origin')}</TableHead>
          <TableHead>{t('proxy.dns')}</TableHead>
          <TableHead className="text-right">{t('proxy.actions')}</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {items.map((item) => (
          <TableRow key={item.id}>
            <TableCell className="font-medium">
              <a
                className="inline-flex items-center gap-1 hover:underline"
                onClick={(e) => {
                  e.preventDefault()
                  void tauri.shellOpenExternal(`https://${item.hostname}`)
                }}
                href={`https://${item.hostname}`}
              >
                {item.hostname}
                <ExternalLink className="size-3" />
              </a>
            </TableCell>
            <TableCell className="font-mono text-xs">{item.service}</TableCell>
            <TableCell>
              <StatusPill kind="port" value={item.portStatus} />
            </TableCell>
            <TableCell>
              <StatusPill kind="dns" value={item.dnsStatus} />
            </TableCell>
            <TableCell className="text-right">
              <Button variant="ghost" size="icon-sm" onClick={() => onEdit(item)}>
                <Pencil className="size-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon-sm"
                onClick={() => void removeProxyItem(item.id)}
              >
                <Trash2 className="size-4" />
              </Button>
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}
