# Tardigrada

*Dormant identity that persists anywhere.*

Named for the tardigrade — the microscopic animal that survives
anything via cryptobiosis, suspended in a dry shell until water
returns. tardigrada is the same idea for keys: a **per-machine,
per-human age identity** that sleeps in an `identity.txt` keyfile
(mode `0600`) and wakes whenever an encrypted store needs it. It is
the key behind cuora vaults, tamias sealed packs, rage files — any
tool that seals to an `age1…` recipient.

```
tardigrada init      →  ~/.config/tardigrada/identity.txt (0600)
tardigrada show      →  age1…   (the recipient you share)
tardigrada export    →  AGE-SECRET-KEY-…  (move it anywhere)
tardigrada import    →  adopt a keyfile from another tool/machine
```

## Why

Every encrypted store needs the same thing: one long-lived identity
that decrypts everything it protects. cuora vaults, tamias sealed
packs, and `rage`-compatible files all speak age — but each tool
used to grow its own bootstrap path, its own keyfile location, its
own "where does my key live" story. tardigrada is the standalone
lifecycle tool for that key: one identity, one file format, one set
of verbs — and `export`/`import` to carry it between tools and
machines.

## Quickstart

```sh
# install — the command ships as a tamias pack (libraries/tardigrada)
scripts/pack.sh                              # or: cargo build --release
tamias install ./libraries/tardigrada        # deploys `tardigrada`
# dev install without tamias:
cargo install --path crates/tardigrada

tardigrada init        # bootstrap the identity, once per machine
tardigrada show        # print the age1… recipient (scriptable)
tardigrada backend     # inspect storage — always `file` in v1
```

`tardigrada --help` lists every verb; `tardigrada manifest` prints
the machine-readable capability surface (always JSON). `--json` is
honored on every verb; errors are `{"error": "…"}` on stderr with a
non-zero exit.

## The verbs

| Verb | What it does |
| --- | --- |
| `init` | Generate + store the identity. Never clobbers — a second `init` reports the current recipient. |
| `show` | Print the `age1…` recipient to stdout (scriptable); backend/path context on stderr. Fails until `init` has run. |
| `import <file>` | Adopt an existing `identity.txt`-style keyfile — parses it, stores the identity at the configured path (overwrites). |
| `export [--out PATH]` | Write the keyfile out (mode `0600`). Bare `export` prints the `AGE-SECRET-KEY-…` line to stdout — the pipeable escape hatch. |
| `backend` | Inspect configured storage — `file` + resolved path in v1. |
| `manifest` | Capabilities/env/JSON shapes — agent self-discovery. |

## Where the key lives

Resolution order for the keyfile path:

1. `--identity-file PATH` (global flag)
2. `$TARDIGRADA_IDENTITY_FILE`
3. `$XDG_CONFIG_HOME/tardigrada/identity.txt`
4. `~/.config/tardigrada/identity.txt`

The file is an `identity.txt`-style keyfile — `#` comments plus one
`AGE-SECRET-KEY-…` line — written mode `0600` with parent dirs
created, the same format cuora and rage read. A group/world-readable
file warns on load but still works (ssh's strict-modes posture,
non-fatal).

## cuora interop

cuora vaults decrypt with an age identity too. Two ways to share:

- **Point cuora at the same file** — set `CUORA_IDENTITY_FILE` (or a
  vault's `identity` config) to tardigrada's keyfile path, or let
  cuora's own `~/.config/cuora/identity.txt` be the file tardigrada
  manages via `--identity-file`/`TARDIGRADA_IDENTITY_FILE`.
- **Move the key material** — `tardigrada export --out
  ~/.config/cuora/identity.txt` writes a cuora-readable keyfile, or
  `tardigrada import ~/.config/cuora/identity.txt` adopts cuora's
  existing identity so both tools share one key.

`tardigrada` manages *generic* identities — it does not know about
vaults, refs, or envs. cuora owns secret materialization; tardigrada
owns the key's lifecycle.

## Position in the stack

- **cuora** — secrets broker; consumes age identities for vaults.
- **tamias** — distribution; tardigrada ships as a `type: files`
  pack with a payload bin (no driver).
- **myco** — work ledger (`project/` worktree).
- **tarsi** — release plane (`scripts/release.sh` shim, `tarsi.toml`).

## Docs

- [AGENTS.md](AGENTS.md) — layout, commands, rules
- [CONTRIBUTING.md](CONTRIBUTING.md) · [SECURITY.md](SECURITY.md)
- [LICENSE](LICENSE) (MIT)
- Docs site: <https://9aia.com/tardigrada> — `docs/` worktree,
  VitePress + Diátaxis (orphan `docs` branch)
