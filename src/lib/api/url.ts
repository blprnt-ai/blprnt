const DEFAULT_API_BASE_URL = 'http://localhost:9171/api/v1'

function normalizeBaseUrl(url: URL) {
  return url.toString().replace(/\/$/, '')
}

export function resolveApiBaseUrl(rawBaseUrl: string = DEFAULT_API_BASE_URL) {
  try {
    return normalizeBaseUrl(new URL(rawBaseUrl))
  } catch {
    const baseOrigin = typeof window === 'undefined' ? DEFAULT_API_BASE_URL : window.location.origin
    return normalizeBaseUrl(new URL(rawBaseUrl, baseOrigin))
  }
}

export function buildWebSocketUrl(path: string, searchParams?: Record<string, string>) {
  const url = new URL(resolveApiBaseUrl(import.meta.env?.VITE_API_URL ?? DEFAULT_API_BASE_URL))
  url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:'
  url.pathname = `${url.pathname.replace(/\/$/, '')}${path}`

  if (searchParams) {
    Object.entries(searchParams).forEach(([key, value]) => {
      url.searchParams.set(key, value)
    })
  }

  return url.toString()
}

export { DEFAULT_API_BASE_URL }
