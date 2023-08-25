# File:        Justfile
# Author:      Zakhary Kaplan <https://zakhary.dev>
# Created:     27 Apr 2022
# SPDX-License-Identifier: MIT OR Apache-2.0
# Vim:         set fdl=0 fdm=marker ft=make:

alias b := build
alias c := check
alias r := run
alias t := test

# default recipe
_: help

# build all artifacts
all: build doc release

# compile local package
build: dev

# check local package for errors
check:
    @cargo check --workspace --all-targets

# clean build artifacts
clean:
    @cargo clean

# build `dev` profile
dev:
    @cargo build --all-targets

# apply lints
fix: && fmt
    @cargo clippy --workspace --fix --allow-dirty --allow-staged

# format source files
fmt:
    @cargo +nightly fmt

# document source files
doc:
    @cargo doc

# list available recipes
help:
    @just --list

# lint source files
lint:
    @cargo clippy --workspace --all-targets

# lint pedantically
pedantic:
    @cargo clippy --workspace --all-targets -- --warn=clippy::pedantic

# build `release` profile
release:
    @cargo build --all-targets --release

# run binary
run package *opts:
    @cargo run --release -p {{ package }} -- {{ opts }}

# perform tests
test *opts:
    @cargo test --workspace -- {{ opts }}
