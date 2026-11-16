# Share your recipient with a tool

Goal: get the `age1…` string a tool needs to seal data to your
identity — without ever handling the secret key.

## Steps

1. Print the recipient:

   ```sh
   tardigrada show
   ```

   stdout is the bare `age1…` string — pipe it straight into config:

   ```sh
   RECIPIENT=$(tardigrada show)
   echo "recipient = \"$RECIPIENT\"" >> recipients.txt
   ```

2. For machine-readable use:

   ```sh
   tardigrada --json show
   # {"backend":"file","recipient":"age1…","source":"…"}
   ```

3. Hand the recipient to the tool — cuora `[vaults.*].recipients`,
   a tamias sealed-pack manifest, or `rage -r "$RECIPIENT"`.

## You should see

- `tardigrada show` exits 0 and prints one `age1…` line.
- Before `init` has ever run, it exits 1 with
  `no identity at … — run 'tardigrada init' first`.

## Deeper link

- [Point cuora at tardigrada's key](cuora-interop) — the full
  onboarding story.
