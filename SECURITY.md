# Security Policy

Report vulnerabilities privately by emailing
luisfuturist+9aia@gmail.com with "SECURITY" in the subject — do
not open public issues, do not use GitHub's advisory or
private-reporting features, and do not disclose details publicly;
contact us first and we will coordinate disclosure privately.

## Scope

- tardigrada manages the **age identity** — the master key every
  age-encrypted store decrypts with (cuora vaults, tamias sealed
  packs, rage files). See `README.md` for the verb surface.
- `identity.txt` (mode `0600`) is raw key material — treat it like
  a private SSH key. `export` prints secret material to stdout by
  design; it is the explicit escape hatch.
- `init` never clobbers an existing identity; `import` overwrites
  deliberately — pointing `import` at the wrong file destroys the
  current key. There is no undo.
- Keyfile permissions looser than `0600` warn on load (non-fatal,
  ssh strict-modes posture). Symlinked keyfile paths are refused on
  write (inherited from `cuora-identity-file`).

## Notes

We respond to reports as quickly as we can; expect acknowledgement
within a few days.
