import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import CodeMirror from '@uiw/react-codemirror'
import { yaml } from '@codemirror/lang-yaml'
import { oneDark } from '@codemirror/theme-one-dark'
import { useTheme } from 'next-themes'
import { toast } from 'sonner'
import { Folder, Save } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { useAppStore } from '@/store/app-store'
import { tauri } from '@/lib/tauriApi'

function Editor({
  initial,
  onSave,
}: {
  initial: string
  onSave: (raw: string) => Promise<void>
}) {
  const { t } = useTranslation()
  const { resolvedTheme } = useTheme()
  const [draft, setDraft] = useState(initial)
  const [saving, setSaving] = useState(false)
  const dirty = draft !== initial

  return (
    <>
      <div className="flex gap-2">
        <Button
          disabled={!dirty || saving}
          onClick={async () => {
            setSaving(true)
            try {
              await onSave(draft)
              toast.success(t('config.saved'))
            } catch (e) {
              toast.error(t('config.saveFailed', { message: String(e) }))
            } finally {
              setSaving(false)
            }
          }}
        >
          <Save className="size-4" /> {t('config.save')}
        </Button>
      </div>

      <div className="flex-1 overflow-hidden rounded-md border">
        <CodeMirror
          value={draft}
          onChange={setDraft}
          theme={resolvedTheme === 'dark' ? oneDark : 'light'}
          extensions={[yaml()]}
          height="100%"
          style={{ height: '100%' }}
        />
      </div>
    </>
  )
}

export default function ConfigPage() {
  const { t } = useTranslation()
  const configRaw = useAppStore((s) => s.configRaw)
  const configPath = useAppStore((s) => s.configPath)
  const activeProfileId = useAppStore((s) => s.activeProfileId)
  const saveConfig = useAppStore((s) => s.saveConfig)

  if (!activeProfileId) {
    return (
      <main className="p-6">
        <div className="mx-auto max-w-3xl rounded-md border bg-card p-8 text-sm text-muted-foreground">
          {t('config.selectProfile')}
        </div>
      </main>
    )
  }

  return (
    <main className="flex h-full flex-col gap-3 p-6">
      <header className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h2 className="text-xl font-semibold">{t('config.title')}</h2>
          <p className="text-xs text-muted-foreground">
            <code className="rounded bg-muted px-1">{configPath ?? t('config.notLoaded')}</code>
          </p>
        </div>
        <Button
          variant="outline"
          disabled={!configPath}
          onClick={() => configPath && void tauri.shellOpenPath(configPath)}
        >
          <Folder className="size-4" /> {t('config.reveal')}
        </Button>
      </header>

      <Editor key={configRaw} initial={configRaw} onSave={saveConfig} />
    </main>
  )
}
