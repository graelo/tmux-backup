#!/bin/bash

set -e

CRATE=tmux-backup
MSRV=1.95

get_rust_version() {
  local array=($(rustc --version));
  echo "${array[1]}";
  return 0;
}
RUST_VERSION=$(get_rust_version)

check_version() {
  IFS=. read -ra rust <<< "$RUST_VERSION"
  IFS=. read -ra want <<< "$1"
  [[ "${rust[0]}" -gt "${want[0]}" ||
   ( "${rust[0]}" -eq "${want[0]}" &&
     "${rust[1]}" -ge "${want[1]}" )
  ]]
}

echo "Testing $CRATE on rustc $RUST_VERSION"
if ! check_version $MSRV ; then
  echo "The minimum for $CRATE is rustc $MSRV"
  exit 1
fi

FEATURES=()
echo "Testing supported features: ${FEATURES[*]}"

set -x

# test the default
cargo build
cargo nextest run

# test `no_std`
cargo build --no-default-features
cargo nextest run --no-default-features

# test each isolated feature, with and without std
for feature in "${FEATURES[@]}"; do
  # cargo build --no-default-features --features="std $feature"
  # cargo nextest run --no-default-features --features="std $feature"

  cargo build --no-default-features --features="$feature"
  cargo nextest run --no-default-features --features="$feature"
done

# test all supported features, with and without std
# cargo build --features="std ${FEATURES[*]}"
# cargo nextest run --features="std ${FEATURES[*]}"

cargo build --features="${FEATURES[*]}"
cargo nextest run --features="${FEATURES[*]}"

# doc tests (not supported by nextest)
cargo test --doc
