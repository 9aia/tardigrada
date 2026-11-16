# Environment variables

| Var | Read by | Effect |
| --- | --- | --- |
| `TARDIGRADA_IDENTITY_BACKEND` | backend selection | `keyring` (default, fail closed) \| `auto` (keyring → keyfile fallback) \| `file` (keyfile only). Unrecognized values warn on stderr (`{"warning": …}` under `--json`) and mean `keyring`. |
| `TARDIGRADA_IDENTITY_FILE` | path resolution + pin | Keyfile path override (empty = ignored). A non-empty value pins the file backend — the keyring is never probed. |
| `CUORA_IDENTITY_FILE` / `ARCA_IDENTITY_FILE` | path resolution + pin | Legacy keyfile pins, honored identically to `TARDIGRADA_IDENTITY_FILE` — a non-empty value pins the file backend so tardigrada and cuora can never split onto different identities. Same-uid trust: whoever can set env can already read the keyfile. |
| `XDG_CONFIG_HOME` | path resolution | Config root — default keyfile is `$XDG_CONFIG_HOME/tardigrada/identity.txt` |
| `HOME` | path resolution | Fallback config root `~/.config/…` when `XDG_CONFIG_HOME` is unset |
| `CUORA_KEYRING_TIMEOUT_MS` | `cuora-identity-keyring` | Bounds one keyring op in ms (`0` waits forever); default 120s. |

## Backend selection

`TARDIGRADA_IDENTITY_BACKEND` picks the store, mirroring cuora:

- **`keyring`** (default) — the platform session secret store,
  entry `tardigrada/age-identity` (legacy `cuora`/`arca` entries
  read as fallbacks). **Fail closed**: an unreachable store is an
  error, never a silent keyfile read.
- **`auto`** — keyring when reachable; an unreachable store, an
  unreadable entry, or an empty store falls back to the keyfile
  (with a stderr note on failures). The headless/CI mode.
- **`file`** — the keyfile only; the store is never probed.

An explicit `--identity-file` flag, `TARDIGRADA_IDENTITY_FILE`, or a
legacy `CUORA_IDENTITY_FILE`/`ARCA_IDENTITY_FILE` pin always means
file and wins over every backend mode — no dbus round-trip. An env
pin under `backend=keyring` (the default) emits a warning; the flag
is the caller's deliberate choice and stays a quiet note. All env
vars are same-uid trust — they redirect resolution freely by
design, not a security boundary.

## Keyfile path resolution

Used by the `file` backend and the `auto` fallback (the
`cuora-identity-file` default):

1. `--identity-file PATH` (global flag)
2. `TARDIGRADA_IDENTITY_FILE`
3. `CUORA_IDENTITY_FILE` / `ARCA_IDENTITY_FILE` (legacy, via the
   shared impl)
4. first existing of `<config>/tardigrada|cuora|arca/identity.txt`
   across the XDG config home and `~/.config`
5. creation target: `<config>/tardigrada/identity.txt`
