# Move your identity to another machine

Goal: carry your tardigrada identity — and therefore every store it
unlocks — to a new machine, or adopt a keyfile another tool already
holds. `export`/`import` always move **keyfile** material: `export`
reads from whichever backend is configured (the session keyring by
default) and `import` writes to it. To switch *where* the identity
lives on one machine, see [Switch backends](switch-backends).

## Prerequisites

- `tardigrada` installed on both machines.
- A safe channel for the keyfile (encrypted volume, password
  manager, `scp` over ssh — never email or chat).

## Steps

1. On the source machine, write the keyfile out:

   ```sh
   tardigrada export --out ~/tardigrada-identity.txt
   ```

   Verify it was written mode `0600`:

   ```sh
   ls -l ~/tardigrada-identity.txt   # -rw-------
   ```

2. Copy it to the destination machine.

3. On the destination, adopt it:

   ```sh
   tardigrada import ~/tardigrada-identity.txt
   tardigrada show                   # same age1… as the source
   ```

   `import` overwrites whatever identity the destination held —
   there is no undo, so be sure the source file is the right one.

4. Clean up the copy when done:

   ```sh
   rm ~/tardigrada-identity.txt      # on both machines
   ```

## Pipeable variant

Bare `export` prints just the `AGE-SECRET-KEY-…` line — usable as a
stream:

```sh
ssh newmachine 'cat > ~/identity.txt && chmod 600 ~/identity.txt' \
    < <(tardigrada export)
# on the destination: tardigrada import ~/identity.txt
```

The stream holds just the `AGE-SECRET-KEY-…` line — `import` parses
it fine (comments are optional).

## Deeper link

- [CLI reference](../reference/cli) — every flag and env var.
- [Concepts](../explanation/concepts) — what the keyfile actually is.
