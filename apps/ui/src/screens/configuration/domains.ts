/**
 * The bypass list is stored in the form as a single newline-delimited string so it
 * round-trips cleanly through react-hook-form and the zod schema. These helpers are the
 * single source of truth for converting between that string and a clean list of domains.
 */
export const parseDomains = (value: string): string[] =>
  value
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)

export const serializeDomains = (domains: string[]): string => domains.join('\n')
