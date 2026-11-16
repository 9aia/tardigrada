# Getting started

Bootstrap your first tardigrada identity — the key every age-encrypted
store on this machine will share.

## Install

Once the pack is published to the org registry:

```sh
tamias install tardigrada
```

From a checkout (dev):

```sh
scripts/pack.sh                                # stage the payload
# or simply:
cargo install --path crates/tardigrada
```

## Initialize

```sh
tardigrada init
```

`init` prefers the **system keyring** — the identity lands in the
platform session secret store (entry `tardigrada/age-identity`),
unlocked with your login. Expected output — a note on stderr and
your `age1…` recipient on stdout:

```
tardigrada: created identity at system keyring (entry tardigrada/age-identity)
age1q2w3e…
```

Headless or CI? There is no session store — pick a backend first:

```sh
export TARDIGRADA_IDENTITY_BACKEND=auto   # keyring → keyfile fallback
# or: export TARDIGRADA_IDENTITY_BACKEND=file   # keyfile only
```

With `file` (or `auto` falling back), the keyfile lands at
`$XDG_CONFIG_HOME/tardigrada/identity.txt` (default
`~/.config/tardigrada/identity.txt`), mode `0600`. `init` never
clobbers — run it again and it reports the existing identity. An
existing keyfile is adopted into the keyring, not orphaned.

## Read your recipient

```sh
tardigrada show
```

The `age1…` string on stdout is what you share — with cuora vault
configs, tamias sealed packs, or a `recipients` file for `rage`.

## What you now know

- tardigrada holds **one identity per machine** — in the session
  keyring by default, or a keyfile when you opt out.
- `show` is scriptable: recipient on stdout, context on stderr.
- `--json` wraps every verb's output; errors are `{"error": …}` on
  stderr. `tardigrada backend` reports which store is in effect.

Next: [move your identity](../how-to/move-your-identity) when you
change machines.
