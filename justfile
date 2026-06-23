#!/usr/bin/env -S just --justfile

set shell := ["sh", "-cu"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

[private]
default:
    @just --list

# Just set the `main` bookmark to `@-` and push it to origin
[group: 'jj']
jjship r='@-':
    jj bookmark set -r '{{r}}' main
    jj git push -b main

# Just jj commit -m "{{msg}}"
[group: 'jj']
jjc msg:
    jj commit -m "{{msg}}"

# Just print jj log
[group: 'jj']
jjl:
    jj log

# Just lint = fmt + clippy
[group: 'lint']
lint:
    cargo fmt
    cargo clippy --fix --allow-dirty

# Just test = nextest
[group: 'test']
test:
    cargo nextest run --all-features --profile default --no-tests pass
