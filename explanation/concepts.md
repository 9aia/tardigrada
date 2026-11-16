# Concepts

## Identity

The age keypair — an `x25519::Identity` (secret) and its
`x25519::Recipient` (public, the `age1…` string). One identity
decrypts everything sealed to its recipient. tardigrada holds
exactly one per machine — in the configured backend.

## Backend

Where the identity lives. `TARDIGRADA_IDENTITY_BACKEND` selects it
(cuora's model):

- **`keyring`** (default) — the platform session secret store
  (secret-service / Keychain / Credential Manager), entry
  `tardigrada/age-identity`. Fail closed: no reachable store is an
  error, never a silent keyfile read.
- **`auto`** — keyring when reachable, keyfile fallback (a
  degraded session notes it on stderr, never crashes).
- **`file`** — the keyfile only; the store is never probed.

An explicit `--identity-file`/`TARDIGRADA_IDENTITY_FILE` pin always
means file. `backend` reports the resolved backend, the configured
mode, and the pin state.

## Keyfile / `identity.txt`

The on-disk form: `#` comment lines (including the public
recipient) plus one `AGE-SECRET-KEY-…` line — the format rage and
cuora's `cuora-identity-file` both read. Written mode `0600`,
parents created, fsync'd. `init` never overwrites; `import` does,
deliberately. Keyfiles remain fully supported — the `file`
backend (and `auto`'s fallback) reads them, and `init` under
`keyring`/`auto` adopts an existing keyfile identity into the
store rather than orphaning it.

## Recipient

The public half — `age1…`. Safe to share, print, commit-adjacent
(configs, manifests). `tardigrada show` prints it bare on stdout.

## Resolution order

The keyfile path, first hit wins (the `file` backend and `auto`
fallback — via `cuora-identity-file`'s shared default, which also
resolves legacy `cuora`/`arca` keyfiles):

1. `--identity-file PATH` (global flag)
2. `TARDIGRADA_IDENTITY_FILE`
3. `CUORA_IDENTITY_FILE` / `ARCA_IDENTITY_FILE` (legacy overrides)
4. `<config>/tardigrada|cuora|arca/identity.txt` — first existing,
   tardigrada canonical first
5. creation target: `<config>/tardigrada/identity.txt`
   (`$XDG_CONFIG_HOME` → `~/.config`)

## Cryptobiosis

The namesake: the tardigrade's dormant survival state. The identity
equivalent — key material asleep in a `0600` file, persisting
across reboots, tool upgrades, and machine migrations until an
encrypted store wakes it.
