# Wiki

A fast Rust-based command-line tool for searching and retrieving Wikipedia summaries (English and Italian). It selects the best match using fuzzy ranking and prints a cleaned version of the article in the terminal.

---

## Features

- Search English and Italian Wikipedia
- Fuzzy matching using Jaro–Winkler similarity
- Streaming HTTP requests
- Cleans Wikipedia markup, math, and references
- Formatted terminal output with highlighted query matches
- Exit-code based error handling
- Global timeout protection

---

## Installation

```bash
git clone https://github.com/my-test-account-ai/wiki.git
cd wiki
cargo build --release
```

Binary:

```bash
target/release/wiki
```

Optional:

```bash
cargo install --path .
```

---

## Usage

```bash
wiki <query>
```

Example:

```bash
wiki tokyo
wiki artificial intelligence
```

---

## Exit Codes

- 1: Network error / timeout
- 2: JSON/schema error
- 3: Server error
- 4: Not found

---

## Dependencies

- tokio
- reqwest
- serde_json
- regex
- futures-util
- strsim
- urlencoding

---

## License

This project is licensed under the Unlicense: https://unlicense.org/

---

## Note

This README was generated with AI assistance.
