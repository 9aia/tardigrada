# Output contract

How tardigrada talks on stdout/stderr — the machine-readable
surface `--json` and scripts rely on. Verbs and flags:
[CLI](cli) (generated from `--help`).

## Streams

- **stdout** carries the payload: the bare `age1…` recipient for
  `show`/`init`/`import`, the keyfile secret line for bare
  `export`, `file` for `backend`, JSON under `--json`.
- **stderr** carries context notes (`tardigrada: …`) and errors.
  Under `--json` notes are silenced so stderr stays clean for
  piping; errors still emit `{"error": "…"}` on stderr.

## `--json` shapes

`init`, `show`, `import`, `export --out` share one envelope:

```json
{"backend":"file","recipient":"age1…","source":"<path>"}
```

- `init` adds `"created": true|false`.
- `import` adds `"imported_from": "<path>"`.
- `backend` emits `{"backend":"file","source":"<path>"}`.
- `manifest` is always JSON (flag or not): verbs, globals, env
  vars, backend, cuora interop note.

Bare `export --json` is **refused** (`export --json needs --out
PATH`) — a secret inside a JSON envelope invites accidental
logging. Use `--out` or bare `export` for the pipeable form.

## Errors

Any failure exits `1`:

- plain mode: `tardigrada: error: <message>` on stderr
- `--json`: `{"error": "<message>"}` on stderr

Missing identity is actionable: `no identity at <path> — run
`tardigrada init` first`. clap usage errors keep clap's own exit
codes (`2` for parse failures).
