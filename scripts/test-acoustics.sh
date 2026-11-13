#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
mod="$root/mods/horizon-acoustics"
scratch="$(mktemp -d)"
trap 'rm -rf -- "$scratch"' EXIT
# On macOS, TMPDIR can start with /var, a symlink the staging tool rejects.
scratch="$(cd "$scratch" && pwd -P)"

read -r horizon mixins rust < <(python3 - "$mod/mod.toml" <<'PY'
import sys
import tomllib

with open(sys.argv[1], "rb") as file:
    metadata = tomllib.load(file)
print(metadata["horizon"], metadata["horizon_mixins"], metadata["rust"])
PY
)

rustup toolchain install "$rust" --profile minimal --component rustfmt
export RUSTUP_TOOLCHAIN="$rust"
rustfmt --edition 2024 --check "$mod"/mixins/horizon_acoustics/*.rs

git init -q "$scratch/horizon"
git -C "$scratch/horizon" fetch --depth 1 https://github.com/horizon-rust/horizon.git "$horizon"
git -C "$scratch/horizon" checkout -q --detach FETCH_HEAD
# Git checks out the documentation alias as a symlink, which mixins cannot stage.
if [[ -L "$scratch/horizon/CLAUDE.md" ]]; then
    rm -- "$scratch/horizon/CLAUDE.md"
fi
git -C "$scratch/horizon" apply "$mod/audio-host.patch"

cargo install --locked --root "$scratch/tools" --git https://github.com/horizon-rust/horizon-mixins --rev "$mixins" horizon-mixins-cli
cargo fetch --locked --manifest-path "$scratch/horizon/Cargo.toml" --target aarch64-apple-darwin
"$scratch/tools/bin/horizon-mixins" apply --workspace "$scratch/horizon" --mods "$mod/mixins" --out "$scratch/staged"
cd "$scratch/staged"
cargo test --locked --profile fast -p horizon-client-render --features audio --lib audio::
cargo check --locked -p horizon-client --bin horizon-client
