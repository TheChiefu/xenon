## Code Style
How Xenon code is formatted and organized.

### Comments
This section describes how comments and docstrings are handled in the repo.

Global Rules:
- No periods unless comment is a full sentence in paragraph format
- Prefer present tense imperative such as "check user permission", "start transaction", etc
- Never write "You" or "Your" (ie. directing language at a reader)

#### Inline comments
Put a comment above the code it describes in normal circumstances.
Use short notes inline on enum variants, struct fields, or match arms.

```rust
pub enum Cursor {
    Latest,      // Newest page
    After(i64),  // Everything newer than the given seq
    Before(i64), // Everything older than the given seq
}
```

A comment should describe what the code is doing and how it's modifying data.
Only make comment blocks when a detailed note is needed or it isn't obvious as to what it's doing.

#### Docstrings
Give functions, types, and constants a docstring that explains what they are for and how to use them.
Keep it to a few sentences and leave the details to inline comments that describe themselves.

#### Rust comment syntax
- `//!` for module docstrings
- `///` for docstrings
- `//` for inline comments

Example:
```rust
//! Loading and validating server configuration

/// Load the server configuration from the given path
fn load_config(path: &Path) -> Config {

    // Read the file into a string
    let contents = fs::read_to_string(path).expect("config file not found");

    // Parse the string as TOML
    toml::from_str(&contents).expect("invalid config format")
}
```

#### Section banners
Files are split into sections with banner comments to keep them easy to read.
Data structs come first, then public methods, then helpers/private methods:

```rust
// DATA STRUCTS //
// PUBLIC METHODS //
// HELPER METHODS //
```

Route files use `// ROUTING METHODS //` in place of public methods.
If a file has a distinct group such as shutdown logic give it its own banner.

### Code

#### Formatting
Use the language's standard indentation and bracket style. The Rust code is formatted by hand,
so match the formatting of the code around the change. Do **not** use `rustfmt` or `cargo fmt` since it
would reformat unrelated files.

#### Readability
Code that is easy to read compared to code that is compact (and even if the compact version is faster) is preferred
because code is to be understood not just run. If a closure or long method chain is hard to follow, write it as a loop
or function instead. Logic that needs its own tests should live in its own named function.

#### Rust: ownership and lifetimes
Write lifetime annotations only where the compiler requires them. Prefer taking an owned value, or calling
`.clone()`, when it lets you drop a lifetime annotation.

Note: This is a personal preference as I am not an experienced rust developer (TheChiefu)

### File and module structure

#### Splitting files
Split a file when its name no longer describes everything in it or when it is about 500+ lines long.
For example, `config.rs` became `config/auth.rs`, `config/database.rs`, `config/files.rs`, and so on.

#### Naming
Name files after what they contain and folders after what they do.
Group related files into subfolders with descriptive names.

#### Server layering
The server is split into layers, each layer only calls the components its allowed:

| Layer | Contains | Can Call |
|-|-|-|
| `api/` | Logic and per-table queries | `db` |
| `cli/` | Command-line tools | Anything |
| `routes/` | HTTP handlers | `api/`, `services/`, `rtc/`, `sockets/` |
| `rtc/` | Optional voice and video calls | `api/`, `sockets/` |
| `services/` | Optional integrations (push, account links) | `api/`, `sockets/` |
| `sockets/` | WebSocket messaging and the session registry | — |
| `db.rs` | Queries shared across modules | — |

Notes:
- **API** is independent of routes and the CLI so that both can use it independently
- **CLI** is the exception and may use any layer or run its own queries
  - It is assumed that anyone with CLI has the necessary permission that routes cannot ensure
- **db.rs** is the bottom most layer and never calls up
  - Queries in `db.rs` take a connection, so a caller can run several of them in one transaction
