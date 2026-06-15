# TypeScript guidance

## Formatting

- After editing any `.ts` or `.tsx` file, `prettier` runs automatically via a PostToolUse hook in `.claude/settings.json`. No manual formatting step needed.

Applies to `.ts` / `.tsx` source files in this app.

## Type casting

- Never use double casts (e.g. `as unknown as SomeType`). They bypass the type checker entirely and hide real type mismatches. If a value genuinely needs reshaping, fix the types at the source, narrow with a type guard, or validate at the boundary instead.

## Robustness

- Watch for brittle code and prefer constraints the compiler can enforce. For example, a form field `name` typed as a bare `string` lets typos and stale references slip through silently — type it against the actual shape of the form (a keyof, a union of allowed names, etc.) so the compiler catches mismatches. More generally, when a value has a known, finite set of valid options, encode that in the types rather than relying on free-form strings.

## React effects

- Always question whether a `useEffect` is actually needed. If logic runs in response to a user event (a click, a submit, a change), put it in the event handler instead. Likewise, don't use effects to derive state that can be computed during render, or to chain state updates that could be a single handler. Reach for `useEffect` only when there's a genuine external side effect to sync with.

## Accessibility

- Always consider accessibility when adding or changing form fields. Every input should have an associated `<label>` (or `aria-label`), validation errors should be programmatically linked to their field (e.g. `aria-describedby`) and announced, and fields should be keyboard-navigable with a visible focus state. Prefer native form elements over custom controls unless there's a clear reason not to.

## Playwright tests

- Always select elements using accessible, user-meaningful queries — prefer `getByRole`, `getByLabel`, `getByText`, and `getByPlaceholder` over CSS selectors, test IDs, or DOM structure. Tests should find elements the way a user perceives them, which keeps them resilient to markup changes and doubles as a check that the UI is actually accessible.
