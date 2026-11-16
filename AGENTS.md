# Tardigrada

Age-identity lifecycle tool — the per-machine, per-human key behind
encrypted stores. Named for the tardigrade: dormant identity that
persists anywhere via cryptobiosis. The `tardigrada` CLI bootstraps,
inspects, and moves a single age identity held in an
`identity.txt`-style keyfile (mode `0600`).

<!-- myco:agent-awareness -->

## Agent awareness

- Session start / any work discussion: run `myco status --prompt`
  before proposing next steps — know the board first.
- User describes new work → offer to capture it (`myco todo add`,
  `issue new`, `proposal new`). Don't wait to be asked.
- Picking up a task: `myco resume <ID>` is the briefing.
- Wrapping up: `myco handoff <ID>` anything resumable; keep statuses
  honest (claimed → in-progress → blocked/done).

<!-- /myco:agent-awareness -->

<!-- sturn:agent-awareness -->

## Sturn awareness

- **No PRs.** The pushed branch/commit is the deliverable; review
  streams continuously (sturn WAL), merge is a human/sturn call.
- sturn-wired repo (`sturn.toml` present): `sturn claim "<glob>"
  --ws <ws>` *before* edits — claims surface conflicts in ms.
- Ops via `sturn write|rename|delete`; `sturn snap` folds the ws
  into a real commit; `sturn push` publishes heads + refs/sturn/*.
- `sturn leave` on exit — drops the actor's claims.

<!-- /sturn:agent-awareness -->

## Layout

Flat `crates/` workspace — one crate today.

| Crate | Role |
| --- | --- |
| `crates/tardigrada` (pkg `tardigrada`) | The whole tool — engine lib (`Cli`/`Cmd`, `parse`/`parse_from`/`dispatch`/`run`/`run_from`, clap + `--json` everywhere, `manifest` self-discovery) plus a thin `[[bin]]`. No driver crate: the `tardigrada` command ships as the tamias pack in `libraries/tardigrada/` — a `type: files` payload bin (`bins[].payload` → `tools/bin/<triple>/tardigrada`, staged by `scripts/pack.sh`), the `libraries/tarsi` shape |
| `libraries/tardigrada` | The tamias pack — `tamias.yaml` declares `bins: [{name: tardigrada, payload: tools/bin/<triple>/tardigrada}]`; `tools/` is gitignored, built by `scripts/pack.sh` |
| `scripts/` | `pack.sh` (stage payload), `release.sh` (tarsi shim), `coverage.sh` (llvm-cov) |

Deps of note: `cuora-identity` + `cuora-identity-file` (crates.io
`0.1`) provide the `IdentitySource` port and the keyfile impl —
tardigrada consumes them via `FileIdentity::new(<resolved path>)`
and `cuora_identity::ensure`; it does **not** copy their code.
`age` (default-features off) for `x25519::Identity`.

## Boundary

- tardigrada owns **identity lifecycle** — bootstrap, inspect,
  move. It never sees ciphertext, vaults, refs, or envs.
- **cuora** owns secret materialization; tardigrada's identity is
  what cuora vaults decrypt with — share via the same keyfile path
  (`CUORA_IDENTITY_FILE`) or `export`/`import`.
- **tamias** owns distribution — tardigrada ships as a `type:
  files` pack.
- **myco** owns the `project/` ledger; **tarsi** owns releases.

## Commands

No xtask — plain cargo + scripts:

```bash
cargo build --workspace && cargo test --workspace   # unit + e2e
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
scripts/pack.sh          # release tardigrada → libraries/tardigrada/tools/bin/<triple>/
scripts/release.sh       # operator-run: tarsi-serve release shim
scripts/coverage.sh      # llvm-cov HTML → target/llvm-cov/html
```

Deploy the command: `scripts/pack.sh` then `tamias install
./libraries/tardigrada` (or `cargo install --path crates/tardigrada`
for a dev install).

Git hooks: none wired yet — run the four cargo commands above
before committing.

## The worktree anatomy

`project/` and `docs/` are git worktrees of orphan branches —
`project` (the PM ledger, created by `myco init`), `docs` (the
VitePress site). Both are gitignored on `main`; never commit their
contents to `main`.

## Rules

- Workspace lints are shared (`Cargo.toml [workspace.lints]`):
  `unsafe_code` forbidden, `unwrap_used` warned — treat warnings
  as errors.
- No `unsafe`. Edition 2021.
- Keep the binary dependency-light and sync; no async runtime.
- Secret material (`AGE-SECRET-KEY-…`) never appears in tests,
  fixtures, logs (beyond the explicit `export` stdout escape
  hatch), or docs — fixtures generate fresh identities.
- `init` never clobbers; `import` overwrites deliberately (it is
  the adopt-an-existing-key path — say so in the UI).
- Agents never hand-edit `project/` when a `myco` verb exists.
- `.agents/skills/` is the canonical skill location.

## Docs

`docs/` worktree — VitePress + Diátaxis (tutorials/how-to/
reference/explanation). v1 keeps the site minimal; the README +
this file carry the full contract until the docs site grows.

## Future work

- **Keyring backend** — `cuora-identity-keyring` exists but its
  service name is hardcoded `"cuora"`; a tardigrada backend needs
  either an upstream parameterization or a thin local impl. Track
  in `project/`.
- Passphrase-wrapped portable identities (a `cuora-identity` arm,
  per its port docs).
