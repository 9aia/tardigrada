#!/usr/bin/env bash
# tardigrada release — thin shim over tarsi-serve (the org release
# plane, project ADR-0005; repo config: ./tarsi.toml). One call runs
# plan → changelog → bump → tag → push; publish targets are off
# (ADR-0004: crates.io is tamias-only, GitHub Releases unused).
# Operator-run on main — `tarsi-serve release` is dry-run by
# default; --yes is the explicit opt-in.
#
# The pack publish itself stays manual (`tamias publish
# libraries/tardigrada`).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
# worktree-local env (gitignored); the service env file is canonical
[ -f ./.env ] && { set -a; source ./.env; set +a; }

tarsi-serve release --dir . --yes

echo "released: see tag tardigrada-v<ver>"
echo "next: tamias publish libraries/tardigrada  # ship 9aia:tardigrada@<ver> to the org registry"
