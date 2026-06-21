export const parseDomains = (value: string): string[] =>
  value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)

export const serializeDomains = (domains: string[]): string => domains.join('\n')
