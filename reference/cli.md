# Command-Line Help for `tardigrada`

This document contains the help content for the `tardigrada` command-line program.

tardigrada — the per-machine, per-human age identity behind encrypted stores. `init` bootstraps the identity (generate + store, never clobbers); `show` prints the `age1…` recipient to share with tools that seal to it; `import`/`export` move key material between files and machines; `backend` inspects storage. `--json` is honored on every verb; the keyfile is the only secret tardigrade touches.

**Command Overview:**

* [`tardigrada`↴](#tardigrada)
* [`tardigrada init`↴](#tardigrada-init)
* [`tardigrada show`↴](#tardigrada-show)
* [`tardigrada import`↴](#tardigrada-import)
* [`tardigrada export`↴](#tardigrada-export)
* [`tardigrada backend`↴](#tardigrada-backend)
* [`tardigrada manifest`↴](#tardigrada-manifest)

## `tardigrada`

tardigrada — the per-machine, per-human age identity behind encrypted stores. `init` bootstraps the identity (generate + store, never clobbers); `show` prints the `age1…` recipient to share with tools that seal to it; `import`/`export` move key material between files and machines; `backend` inspects storage. `--json` is honored on every verb; the keyfile is the only secret tardigrade touches.

**Usage:** `tardigrada [OPTIONS] <COMMAND>`

###### **Commands:**

* `init` — Bootstrap the age identity — generate + store it. Never clobbers an existing identity: a second `init` reports the current recipient and exits clean
* `show` — Print the `age1…` recipient of the stored identity — the value tools seal to (cuora vaults, tamias packs, rage recipients files). Goes to stdout bare (scriptable); backend/source context on stderr. Fails when no identity exists — run `tardigrada init` first
* `import` — Adopt an existing `identity.txt`-style keyfile as this machine's identity — parses the file, stores the identity at the configured path (overwrites whatever is there; the old key is gone for good)
* `export` — Write the identity keyfile out — mode `0600`, the same format `import` reads. `--out PATH` writes a file for moving between tools/machines; bare `export` prints the `AGE-SECRET-KEY-…` line to stdout (pipe it into another tool's keyfile, or `rage -e -i /dev/stdin`). This is the explicit escape hatch — the value is secret
* `backend` — Inspect the configured identity backend — always `file` in v1, plus the resolved keyfile path
* `manifest` — Capabilities, JSON shapes, env vars — agent self-discovery (always JSON)

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--identity-file <PATH>` — Identity keyfile path, overriding `$TARDIGRADA_IDENTITY_FILE` and the XDG default (`$XDG_CONFIG_HOME/tardigrada/identity.txt`)
* `-h, --help` — Print help (see a summary with '-h')
* `-V, --version` — Print version

## `tardigrada init`

Bootstrap the age identity — generate + store it. Never clobbers an existing identity: a second `init` reports the current recipient and exits clean

**Usage:** `tardigrada init [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--identity-file <PATH>` — Identity keyfile path, overriding `$TARDIGRADA_IDENTITY_FILE` and the XDG default (`$XDG_CONFIG_HOME/tardigrada/identity.txt`)
* `-h, --help` — Print help

## `tardigrada show`

Print the `age1…` recipient of the stored identity — the value tools seal to (cuora vaults, tamias packs, rage recipients files). Goes to stdout bare (scriptable); backend/source context on stderr. Fails when no identity exists — run `tardigrada init` first

**Usage:** `tardigrada show [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--identity-file <PATH>` — Identity keyfile path, overriding `$TARDIGRADA_IDENTITY_FILE` and the XDG default (`$XDG_CONFIG_HOME/tardigrada/identity.txt`)
* `-h, --help` — Print help

## `tardigrada import`

Adopt an existing `identity.txt`-style keyfile as this machine's identity — parses the file, stores the identity at the configured path (overwrites whatever is there; the old key is gone for good)

**Usage:** `tardigrada import [OPTIONS] <FILE>`

###### **Arguments:**

* `<FILE>` — Path to the identity.txt keyfile to adopt

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--identity-file <PATH>` — Identity keyfile path, overriding `$TARDIGRADA_IDENTITY_FILE` and the XDG default (`$XDG_CONFIG_HOME/tardigrada/identity.txt`)
* `-h, --help` — Print help

## `tardigrada export`

Write the identity keyfile out — mode `0600`, the same format `import` reads. `--out PATH` writes a file for moving between tools/machines; bare `export` prints the `AGE-SECRET-KEY-…` line to stdout (pipe it into another tool's keyfile, or `rage -e -i /dev/stdin`). This is the explicit escape hatch — the value is secret

**Usage:** `tardigrada export [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--out <PATH>` — Destination keyfile path (mode `0600`, parents created)
* `--identity-file <PATH>` — Identity keyfile path, overriding `$TARDIGRADA_IDENTITY_FILE` and the XDG default (`$XDG_CONFIG_HOME/tardigrada/identity.txt`)
* `-h, --help` — Print help

## `tardigrada backend`

Inspect the configured identity backend — always `file` in v1, plus the resolved keyfile path

**Usage:** `tardigrada backend [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--identity-file <PATH>` — Identity keyfile path, overriding `$TARDIGRADA_IDENTITY_FILE` and the XDG default (`$XDG_CONFIG_HOME/tardigrada/identity.txt`)
* `-h, --help` — Print help

## `tardigrada manifest`

Capabilities, JSON shapes, env vars — agent self-discovery (always JSON)

**Usage:** `tardigrada manifest [OPTIONS]`

###### **Options:**

* `--json` — Emit JSON — stable machine shapes on every verb; errors are `{"error": "…"}` on stderr with a non-zero exit
* `--identity-file <PATH>` — Identity keyfile path, overriding `$TARDIGRADA_IDENTITY_FILE` and the XDG default (`$XDG_CONFIG_HOME/tardigrada/identity.txt`)
* `-h, --help` — Print help
