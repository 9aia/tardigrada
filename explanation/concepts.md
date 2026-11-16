# Concepts

## Identity

The age keypair — an `x25519::Identity` (secret) and its
`x25519::Recipient` (public, the `age1…` string). One identity
decrypts everything sealed to its recipient. tardigrada holds
exactly one per keyfile path.

## Keyfile / `identity.txt`

The on-disk form: `#` comment lines (including the public
recipient) plus one `AGE-SECRET-KEY-…` line — the format rage and
cuora's `cuora-identity-file` both read. Written mode `0600`,
parents created, fsync'd. `init` never overwrites; `import` does,
deliberately.

## Recipient

The public half — `age1…`. Safe to share, print, commit-adjacent
(configs, manifests). `tardigrada show` prints it bare on stdout.

## Backend

Where the identity lives. v1 has exactly one: `file` — the keyfile.
`backend` reports it plus the resolved path. A `keyring` backend is
future work (blocked on upstream service-name parameterization).

## Resolution order

The keyfile path, first hit wins:

1. `--identity-file PATH` (global flag)
2. `TARDIGRADA_IDENTITY_FILE`
3. `$XDG_CONFIG_HOME/tardigrada/identity.txt`
4. `~/.config/tardigrada/identity.txt`

## Cryptobiosis

The namesake: the tardigrade's dormant survival state. The identity
equivalent — key material asleep in a `0600` file, persisting
across reboots, tool upgrades, and machine migrations until an
encrypted store wakes it.
