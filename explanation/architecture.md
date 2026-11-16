# Architecture

## The idea

One long-lived **age identity** per person per machine — the master
key every age-encrypted store on the box decrypts with. tardigrada
owns that key's *lifecycle*: bootstrap, inspect, move. It never
touches ciphertext, vaults, refs, or envs — those belong to the
consumers (cuora, tamias sealed packs, rage).

Named for the tardigrade: the animal survives anything by entering
cryptobiosis — a dormant state it persists through until conditions
return. The identity sleeps in the platform session keyring (or a
`0600` keyfile) and wakes whenever a store needs it; it persists
across tools, machines, and years.

## Components

```
tardigrada (CLI)
   │  clap parse → backend select → dispatch → i32
   │  (crates/tardigrada, lib + thin bin)
   ▼
IdentitySource impls — consumed from cuora, git-pinned cuora-v1.0.2
   ├─ KeyringIdentity   system keyring entry tardigrada/age-identity
   │                    (secret-service / Keychain / Credential Manager)
   └─ FileIdentity      identity.txt keyfile, mode 0600, fsync
   ▼
session keyring  or  identity.txt at the resolved path
```

Backend selection mirrors cuora: `TARDIGRADA_IDENTITY_BACKEND` =
`keyring` (default, fail closed) | `auto` (keyring → keyfile
fallback) | `file` (keyfile only). An explicit
`--identity-file`/`TARDIGRADA_IDENTITY_FILE` pin always means file
and never probes the store.

Single crate, no driver: the command ships as a tamias `type:
files` pack with a `payload:` bin (`libraries/tardigrada`) — a real
binary staged by `scripts/pack.sh`, deployed onto PATH by
`tamias install`. Same shape as `tarsi`'s pack; no `bins[]` shim,
no `tamias-driver-*` crate.

## Ports (inherited, not reinvented)

| Piece | Source |
| --- | --- |
| `IdentitySource` port + `ensure()` | `cuora-identity` (git `cuora-v1.0.2`) |
| `FileIdentity` keyfile impl | `cuora-identity-file` (git `cuora-v1.0.2`) |
| `KeyringIdentity` session-store impl | `cuora-identity-keyring` (git `cuora-v1.0.2`) |
| `x25519::Identity` | `age` (default-features off) |

`tardigrada` resolves one `IdentitySource` per run — a
`KeyringIdentity` or a `FileIdentity` at the resolved path. The
impls are path-agnostic, so tardigrada's
`~/.config/tardigrada/` location, cuora's, and the shared
`tardigrada/age-identity` keyring entry are the same code — and the
same identity.

## Where it sits

Delivery → memory → process → interface: tardigrada is **process**
infrastructure — a lifecycle tool other tools' identities flow
through. cuora (secrets broker) consumes the identity for vault
decryption; tamias (distribution) ships the binary; myco (work
memory) tracks the ledger; tarsi (release plane) cuts the tags.

## Neighbors

See [Boundaries](boundaries) for the owns / doesn't-own split and
the per-neighbor relationship table; [Decisions](decisions) for
the why behind each of those lines.
