# Contributing

Issues and PRs are **disabled** — the project is agent-maintained and
dogfoods its own PM ledger (`project/`). Fork freely under MIT.

## Layout

`crates/` — flat rust workspace, one crate (`crates/tardigrada`):
the engine lib + thin `[[bin]]`. `libraries/tardigrada/` is the
tamias pack (`type: files` payload bin — `scripts/pack.sh` stages
`tools/bin/<triple>/tardigrada`). `scripts/` holds pack/release/
coverage helpers. See the crate table in
[AGENTS.md](AGENTS.md#layout).

## Rules of thumb

- `cargo build`, `cargo test`, `cargo clippy --workspace
  --all-targets -- -D warnings`, and `cargo fmt --all -- --check`
  must stay green.
- Keep the binary dependency-light and sync; no async runtime.
- Never commit key material — tests and fixtures generate fresh
  identities; no `AGE-SECRET-KEY-…` in docs, logs, or fixtures.
- `init` is no-clobber; `import` is the deliberate-overwrite path.
- Commits: conventional prefixes (`feat:`/`fix:`/`refactor:`/
  `docs:`/`chore:`).
- Never hand-edit `project/` when a `myco` verb exists — the ledger
  is myco's, verbs are the audit trail.

## Releasing

No crates.io publish — the install path is the tamias pack
(`libraries/tardigrada`): `scripts/pack.sh` stages the compiled
payload and `tamias install ./libraries/tardigrada` deploys the
`tardigrada` command. `scripts/release.sh` is run manually by an
operator on `main`: one workspace version bump, a `tardigrada-v<ver>`
tag via `tarsi-serve release`. After the tag the operator ships the
pack with `tamias publish libraries/tardigrada` (`9aia:tardigrada@<ver>`).

## Docs

Follow Diátaxis: `docs/tutorials/` (learn) · `docs/how-to/` (tasks) ·
`docs/reference/` (lookup) · `docs/explanation/` (why). Keep
`lib.rs` module docs accurate — they feed the docs site.
