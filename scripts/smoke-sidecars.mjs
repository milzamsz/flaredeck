import { spawnSync } from 'node:child_process'
import { mkdtempSync, readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { tmpdir } from 'node:os'

const root = dirname(dirname(fileURLToPath(import.meta.url)))
const target = process.argv[2]
if (!target) throw new Error('usage: node scripts/smoke-sidecars.mjs <target-triple>')
const extension = target.includes('windows') ? '.exe' : ''
const version = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8')).version
const isolatedConfig = mkdtempSync(join(tmpdir(), 'flaredeck-release-smoke-'))
const binary = (name) => join(root, 'src-tauri', 'binaries', `${name}-${target}${extension}`)
const run = (name, args, input) => {
  const result = spawnSync(binary(name), args, {
    input,
    encoding: 'utf8',
    timeout: 30_000,
    env: { ...process.env, XDG_CONFIG_HOME: isolatedConfig, APPDATA: isolatedConfig, LOCALAPPDATA: isolatedConfig },
  })
  if (result.status !== 0) throw new Error(`${name} smoke failed: ${result.stderr}`)
  return result.stdout.trim()
}

if (!run('flaredeck-cli', ['version']).includes(version)) throw new Error('CLI version mismatch')
const doctor = JSON.parse(run('flaredeck-cli', ['--output=json', 'doctor']))
if (!doctor.ok || doctor.data?.version !== version || doctor.meta?.schemaVersion !== '1') throw new Error('CLI doctor mismatch')
if (run('flaredeck-mcp', ['--version']) !== version) throw new Error('MCP version mismatch')
if (run('flaredeck-webhook-proxy', ['--version']) !== version) throw new Error('proxy version mismatch')

const protocol = run('flaredeck-mcp', [], [
  JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'initialize', params: { protocolVersion: '2025-11-25', capabilities: {}, clientInfo: { name: 'release-smoke', version: '1' } } }),
  JSON.stringify({ jsonrpc: '2.0', id: 2, method: 'tools/list' }),
  '',
].join('\n')).split('\n').map(JSON.parse)
if (protocol[0]?.result?.serverInfo?.version !== version || protocol[1]?.result?.tools?.length !== 11) {
  throw new Error('MCP protocol/version smoke failed')
}
console.log(`Sidecar smoke passed for ${target} at ${version}`)
