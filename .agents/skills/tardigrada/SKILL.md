---
name: tardigrada
description: Manage the per-machine, per-human age identity — tardigrada init/show/import/export/backend over the identity.txt keyfile. Trigger when a task needs the age1… recipient for an encrypted store (cuora vault, tamias sealed pack, rage file) or when moving identity key material between machines/tools.
metadata:
  version: "0.1.0"
---

# tardigrada — age-identity lifecycle

`tardigrada` owns **one thing**: the age identity — the master key
every age-encrypted store on the machine decrypts with. Dormant in a
keyfile, persistent anywhere. It never sees ciphertext, vaults, or
refs — cuora/tamias/rage consume the key; tardigrada manages its
lifecycle.

## The one rule

**The secret key never enters context.** `export` prints
`AGE-SECRET-KEY-…` — that's the explicit escape hatch for *moving*
key material, not for reading it. Prefer `export --out <path>` so
the value lands in a `0600` file, never in your output.

## Bootstrap

```bash
command -v tardigrada   # missing? tamias pack — from a checkout:
scripts/pack.sh && tamias install ./libraries/tardigrada
# or dev install: cargo install --path crates/tardigrada

tardigrada init         # once per machine — never clobbers
tardigrada show         # the age1… recipient (scriptable)
```

## Verbs — by intent

| Intent | Command |
| --- | --- |
| First-time setup | `tardigrada init` |
| Get the recipient to share | `tardigrada show` (stdout bare; `--json` for machines) |
| Move key to another machine/tool | `tardigrada export --out <path>` → `tardigrada import <path>` there |
| Pipe the raw key somewhere | `tardigrada export` (stdout — secret!) |
| Check storage | `tardigrada backend` → always `file` + resolved path |
| Agent self-discovery | `tardigrada manifest` (JSON) |

## Where the keyfile lives

`--identity-file` → `TARDIGRADA_IDENTITY_FILE` →
`$XDG_CONFIG_HOME/tardigrada/identity.txt` →
`~/.config/tardigrada/identity.txt`. Mode `0600`; format is
`identity.txt`-style (`#` comments + one `AGE-SECRET-KEY-…` line) —
the same file cuora and rage read.

## cuora interop

- Same file: `export CUORA_IDENTITY_FILE=<tardigrada keyfile>` (or
  point tardigrada at `~/.config/cuora/identity.txt` via
  `TARDIGRADA_IDENTITY_FILE`).
- Move material: `tardigrada export --out ~/.config/cuora/identity.txt`
  or `tardigrada import ~/.config/cuora/identity.txt`.

## Cautions

- `import` **overwrites** the current identity — no undo. Verify the
  source file first (`head -3` shows comments + the key line).
- `init` is safe to re-run — it reports the existing identity.
- `--json` on every verb; errors are `{"error": …}` on stderr.
