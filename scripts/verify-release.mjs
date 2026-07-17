import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = dirname(dirname(fileURLToPath(import.meta.url)))
const json = (path) => JSON.parse(readFileSync(join(root, path), 'utf8'))
const pkg = json('package.json')
const lock = json('package-lock.json')
const tauri = json('src-tauri/tauri.conf.json')
const releaseConfig = json('src-tauri/tauri.release.conf.json')
const compatibility = json('docs/specs/release-compatibility.json')
const cargo = readFileSync(join(root, 'src-tauri', 'Cargo.toml'), 'utf8')
const cargoVersion = cargo.match(/^version = "([^"]+)"/m)?.[1]
const cargoLock = readFileSync(join(root, 'src-tauri', 'Cargo.lock'), 'utf8')
const cargoLockVersion = cargoLock.match(/\[\[package\]\]\nname = "flaredeck"\nversion = "([^"]+)"/)?.[1]
const versions = [
  pkg.version,
  lock.version,
  lock.packages?.['']?.version,
  tauri.version,
  cargoVersion,
  cargoLockVersion,
  compatibility.appVersion,
]
if (new Set(versions).size !== 1) throw new Error(`release versions differ: ${versions.join(', ')}`)
if (!pkg.scripts['desktop:build']?.includes('prepare-sidecars.mjs') || !pkg.scripts['desktop:build']?.includes('tauri.release.conf.json')) {
  throw new Error('desktop release build does not require prepared sidecars')
}

const expectedBins = compatibility.companions.map((name) => `binaries/${name}`).sort()
const actualBins = [...(releaseConfig.bundle.externalBin || [])].sort()
if (JSON.stringify(expectedBins) !== JSON.stringify(actualBins)) throw new Error('packaged companions differ from compatibility contract')
if (!tauri.bundle.createUpdaterArtifacts) throw new Error('updater artifacts are disabled')
if (tauri.identifier !== compatibility.updater.identifier) throw new Error('updater application identifier changed')
const keyHash = createHash('sha256').update(tauri.plugins.updater.pubkey).digest('hex')
if (keyHash !== compatibility.updater.publicKeySha256) throw new Error('updater public key changed')
if (JSON.stringify(tauri.plugins.updater.endpoints) !== JSON.stringify(compatibility.updater.endpoints)) throw new Error('updater endpoints changed')

const downloads = [
  readFileSync(join(root, 'docs', 'website', 'download.html'), 'utf8'),
  readFileSync(join(root, 'docs', 'website', 'Download.tsx'), 'utf8'),
]
for (const artifact of compatibility.stableArtifacts) {
  if (downloads.some((source) => !source.includes(artifact))) throw new Error(`stable download artifact missing: ${artifact}`)
}
const workflow = readFileSync(join(root, '.github', 'workflows', 'release.yml'), 'utf8')
for (const required of ['prepare-sidecars.mjs', 'smoke-sidecars.mjs', 'tauri.release.conf.json', 'SHA256SUMS.txt', 'releaseDraft: true']) {
  if (!workflow.includes(required)) throw new Error(`release workflow is missing ${required}`)
}
console.log(`Release contract valid for ${pkg.version}`)
