#!/usr/bin/env bash
# Stage the tamias pack payload: release-build `tardigrada` and copy
# it into `libraries/tardigrada/tools/bin/<host-triple>/` (the
# `payload:` bin dir — gitignored, a build artifact). Then
# `tamias install ./libraries/tardigrada` deploys the `tardigrada`
# command onto PATH.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

cargo build --release -p tardigrada

triple="$(rustc -vV | sed -n 's/^host: //p')"
dest="libraries/tardigrada/tools/bin/$triple"
mkdir -p "$dest"
cp "target/release/tardigrada" "$dest/tardigrada"

echo "pack staged: $dest/tardigrada"
echo
echo "install the tardigrada command (payload bin):"
echo "  tamias install ./libraries/tardigrada"
echo "  # or a pinned tarball: tamias bundle pack libraries/tardigrada"
echo "dev install without tamias:"
echo "  cargo install --path crates/tardigrada"
