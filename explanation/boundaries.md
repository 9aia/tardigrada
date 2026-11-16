# Boundaries

## Owns

The **age identity's lifecycle** — bootstrap (`init`), inspect
(`show`/`backend`/`manifest`), and movement (`import`/`export`) of
the single per-machine, per-human identity. Plus the backend
selection (`TARDIGRADA_IDENTITY_BACKEND`: `keyring`/`auto`/`file`),
the keyfile *path* convention (`--identity-file` →
`TARDIGRADA_IDENTITY_FILE` → the shared `cuora-identity-file`
default → XDG → `~/.config`), and the `identity.txt` on-disk format
it writes.

## Doesn't own

- **Ciphertext** — tardigrada never sees vaults, sealed packs,
  `rage` files, or any encrypted payload. It produces and moves the
  key; consumers do the sealing.
- **Secret materialization** — refs, env injection, audit: cuora.
- **Distribution** — the `tardigrada` command ships as a tamias
  `type: files` pack; tamias owns install/registry.
- **Releases** — tags and version bumps are tarsi's
  (`scripts/release.sh` is a shim); the work ledger is myco's
  (`project/`).

## Neighbors

| Neighbor | Relationship |
| --- | --- |
| **cuora** | consumes age identities for vault decryption. Interop is storage-level: the same `tardigrada/age-identity` keyring entry, the same keyfile (`CUORA_IDENTITY_FILE` ↔ `TARDIGRADA_IDENTITY_FILE`), or moved material via `export --out`/`import`. No cuora-aware verbs here. |
| **tamias** | ships the binary — `libraries/tardigrada` pack, `type: files` payload bin staged by `scripts/pack.sh`. |
| **rage / age ecosystem** | the keyfile format is `identity.txt`-compatible — anything that reads `AGE-SECRET-KEY-…` files interoperates for free. |
| **cuora-identity-*** | the `IdentitySource` port and both impls (keyfile, platform keyring) are consumed from cuora, git-pinned to `cuora-v1.0.2` — tardigrada never reimplements them. The keyring entry `tardigrada/age-identity` is the canonical namespace both tools share. |

## The rule of thumb

If a feature needs to know what the identity *decrypts*, it belongs
in the consumer (cuora, tamias sealed packs), not here. tardigrada
stays a lifecycle tool: keys in, keys out, nothing in between.
