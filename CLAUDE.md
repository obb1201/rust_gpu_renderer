# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project goal

The owner is learning Rust and building a GPU renderer in it. Treat this as a learning project: when introducing Rust concepts (ownership, lifetimes, traits, etc.) or GPU concepts, briefly explain the why, and prefer simple, idiomatic code over clever abstractions.

## Working rules

- Write in clear, simple English, in code comments, docs and replies.
- If a requirement is unclear, ask the user key questions before proceeding.
- Record changes in the change log in the `docs/` folder (`docs/CHANGELOG.md`). Add an entry whenever you make a notable change.

## Current state

The repository is empty: no Cargo project, no source, no git history yet. Rust (`cargo`/`rustc`) was not found on PATH when this file was created (Windows 11, PowerShell/Git Bash). Install via https://rustup.rs before building.

## Commands

Not yet applicable. Once `cargo init` has been run, the standard commands are `cargo build`, `cargo run`, `cargo test`, `cargo test <name>` (single test), `cargo clippy`, `cargo fmt`.

## Architecture

To be filled in once the renderer takes shape. Key decisions still open and worth recording here when made: GPU API/crate choice (e.g. wgpu vs. ash/Vulkan), windowing crate (e.g. winit), and shader language (e.g. WGSL).
