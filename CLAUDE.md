# Project guidance

## README

- Whenever you edit `README.md`, scan the surrounding content for stale information — commands that no longer exist, prerequisites that have changed, workflow descriptions that don't match current CI, or sections that describe features that have been added or removed. Fix anything that's out of date in the same edit rather than leaving it for later.

## Code comments

- Don't write comments that restate what the code already makes obvious. Only add a comment when it captures something the code can't: a non-obvious constraint, a subtle invariant, a workaround for a specific bug, or a security/performance pitfall that isn't inferable from reading the surrounding lines. If a comment would just describe what a well-named function or variable already says, leave it out.
