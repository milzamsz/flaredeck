import { execFileSync } from 'node:child_process'
import { chmodSync, copyFileSync, mkdirSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const root = dirname(dirname(fileURLToPath(import.meta.url)))
const manifest = join(root, 'src-tauri', 'Cargo.toml')
const output = join(root, 'src-tauri', 'binaries')
const companions = ['flaredeck-cli', 'flaredeck-mcp', 'flaredeck-webhook-proxy']
const extension = process.platform === 'win32' ? '.exe' : ''
const requested = process.argv[2] || execFileSync('rustc', ['--print', 'host-tuple'], { encoding: 'utf8' }).trim()

mkdirSync(output, { recursive: true })

function build(target) {
  const args = ['build', '--release', '--locked', '--manifest-path', manifest, '--target', target]
  for (const companion of companions) args.push('--bin', companion)
  execFileSync('cargo', args, { stdio: 'inherit' })
}

if (requested === 'universal-apple-darwin') {
  const targets = ['aarch64-apple-darwin', 'x86_64-apple-darwin']
  targets.forEach(build)
  for (const companion of companions) {
    for (const target of targets) {
      const source = join(root, 'src-tauri', 'target', target, 'release', companion)
      const destination = join(output, `${companion}-${target}`)
      copyFileSync(source, destination)
      chmodSync(destination, 0o755)
    }
    const destination = join(output, `${companion}-${requested}`)
    execFileSync('lipo', [
      '-create',
      join(root, 'src-tauri', 'target', targets[0], 'release', companion),
      join(root, 'src-tauri', 'target', targets[1], 'release', companion),
      '-output', destination,
    ])
    chmodSync(destination, 0o755)

    const releaseDestination = join(root, 'src-tauri', 'target', requested, 'release', companion)
    copyFileSync(destination, releaseDestination)
    chmodSync(releaseDestination, 0o755)
  }
} else {
  build(requested)
  for (const companion of companions) {
    const source = join(root, 'src-tauri', 'target', requested, 'release', `${companion}${extension}`)
    const destination = join(output, `${companion}-${requested}${extension}`)
    copyFileSync(source, destination)
    if (process.platform !== 'win32') chmodSync(destination, 0o755)
  }
}

console.log(`Prepared ${companions.length} sidecars for ${requested}`)
