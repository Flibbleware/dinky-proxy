export const parseDomains = (value: string): string[] =>
  value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)

export const serializeDomains = (domains: string[]): string => domains.join('\n')

const hasScheme = (value: string): boolean => /^[a-zA-Z][a-zA-Z\d+\-.]*:\/\//.test(value)

const stripWww = (host: string): string => (host.startsWith('www.') ? host.slice(4) : host)

// Only parse entries that already look like a URL (have a scheme). Bare entries
// may be bypass patterns like "*.internal.company" rather than real hostnames,
// and WHATWG URL parsing would mangle characters such as "*" that aren't valid
// in a real host (e.g. percent-encoding it) instead of leaving them alone.
export const normalizeDomain = (input: string): string => {
  const trimmed = input.trim()
  if (!trimmed || !hasScheme(trimmed)) return stripWww(trimmed)

  try {
    return stripWww(new URL(trimmed).hostname)
  } catch {
    return trimmed
  }
}
