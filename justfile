# THIS FILE IS AUTOGENERATED FROM FLAKEBOX CONFIGURATION

alias b := build
alias c := check
alias t := test


[private]
default:
  @just --list


# run `cargo build` on everything
build *ARGS="--workspace --all-targets":
  #!/usr/bin/env bash
  set -euo pipefail
  if [ ! -f Cargo.toml ]; then
    cd {{invocation_directory()}}
  fi
  cargo build {{ARGS}}


# run `cargo check` on everything
check *ARGS="--workspace --all-targets":
  #!/usr/bin/env bash
  set -euo pipefail
  if [ ! -f Cargo.toml ]; then
    cd {{invocation_directory()}}
  fi
  cargo check {{ARGS}}


# run all checks recommended before opening a PR
final-check: clippy
  #!/usr/bin/env bash
  set -euo pipefail
  if [ ! -f Cargo.toml ]; then
    cd {{invocation_directory()}}
  fi
  cargo test --doc
  just run-checks 'all'


# run code formatters
format:
  #!/usr/bin/env bash
  set -euo pipefail
  if [ ! -f Cargo.toml ]; then
    cd {{invocation_directory()}}
  fi
  cargo fmt --all
  nixpkgs-fmt $(echo **.nix)


# run lints (git pre-commit hook)
lint:
  #!/usr/bin/env bash
  set -euo pipefail
  env NO_STASH=true $(git rev-parse --git-common-dir)/hooks/pre-commit


# run tests
test: build
  #!/usr/bin/env bash
  set -euo pipefail
  if [ ! -f Cargo.toml ]; then
    cd {{invocation_directory()}}
  fi
  cargo test


# run and restart on changes
watch *ARGS="-x run":
  #!/usr/bin/env bash
  set -euo pipefail
  if [ ! -f Cargo.toml ]; then
    cd {{invocation_directory()}}
  fi
  env RUST_LOG=${RUST_LOG:-debug} cargo watch {{ARGS}}


# run `cargo clippy` on everything
clippy *ARGS="--locked --offline --workspace --all-targets":
  cargo clippy {{ARGS}} -- --deny warnings --allow deprecated

# run `cargo clippy --fix` on everything
clippy-fix *ARGS="--locked --offline --workspace --all-targets":
  cargo clippy {{ARGS}} --fix


# run `semgrep`
semgrep:
  env SEMGREP_ENABLE_VERSION_CHECK=0 \
    semgrep --error --no-rewrite-rule-ids --config .config/semgrep.yaml


# check typos
[no-exit-message]
typos *PARAMS:
  #!/usr/bin/env bash
  set -eo pipefail

  export FLAKEBOX_GIT_LS
  FLAKEBOX_GIT_LS="$(git ls-files)"
  export FLAKEBOX_GIT_LS_TEXT
  FLAKEBOX_GIT_LS_TEXT="$(echo "$FLAKEBOX_GIT_LS" | grep -v -E "^db/|\.(png|ods|jpg|jpeg|woff2|keystore|wasm|ttf|jar|ico)\$")"


  if ! echo "$FLAKEBOX_GIT_LS_TEXT" | typos {{PARAMS}} --file-list - --force-exclude ; then
    >&2 echo "Typos found: Valid new words can be added to '.typos.toml'"
    return 1
  fi

# fix all typos
[no-exit-message]
typos-fix-all:
  just typos -w

# THIS FILE IS AUTOGENERATED FROM FLAKEBOX CONFIGURATION
