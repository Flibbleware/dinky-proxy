export const parseDomains = (value: string): string[] =>
  value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)

export const serializeDomains = (domains: string[]): string => domains.join('\n')

const hasScheme = (value: string): boolean => /^[a-zA-Z][a-zA-Z\d+\-.]*:\/\//.test(value)

// Entries like "*.internal.company" aren't valid URLs, so a parse failure means
// "not a URL" rather than "invalid domain" — fall back to the raw text and let
// the existing bypass-domain validation reject anything actually malformed.
export const normalizeDomain = (input: string): string => {
  const trimmed = input.trim()
  if (!trimmed) return trimmed

  try {
    const hostname = new URL(hasScheme(trimmed) ? trimmed : `https://${trimmed}`).hostname
    return hostname.startsWith('www.') ? hostname.slice(4) : hostname
  } catch {
    return trimmed
  }
}
