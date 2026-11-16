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

Expected output — a note on stderr and your `age1…` recipient on
stdout:

```
tardigrada: created identity at ~/.config/tardigrada/identity.txt
age1q2w3e…
```

The keyfile lands at `$XDG_CONFIG_HOME/tardigrada/identity.txt`
(default `~/.config/tardigrada/identity.txt`), mode `0600`. `init`
never clobbers — run it again and it reports the existing identity.

## Read your recipient

```sh
tardigrada show
```

The `age1…` string on stdout is what you share — with cuora vault
configs, tamias sealed packs, or a `recipients` file for `rage`.

## What you now know

- tardigrada holds **one identity per keyfile path** — per-machine,
  per-human by convention.
- `show` is scriptable: recipient on stdout, context on stderr.
- `--json` wraps every verb's output; errors are `{"error": …}` on
  stderr.

Next: [move your identity](../how-to/move-your-identity) when you
change machines.
