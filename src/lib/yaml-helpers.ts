import yaml from 'js-yaml'
import type { CloudflaredConfig, IngressRule, ProxyItem } from '@/store/app-store'

const LOOPBACK_HOSTS = new Set(['localhost', '127.0.0.1', '0.0.0.0'])

export function isLoopback(host: string): boolean {
  return LOOPBACK_HOSTS.has(host)
}

export function rewriteServiceHost(service: string, replacement: string): string {
  try {
    const url = new URL(service)
    if (LOOPBACK_HOSTS.has(url.hostname)) {
      url.hostname = replacement
      return url.toString().replace(/\/$/, '')
    }
    return service
  } catch {
    return service
  }
}

export function parseServiceUrl(service: string): { host: string; port: number } {
  try {
    const url = new URL(service)
    return {
      host: url.hostname,
      port: Number(url.port) || (url.protocol === 'https:' ? 443 : 80),
    }
  } catch {
    const match = service.match(/^(?:https?:\/\/)?([^:/]+)(?::(\d+))?/)
    if (match) {
      return {
        host: match[1] ?? 'localhost',
        port: match[2] ? Number(match[2]) : 80,
      }
    }
    return { host: 'localhost', port: 80 }
  }
}

const CATCH_ALL_SERVICE = 'http_status:404'

export function ingressToProxyItems(ingress: IngressRule[] | undefined): ProxyItem[] {
  if (!ingress) return []
  return ingress
    .filter((rule) => rule.hostname)
    .map((rule, index) => ({
      id: `ingress-${index}-${rule.hostname}`,
      hostname: rule.hostname ?? '',
      service: rule.service,
      path: rule.path,
      portStatus: 'unknown' as const,
      dnsStatus: 'unknown' as const,
    }))
}

export function proxyItemsToIngress(
  items: ProxyItem[],
  options: { hostRewrite?: string } = {},
): IngressRule[] {
  const rewrite = options.hostRewrite
  const rules: IngressRule[] = items.map((item) => ({
    hostname: item.hostname,
    ...(item.path ? { path: item.path } : {}),
    service: rewrite ? rewriteServiceHost(item.service, rewrite) : item.service,
  }))
  rules.push({ service: CATCH_ALL_SERVICE })
  return rules
}

export function serializeConfig(
  config: CloudflaredConfig,
  items: ProxyItem[],
  options: { hostRewrite?: string } = {},
): string {
  const out: Record<string, unknown> = {
    ...config,
    ingress: proxyItemsToIngress(items, options),
  }
  return yaml.dump(out, { lineWidth: 120, noRefs: true })
}

export function parseConfigYaml(raw: string): CloudflaredConfig | null {
  if (!raw.trim()) return null
  try {
    const parsed = yaml.load(raw)
    if (parsed && typeof parsed === 'object') return parsed as CloudflaredConfig
    return null
  } catch {
    return null
  }
}
