# Point cuora at tardigrada's key

Goal: make cuora vaults decrypt with the same identity tardigrada
manages — one key, one lifecycle, no duplicated keyfiles.

Since cuora canonicalized identity under the tardigrada namespace,
both tools resolve the **same identity by default**: the
`tardigrada/age-identity` session-keyring entry, or the shared
keyfile. The steps below are for pinning an explicit file instead.

## Two ways

### Same file (simplest)

Tell cuora to read tardigrada's keyfile. Either set the env var:

```sh
export CUORA_IDENTITY_FILE="$HOME/.config/tardigrada/identity.txt"
```

…or point tardigrada at cuora's default instead, once, globally:

```sh
export TARDIGRADA_IDENTITY_FILE="$HOME/.config/cuora/identity.txt"
```

Both tools then read and write the same `identity.txt` — cuora's
`init` and tardigrada's verbs stay in sync.

### Move the key material

Copy the keyfile itself:

```sh
# adopt cuora's existing identity into tardigrada
tardigrada import ~/.config/cuora/identity.txt

# or write tardigrada's identity where cuora reads it
tardigrada export --out ~/.config/cuora/identity.txt
```

`export --out` writes the same `identity.txt` format cuora's
`cuora-identity-file` impl reads — comments and all. Note `export`
reads from tardigrada's *configured* backend: under the default
`keyring` it exports the keyring identity; under `file`/`auto`
fallback it exports the keyfile's.

## You should see

```sh
tardigrada show        # age1…
```

cuora has no verb that prints the recipient after `init` — but
`cuora init` is safe to re-run against an existing keyfile (it
imports rather than orphaning) and reports it:

```sh
CUORA_IDENTITY_FILE=~/.config/tardigrada/identity.txt cuora init
# stderr: recipient: age1…  — the same string tardigrada show prints
```

## Boundary note

`tardigrada` manages *generic* identities — it knows nothing about
vaults, `cuora://` refs, or envs. cuora owns secret
materialization; tardigrada owns the key's lifecycle. Sharing the
file is the whole integration.

## Deeper link

- [cuora docs](https://9aia.com/cuora) — vault setup and `cuora
  init`.
