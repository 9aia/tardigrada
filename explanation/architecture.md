# Architecture

## The idea

One long-lived **age identity** per person per machine — the master
key every age-encrypted store on the box decrypts with. tardigrada
owns that key's *lifecycle*: bootstrap, inspect, move. It never
touches ciphertext, vaults, refs, or envs — those belong to the
consumers (cuora, tamias sealed packs, rage).

Named for the tardigrade: the animal survives anything by entering
cryptobiosis — a dormant state it persists through until conditions
return. The identity sleeps in a keyfile (`0600`) and wakes whenever
a store needs it; it persists across tools, machines, and years.

## Components

```
tardigrada (CLI)
   │  clap parse → dispatch → i32        (crates/tardigrada, lib + thin bin)
   ▼
FileIdentity  ── the keyfile impl (cuora-identity-file, crates.io)
   │  load: first non-comment line → x25519::Identity
   │  store: "#" header + AGE-SECRET-KEY-…, mode 0600, fsync
   ▼
identity.txt  ── $TARDIGRADA_IDENTITY_FILE → $XDG_CONFIG_HOME/
                tardigrada/identity.txt → ~/.config/…
```

Single crate, no driver: the command ships as a tamias `type:
files` pack with a `payload:` bin (`libraries/tardigrada`) — a real
binary staged by `scripts/pack.sh`, deployed onto PATH by
`tamias install`. Same shape as `tarsi`'s pack; no `bins[]` shim,
no `tamias-driver-*` crate.

## Ports (inherited, not reinvented)

| Piece | Source |
| --- | --- |
| `IdentitySource` port + `ensure()` | `cuora-identity` (crates.io `0.1`) |
| `FileIdentity` keyfile impl | `cuora-identity-file` (crates.io `0.1`) |
| `x25519::Identity` | `age` (default-features off) |

`tardigrada` calls `FileIdentity::new(<resolved path>)` — the impl
is path-agnostic, so tardigrada's `~/.config/tardigrada/` location
and cuora's `~/.config/cuora/` location are the same code.

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
