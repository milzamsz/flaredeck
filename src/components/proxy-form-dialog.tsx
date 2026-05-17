import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'

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

const HOSTNAME_RE = /^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)*$/i

const schema = z.object({
  hostname: z
    .string()
    .min(1, 'proxy.hostnameRequired')
    .regex(HOSTNAME_RE, 'proxy.hostnameInvalid'),
  service: z
    .string()
    .min(1, 'proxy.serviceRequired')
    .regex(/^https?:\/\//, 'proxy.serviceScheme'),
  path: z.string().optional(),
})

export type ProxyFormValues = z.infer<typeof schema>

type Props = {
  open: boolean
  initial?: Partial<ProxyFormValues>
  title: string
  onSubmit: (values: ProxyFormValues) => void
  onOpenChange: (open: boolean) => void
}

export function ProxyFormDialog({ open, initial, title, onSubmit, onOpenChange }: Props) {
  const { t } = useTranslation()
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<ProxyFormValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      hostname: initial?.hostname ?? '',
      service: initial?.service ?? 'http://localhost:8000',
      path: initial?.path ?? '',
    },
  })

  useEffect(() => {
    if (open) {
      reset({
        hostname: initial?.hostname ?? '',
        service: initial?.service ?? 'http://localhost:8000',
        path: initial?.path ?? '',
      })
    }
  }, [open, initial, reset])

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{t('proxy.routeDescription')}</DialogDescription>
        </DialogHeader>

        <form
          className="space-y-4"
          onSubmit={handleSubmit((values) => {
            onSubmit({
              ...values,
              path: values.path?.trim() ? values.path.trim() : undefined,
            })
          })}
        >
          <div className="space-y-2">
            <Label htmlFor="hostname">{t('proxy.hostnameLabel')}</Label>
            <Input id="hostname" placeholder="app.example.com" {...register('hostname')} />
            {errors.hostname && (
              <p className="text-xs text-destructive">{t(errors.hostname.message ?? '')}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="service">{t('proxy.serviceLabel')}</Label>
            <Input id="service" placeholder="http://localhost:8000" {...register('service')} />
            {errors.service && (
              <p className="text-xs text-destructive">{t(errors.service.message ?? '')}</p>
            )}
          </div>

          <details
            className="rounded-md border border-border/60 bg-muted/30 px-3 py-2 text-sm"
            open={Boolean(initial?.path)}
          >
            <summary className="cursor-pointer select-none text-xs font-medium text-muted-foreground">
              {t('proxy.advanced')}
            </summary>
            <div className="mt-2 space-y-2">
              <Label htmlFor="path">{t('proxy.pathOptional')}</Label>
              <Input id="path" placeholder="/api" {...register('path')} />
            </div>
          </details>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              {t('proxy.cancel')}
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {t('proxy.save')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
