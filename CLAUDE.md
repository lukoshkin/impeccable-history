# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`impeccable-history` is a ZSH plugin that removes failed commands from shell history. It tracks commands with non-zero exit codes during a session and filters them out of `$HISTFILE` on shell exit using a Rust-based CLI tool.

## Build Commands

```bash
# Build the Rust binary
cargo build --release

# Run tests
cargo test

# The compiled binary should be placed in bin/
cp target/release/hist-scraper bin/
```

## Architecture

### Components

1. **`hist-scraper.plugin.zsh`** - ZSH plugin that:
   - Sets history options (`histignorespace`, `histreduceblanks`, `histignorealldups`, `histnostore`)
   - Registers three ZSH hooks:
     - `_zsh_add_history` (zshaddhistory) - filters commands matching `HIST_SCRAPER_IGNORE` before adding to history
     - `_add_broken_cmd` (precmd) - logs failed commands to `/tmp/hist-scraper-nzcmds.txt`
     - `_scrape_history` (zshexit) - invokes the Rust binary to filter `$HISTFILE` on exit

2. **`src/hist-scraper.rs`** - Rust CLI tool that filters a target file based on patterns from a query file
   - Uses `clap` for argument parsing
   - Key args: `-t/--target` (file to filter), `-q/--query` (patterns file), `-n/--skip-n` (lines to skip), `--in-place`
   - The `stringfix!` macro handles string slicing for extracting commands from the query format

### Data Flow

1. During shell session: failed commands (exit code + command) are appended to `/tmp/hist-scraper-nzcmds.txt`
2. On shell exit: `hist-scraper` reads `$HISTFILE`, skips first N lines (persisted in `skip_num`), and removes matching failed commands
3. Line count is saved to `skip_num` for the next session to avoid re-processing old history

### Environment Variables

- `HIST_SCRAPER_IGNORE` - Regex pattern for commands to exclude from history immediately (default: `^.{1,3}$`)
- `HISTORY_IGNORE` - ZSH variable for patterns to remove on logout (separate from plugin)

### Temporary Files

- `/tmp/hist-scraper-nzcmds.txt` - Log of failed commands (format: `exit_code command`)
- `/tmp/hist-scraper-error.log` - Error log from hist-scraper execution
- `skip_num` - Persists line count between sessions

## Known Limitation

The plugin currently has issues with non-ASCII characters in the ZSH history file due to metafied format handling.
