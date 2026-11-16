# Switch identity backends

Goal: move between the session keyring and the keyfile — or pick
the right backend for a headless box — without losing the identity.

## Prerequisites

- `tardigrada` installed; `tardigrada backend` shows the current
  resolved backend.

## The modes

`TARDIGRADA_IDENTITY_BACKEND` selects the store:

| Value | Behavior |
| --- | --- |
| `keyring` (default) | Platform session secret store, entry `tardigrada/age-identity`. **Fail closed** — no reachable store is an error. |
| `auto` | Keyring when reachable, keyfile fallback with a stderr note. |
| `file` | Keyfile only; the store is never probed. |

An explicit `--identity-file`/`TARDIGRADA_IDENTITY_FILE` pin always
means file and wins over every mode.

## Steps

1. Check where the identity lives today:

   ```sh
   tardigrada backend
   # keyring — or file + the resolved keyfile path
   ```

2. To move a **keyfile identity into the keyring**: just run
   `tardigrada init` (or set `TARDIGRADA_IDENTITY_BACKEND=auto` and
   run any verb). Under `keyring`/`auto`, `init` adopts the
   keyfile's identity into the store — the file stays as a backup,
   nothing is orphaned. Verify:

   ```sh
   tardigrada backend   # → keyring
   tardigrada show      # same age1… as before
   ```

3. To move a **keyring identity to a keyfile** (another machine, a
   headless box):

   ```sh
   tardigrada export --out ~/.config/tardigrada/identity.txt
   ```

   then on the target, `tardigrada import <path>` — or set
   `TARDIGRADA_IDENTITY_BACKEND=file` and let the `file` backend
   read the keyfile directly.

4. For **headless/CI**, opt out of the keyring probe entirely:

   ```sh
   export TARDIGRADA_IDENTITY_BACKEND=auto   # or =file
   ```

   `auto` still uses the keyring when a session store exists;
   `file` never probes.

## You should see

- `tardigrada backend` prints the resolved backend; `--json`
  adds `mode` (the configured selection) and `pinned`.
- `tardigrada show` prints the **same** `age1…` recipient before
  and after a backend switch — the identity moved, it didn't
  change.

## Deeper link

- [Environment variables](../reference/env-vars) — every var and
  the resolution order.
- [Decisions](../explanation/decisions) — why keyring is the
  fail-closed default.
