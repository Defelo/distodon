set dotenv-load

alias r := run

_default:
    @just --list

run:
    cargo run --locked
