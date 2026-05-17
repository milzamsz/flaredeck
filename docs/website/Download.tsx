/**
 * FlareDeck download page — drop this into the flaredeck-web repo
 * (e.g. as `app/download/page.tsx` in Next, `src/pages/download.astro`
 * for Astro with React, or `routes/download.tsx` in Remix).
 *
 * It detects the visitor's OS via the user-agent / userAgentData and
 * shows the right "Download" button. All links point at stable
 * GitHub Releases URLs that always resolve to the latest tag, so the
 * page does NOT need to be redeployed for new releases — the GitHub
 * Action that ships the binaries is the only thing that has to run.
 *
 * Required env:
 *   NEXT_PUBLIC_GH_REPO (or your framework's equivalent):
 *     "milzamsz/flaredeck"
 *
 * Asset filenames are produced by tauri-action. They follow this
 * pattern; if you change the productName or version scheme, update
 * the FILENAME map below.
 */

import { useEffect, useState } from 'react'

const GH_REPO = 'milzamsz/flaredeck'

type OS = 'windows' | 'macos' | 'linux' | 'unknown'

type Asset = {
  label: string
  filename: string
  size?: string
  hint?: string
}

const ASSETS: Record<Exclude<OS, 'unknown'>, Asset[]> = {
  windows: [
    {
      label: 'Windows installer (.exe)',
      filename: 'FlareDeck_{version}_x64-setup.exe',
      hint: 'Recommended. Handles WebView2 bootstrap.',
    },
    {
      label: 'Windows portable (.exe)',
      filename: 'flaredeck.exe',
      hint: 'Single-file binary. Requires WebView2 already installed.',
    },
  ],
  macos: [
    {
      label: 'macOS Universal (.dmg)',
      filename: 'FlareDeck_{version}_universal.dmg',
      hint: 'Apple Silicon + Intel.',
    },
  ],
  linux: [
    {
      label: 'Linux AppImage',
      filename: 'FlareDeck_{version}_amd64.AppImage',
      hint: 'Runs on most distros. Mark executable, then double-click.',
    },
    {
      label: 'Debian / Ubuntu (.deb)',
      filename: 'FlareDeck_{version}_amd64.deb',
    },
  ],
}

const releaseUrl = (filename: string) =>
  `https://github.com/${GH_REPO}/releases/latest/download/${filename}`

const allReleasesUrl = `https://github.com/${GH_REPO}/releases`

function detectOS(): OS {
  if (typeof navigator === 'undefined') return 'unknown'
  // navigator.userAgentData is more reliable than parsing UA but not
  // available everywhere. Fall through if missing or unhelpful.
  const uaData = (navigator as any).userAgentData
  if (uaData?.platform) {
    const p = String(uaData.platform).toLowerCase()
    if (p.includes('win')) return 'windows'
    if (p.includes('mac')) return 'macos'
    if (p.includes('linux')) return 'linux'
  }
  const ua = navigator.userAgent.toLowerCase()
  if (ua.includes('windows')) return 'windows'
  if (ua.includes('mac os')) return 'macos'
  if (ua.includes('linux')) return 'linux'
  return 'unknown'
}

export default function DownloadPage() {
  const [os, setOs] = useState<OS>('unknown')
  // Fetch the latest version string from latest.json so the page can
  // show what they're downloading. Falls back gracefully.
  const [version, setVersion] = useState<string | null>(null)

  useEffect(() => {
    setOs(detectOS())
    fetch('/latest.json')
      .then((r) => (r.ok ? r.json() : null))
      .then((m) => {
        if (m?.version) setVersion(m.version.replace(/^v/, ''))
      })
      .catch(() => {})
  }, [])

  const fillVersion = (filename: string) =>
    filename.replace('{version}', version ?? '')

  const primary = os !== 'unknown' ? ASSETS[os][0] : null

  return (
    <main className="mx-auto max-w-3xl px-6 py-16">
      <h1 className="text-4xl font-bold tracking-tight">Download FlareDeck</h1>
      <p className="mt-3 text-lg text-muted-foreground">
        Desktop control panel for Cloudflare Tunnel.{' '}
        {version && <span className="text-foreground">v{version}</span>}
      </p>

      {/* Primary CTA — the OS we detected */}
      {primary && (
        <section className="mt-10 rounded-lg border bg-card p-6">
          <h2 className="text-sm font-medium uppercase text-muted-foreground">
            For your system
          </h2>
          <div className="mt-3 flex flex-wrap items-center gap-4">
            <a
              href={version ? releaseUrl(fillVersion(primary.filename)) : '#'}
              className="rounded-md bg-primary px-6 py-3 text-lg font-medium text-primary-foreground hover:opacity-90"
            >
              {primary.label}
            </a>
            {primary.hint && (
              <p className="text-sm text-muted-foreground">{primary.hint}</p>
            )}
          </div>
        </section>
      )}

      {/* All platforms */}
      <section className="mt-10">
        <h2 className="text-sm font-medium uppercase text-muted-foreground">
          All platforms
        </h2>
        <div className="mt-3 grid gap-4 sm:grid-cols-2">
          {(['windows', 'macos', 'linux'] as const).map((key) => (
            <div key={key} className="rounded-lg border p-4">
              <h3 className="font-medium capitalize">{key}</h3>
              <ul className="mt-2 space-y-1 text-sm">
                {ASSETS[key].map((a) => (
                  <li key={a.filename}>
                    <a
                      href={version ? releaseUrl(fillVersion(a.filename)) : '#'}
                      className="text-primary hover:underline"
                    >
                      {a.label}
                    </a>
                    {a.hint && (
                      <span className="block text-xs text-muted-foreground">
                        {a.hint}
                      </span>
                    )}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </section>

      <p className="mt-10 text-sm text-muted-foreground">
        Looking for an older version? Browse{' '}
        <a className="text-primary hover:underline" href={allReleasesUrl}>
          all releases on GitHub
        </a>
        .
      </p>
    </main>
  )
}
