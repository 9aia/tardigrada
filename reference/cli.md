# CLI reference

`tardigrade [--json] [--identity-file PATH] <verb>`

Global flags (before or after the verb):

| Flag | Meaning |
| --- | --- |
| `--json` | Stable machine shapes on every verb; errors are `{"error": "…"}` on stderr, exit 1 |
| `--identity-file PATH` | Keyfile path override — beats `TARDIGRADA_IDENTITY_FILE` and the XDG default |
| `--version`, `--help` | Standard clap |

## Verbs

### `init`

Bootstrap the identity — generate + store. **Never clobbers**: a
second `init` reports the existing recipient and exits 0.

Output: recipient on stdout; `created|found identity at <path>` on
stderr. `--json` adds `"created": true|false`.

### `show`

Print the `age1…` recipient — bare on stdout (scriptable), context
(`identity: <path> (backend file)`) on stderr. Fails with
`no identity at <path> — run 'tardigrada init' first` when nothing
is stored.

`--json`: `{"backend":"file","recipient":"age1…","source":"<path>"}`

### `import <file>`

Adopt an existing `identity.txt`-style keyfile: parse the file,
store the identity at the configured path. **Overwrites** whatever
identity is there — deliberate, no undo. Fails when the source
holds no parseable key.

`--json` adds `"imported_from"`.

### `export [--out PATH]`

Write the keyfile out.

- `--out PATH` — writes a full `identity.txt` (comments +
  `AGE-SECRET-KEY-…`), mode `0600`, parents created.
- bare — prints just the `AGE-SECRET-KEY-…` line to stdout (the
  pipeable escape hatch). Refused under `--json` — a secret inside
  a JSON envelope invites accidental logging; use `--out` or bare.

### `backend`

Inspect storage. Always `file` in v1.

Plain: `file` on stdout, `source: <path>` on stderr.
`--json`: `{"backend":"file","source":"<path>"}`

### `manifest`

Self-discovery — always JSON: verbs, globals, env vars, backend,
cuora interop note.

## Exit codes

`0` success · `1` any error (identity missing/corrupt, io failure,
bad usage handled by clap with its own codes).
