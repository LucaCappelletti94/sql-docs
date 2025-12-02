#!/usr/bin/env bash
set -u
set -o pipefail

# Mirror of GitHub Actions Rust CI, for local use.
# Runs:
# - cargo test (debug + release) on stable/beta/nightly (if installed)
# - cargo fmt (nightly, --check)
# - taplo fmt --check
# - cargo clippy --all-targets --all-features -D warnings
# - cargo doc with RUSTDOCFLAGS=-D warnings
# - cargo tarpaulin (coverage) for package sql_docs
#
# Only prints full command output when a step fails.

CARGO_TERM_COLOR=always
export CARGO_TERM_COLOR

LOG_DIR="$(mktemp -d -t rust-ci-logs-XXXXXX)"
failed=0
step_index=0

cleanup() {
  rm -rf "$LOG_DIR" 2>/dev/null || true
}
trap cleanup EXIT

run_step() {
  local name="$1"
  shift
  step_index=$((step_index + 1))
  local log="$LOG_DIR/${step_index}_${name//[^A-Za-z0-9_.-]/_}.log"

  # Minimal noise on success: just a single line.
  echo "==> $name"
  if "$@" >"$log" 2>&1; then
    echo "    OK"
    rm -f "$log"
  else
    echo "    FAILED"
    echo "---- $name log ----"
    sed 's/^/    /' "$log"
    echo "-------------------"
    failed=1
  fi
}

# --- Sanity checks ---------------------------------------------------------

if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup is required to run this script."
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to run this script."
  exit 1
fi

# --- Test matrix: stable, beta, nightly -----------------------------------

for TOOLCHAIN in stable beta nightly; do
  if rustup toolchain list | grep -q "^$TOOLCHAIN"; then
    run_step "tests-${TOOLCHAIN}-debug"   rustup run "$TOOLCHAIN" cargo test --verbose
    run_step "tests-${TOOLCHAIN}-release" rustup run "$TOOLCHAIN" cargo test --release --verbose
  else
    echo "==> Skipping tests for $TOOLCHAIN (toolchain not installed)"
  fi
done

# --- Rustfmt (nightly) -----------------------------------------------------

if rustup toolchain list | grep -q "^nightly"; then
  run_step "rustfmt-install-nightly" rustup component add --toolchain nightly rustfmt
  run_step "rustfmt-check"           rustup run nightly cargo fmt --all -- --check
else
  echo "==> Skipping rustfmt check (nightly toolchain not installed)"
fi

# --- Taplo (TOML formatting) ----------------------------------------------

if ! command -v taplo >/dev/null 2>&1; then
  # mirror GH action behavior: install taplo if missing
  run_step "install-taplo-cli" cargo install taplo-cli
fi

if command -v taplo >/dev/null 2>&1; then
  run_step "taplo-fmt-check" taplo fmt --check
else
  echo "==> Skipping taplo fmt check (taplo not available and install failed)"
fi

# --- Clippy (stable) -------------------------------------------------------

if rustup toolchain list | grep -q "^stable"; then
  run_step "clippy-install" rustup component add --toolchain stable clippy
  run_step "clippy-check"   rustup run stable cargo clippy --all-targets --all-features -- -D warnings
else
  echo "==> Skipping clippy (stable toolchain not installed)"
fi

# --- Docs (stable) ---------------------------------------------------------

if rustup toolchain list | grep -q "^stable"; then
  run_step "doc-check" env RUSTDOCFLAGS="-D warnings" rustup run stable cargo doc --no-deps --document-private-items
else
  echo "==> Skipping doc check (stable toolchain not installed)"
fi

# --- Coverage (tarpaulin) --------------------------------------------------

if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
  run_step "install-tarpaulin" cargo install cargo-tarpaulin
fi

if command -v cargo-tarpaulin >/dev/null 2>&1; then
  run_step "coverage-tarpaulin" cargo tarpaulin --verbose --engine=llvm --all-features --timeout 120 --out xml -p sql_docs
else
  echo "==> Skipping coverage (cargo-tarpaulin not available and install failed)"
fi

# --- Summary ---------------------------------------------------------------

if [[ "$failed" -ne 0 ]]; then
  echo
  echo "❌ One or more steps failed."
else
  echo
  echo "✅ All steps passed."
fi

exit "$failed"
