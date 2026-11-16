# tardigrada

*Dormant identity that persists anywhere.*

tardigrada is the per-machine, per-human **age identity** behind
encrypted stores — cuora vaults, tamias sealed packs, rage files.
One keyfile, one set of verbs: bootstrap it, read its recipient,
move it between tools and machines.

## Which page answers your goal?

- **New here** → [Getting started](tutorials/getting-started) — the
  golden path: install, `init`, `show`.
- **A task** → [How-to](how-to/move-your-identity) — move the key,
  share the recipient, wire cuora.
- **Why it's shaped this way** → [Explanation](explanation/architecture)
  — the mental model and the boundaries.
- **Lookup** → [Reference](reference/cli) — every verb, flag, and env
  var.

## Boundary

| Repo | Owns |
| --- | --- |
| **tardigrada** | the age identity's lifecycle — init/show/import/export |
| **cuora** | secret materialization — vaults, refs, env injection |
| **tamias** | distribution — tardigrada ships as a `type: files` pack |
| **myco** | the `project/` work ledger |
| **tarsi** | the release plane |
