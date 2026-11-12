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
cd crates/server
cargo build --release --features full
```

Each service is a feature, and none are enabled by default:

| Feature | Service |
|-|-|
| `push` | Web push notifications |
| `links` | Xbox and Steam account linking |
| `rtc` | Voice calls |
| `full` | All of the above |

To update the web TypeScript bindings based off of the rust code, run this from
the `crates/types` directory:
```bash
cargo bindings
```

## Configuration
To configure the server, edit the `config.toml` file that is generated when the server
is first run. By default it is written to the same directory the server is run at.
To change this set the `XENON_CONFIG` environment variable to a path.

To see more information on the configuration file and its options, see here: [crates/server/config.md](crates/server/config.md).

## Commands
To see the commands typed into the server's terminal, including setting up the first
admin account, see here: [crates/server/commands.md](crates/server/commands.md).

## API
The server exposes a REST API along with a WebSocket that delivers live events.

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
Agents such as Claude and Qwen (open source versions) were used in the creation of this repo.
It is advised anyone reading the repo to vet the source code before blindly trusting it.

It was not completely vibe coded where agents were left to create all the code on their own unmonitored.
Days/weeks have been spent reviewing, reiterating, critiquing, and manually updating code in order
to reduce excessive bloat. Rewrites and reordering is likely to continue throughout until
a 1.0 release is set.

This project was primarily to move away from corporate communication software and onto something locally
hosted and something with features particularlly relevant to my (and my friends') needs. So expect it to be
largely bloated early, then as time goes on, iterated, reduced, and refined.

## License:
TBD / Expect compeltely free usage one. Cannot be decided until project is mostly complete as external
libraries are used and their licenses need to be vetted.
