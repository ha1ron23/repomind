# repomind – codebase navigation with AI

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**repomind** indexes symbols from Rust, Python, Go, and JavaScript (extensible), then lets you instantly find or explain them using a local LLM.

## Features

- Index functions, classes, methods, structs
- Fast symbol lookup via SQLite
- AI explanation of any symbol using Ollama (offline, free)
- Easy to add more languages (tree-sitter based)

## Requirements
- **Rust 1.70+**
- **Ollama (if you want to use "repomind explain")**

## Install

```bash
git clone https://github.com/ha1ron23/repomind.git
cd repomind
cargo build --release
sudo cp target/release/repomind /usr/local/bin/
```

## Usage
```bash
# Index your project
repomind index .

# Find a symbol
repomind find read_file

# Explain a symbol (Ollama must be running)
export OLLAMA_URL=http://localhost:11434
export OLLAMA_MODEL=llama3.2:1b
repomind explain read_file
```

## License
MIT License
