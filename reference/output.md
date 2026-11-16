# Output contract

How tardigrada talks on stdout/stderr — the machine-readable
surface `--json` and scripts rely on. Verbs and flags:
[CLI](cli) (generated from `--help`).

## Streams

- **stdout** carries the payload: the bare `age1…` recipient for
  `show`/`init`/`import`, the keyfile secret line for bare
  `export`, the resolved backend (`keyring`/`file`) for `backend`,
  JSON under `--json`.
- **stderr** carries context notes (`tardigrada: …`) and errors —
  including backend fallback notes (`no session keyring reachable
  (…) — using the identity file`). Under `--json` notes are
  silenced so stderr stays clean for piping; errors still emit
  `{"error": "…"}` on stderr.

## `--json` shapes

`init`, `show`, `import`, `export --out` share one envelope:

```json
{"backend":"keyring","recipient":"age1…","source":"system keyring (entry tardigrada/age-identity)"}
```

- `backend` is the resolved source kind — `"keyring"` or `"file"`.
- `source` describes it — a keyfile path or the store entry.
- `init` adds `"created": true|false`; a keyring init that adopted
  an existing keyfile adds `"migrated_from": "<keyfile path>"`.
- `import` adds `"imported_from": "<keyfile path>"`.
- `backend` emits `{"backend","mode","pinned","source"}`:
  `backend` = resolved kind, `mode` = the configured
  `TARDIGRADA_IDENTITY_BACKEND` (`keyring`/`auto`/`file`), `pinned`
  = an explicit `--identity-file`/`TARDIGRADA_IDENTITY_FILE` is in
  effect.
- `manifest` is always JSON (flag or not): verbs, globals, env
  vars, backends, cuora interop note.

Bare `export --json` is **refused** (`export --json needs --out
PATH`) — a secret inside a JSON envelope invites accidental
logging. Use `--out` or bare `export` for the pipeable form.

## Errors

Any failure exits `1`:

- plain mode: `tardigrada: error: <message>` on stderr
- `--json`: `{"error": "<message>"}` on stderr

Missing identity is actionable: `no identity at <source> — run
`tardigrada init` first`. A `keyring` backend with no reachable
session store fails closed:
`identity backend unavailable: backend=keyring: … — set
TARDIGRADA_IDENTITY_BACKEND=auto|file for headless/CI`. clap usage
errors keep clap's own exit codes (`2` for parse failures).
