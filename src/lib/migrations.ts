const LEGACY_KEYS = ['flaredeck.cloudflare.connection'] as const

export function runLegacyMigrations(): void {
  if (typeof window === 'undefined') return
  for (const key of LEGACY_KEYS) {
    if (window.localStorage.getItem(key) !== null) {
      window.localStorage.removeItem(key)
    }
  }
}
