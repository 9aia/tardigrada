# Decisions

ADR-lite: decision → alternatives → consequence.

## Keyring default, fail closed; file opt-out

**Decision:** `TARDIGRADA_IDENTITY_BACKEND` selects the store —
`keyring` (default: the platform session secret store, entry
`tardigrada/age-identity`) fails closed when no store is reachable;
`auto` falls back to the keyfile with a stderr note; `file` never
probes the store. An explicit `--identity-file`/
`TARDIGRADA_IDENTITY_FILE` pin always means file.

**Alternatives:** file-only (v1); keyring default with silent
file fallback; a config file for backend selection.

**Consequence:** the secret leaves the disk on desktop sessions —
the keyring unlocks with login, so reads need no prompt and there is
no `0600` file to leak. Fail-closed means headless/CI must opt out
explicitly (`auto` or `file`) rather than silently diverging onto a
keyfile the keyring can't see — the same semantics cuora adopted.
No config file in v1: one env var covers it, and the explicit pin
always wins. Keyfiles keep working: `file`/`auto` read them
(including legacy `cuora`/`arca` paths via the shared
`cuora-identity-file` resolution), and `init` under
`keyring`/`auto` adopts a keyfile identity into the store instead
of orphaning its ciphertext.

## `export`/`import` over config coupling

**Decision:** cuora interop is "point both tools at the same
keyfile" or "move the keyfile" — no cuora-aware verbs, no shared
config discovery.

**Alternatives:** tardigrada reads/writes cuora's config; a
`tardigrada cuora` subcommand family.

**Consequence:** tardigrada manages *generic* identities — it never
learns what vaults, `cuora://` refs, or envs are. The integration
surface stays one file and two env vars; either tool can be swapped
without the other noticing.

## Payload bin, not a driver shim

**Decision:** the `tardigrada` command ships as a tamias
`type: files` pack whose `bins[].payload` is the compiled binary
staged by `scripts/pack.sh` — a real binary on PATH.

**Alternatives:** the org's usual `bins[]` shim →
`tamias-driver-*` → engine-lib passthrough (cuora/myco shape); a
driver crate.

**Consequence:** no shim→driver→engine indirection for a tool this
size — one fewer crate, one fewer process, `cargo install --path`
and `tamias install` both deploy the same artifact. The cost is a
per-triple build artifact (`tools/bin/<triple>/`, gitignored)
instead of a portable shim.

## `init` never clobbers; `import` overwrites deliberately

**Decision:** `init` is idempotent-safe (second run reports the
existing recipient); `import` replaces whatever identity is
configured, with no undo.

**Alternatives:** `init --force`; `import` to a staging path.

**Consequence:** the dangerous verb is the explicit one — adopting
an existing key is a deliberate act, and pointing `import` at the
wrong file is on the operator (SECURITY.md scope note). `export`
refuses bare-stdout under `--json` for the same reason: a secret
inside a JSON envelope invites accidental logging.
