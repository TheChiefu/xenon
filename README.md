![project logo](./logo.png)

## Description
A simple chat application for use in small to medium sized groups. It is a single
server instance with public and hidden rooms. Unlike discord clones,
this is not an ecosystem server but a single server with rooms acting
as separators for groups.

The intent behind this project is a simple to deploy chat app between friends.
Which is why SQLite was chosen for making database management simple
(not requiring an active service like Postgres to run).

### Summary of Server
The room types are as follows:
- Public: Open to anyone on the server
- Hidden: Non-discoverable and require an invite
  - They can function as a DM (ie. 1-to-1 room between two users)

Each room sets its own permissions for members, for example:
- Post
- Attach (files)
- Create Invites
- Rename Room
- Etc.

The primary form of communication is via messages. A message can be plain text,
links, file attachments, etc. Attachments can have a size and count limit
(both of which are set in the server config). See the [configuration](#configuration) to learn more.

User accounts can be one of the following global roles:
- Admin: Can kick users and delete messages from public room
- Member: Regular permissions (file upload, create rooms, etc)
- Visitor: Can join and post in rooms (cannot upload files nor create rooms)
  - New users are Visitors by default

## Building
Requires Rust 1.85 or newer (this crate uses edition 2024). Distribution packages
are often older than that, so install via: [rustup](https://rustup.rs) rather
than a package manager.

SQLite and the TLS library are compiled from source, so a C compiler and cmake
are also needed.

Arch:
```bash
sudo pacman -S base-devel cmake
```

Debian and Ubuntu:
```bash
sudo apt install build-essential cmake
```

Windows:
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
winget install Kitware.CMake
winget install NASM.NASM
```

Then build:
```bash
cd server
cargo build --release
```

To update the client TypeScript bindings based off of the rust code, run this from
the `server` directory:
```bash
TS_RS_EXPORT_DIR=../client/src/bindings TS_RS_LARGE_INT=number cargo test --features ts_bindings export_bindings
```

Both variables are required. `TS_RS_EXPORT_DIR` is where the files are written, which
otherwise defaults to `server/bindings`. `TS_RS_LARGE_INT` writes 64 bit integers as
`number`, which otherwise defaults to `bigint`.

## Configuration
To configure the server, edit the `config.toml` file that is generated when the server
is first run. By default it is written to the same directory the server is run at.
To change this set the `XENON_CONFIG` environment variable to a path.

To see more information on the configuration file and its options, see here: [server/config.md](server/config.md).

## Commands
To see the commands typed into the server's terminal, including setting up the first
admin account, see here: [server/commands.md](server/commands.md).

## Optional Services
Each is turned on by adding its section to the config file:

- Web push notifications (so closed mobile clients are told about new messages)
- Xbox account linking and game presence
- Steam account linking and game presence

## API
The server exposes a REST API along with a WebSocket that delivers live events.

## Basic Security
|Category| Method | Description |
|-|-|-|
| Invite Codes (12 Characters) | OsRng | Invite is required by an Admin to enter server |
| Password Hashing| Argon2id | Prevents password snooping (no plain-text) |
| Session Tokens | OsRng + SHA256 | Randomly generated token per login session |

## Developer Comment(s):
I am not a database, server, or security developer; so have no expectation of a
professional grade chat application. Although I work in software development I
primarily work on the GUI side (Qt) and video game space.

My plan for this project is something very easy to setup, deploy, and maintain.
The only thing required to get it running is compiling the program and running the
executable. The config file, file upload path, database file, and defaults are all
setup on first run.

_Set expectations accordingly_

## AI Usage:
Server code I write myself and use AI for adding/modifying snippets of code that match the
existing codebase. I keep AI usage minimal here since it's core infrastructure code.
It is also used for re-writing docstrings and comments in bulk.

## License:
TBD
