<div align="center">
  <img height="200" alt="cute wiki-tan" src="https://github.com/user-attachments/assets/ade18816-2796-45ff-b85e-ecf763f12b1f" />
  <h1>wiki</h1>
</div>

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
<img align="right" height="250" alt="wiki-tan check's the dependencies=" src="https://github.com/user-attachments/assets/2ebe42ed-bd9f-4ab1-8741-2c23c1711d66" />

- tokio
- reqwest
- serde_json
- regex
- futures-util
- strsim
- urlencoding

---

## Additional Stuff
<img align="left" height="200" alt="wiki-tan reading the Unlicense" src="https://github.com/user-attachments/assets/df13f53b-ac77-43b9-b822-2637ef1c4e7b" />

### License

This project is licensed under the Unlicense: https://unlicense.org/ 

### Note

This README was generated with AI assistance.

<!--- EXCEPT THE WIKITANS POSITIONING UWU --->
