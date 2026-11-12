# Commands

Commands are typed into the terminal the server is running in.

## First admin

A new server has no accounts. Create one and make it an admin:
```
user create <username>
user role <username> admin
```
`user create` prints the account's password once. New accounts are Visitors until given a role.

## All commands

| Command | Description |
|-|-|
| `user create <username>` | Create an account and print its password |
| `user role <username> <role>` | Set an account's role: `admin`, `member`, or `visitor` |
| `user reset <username>` | Replace an account's password and print the new one |
| `user unlock <username>` | Let an account locked by failed logins log in again |
| `user get <username>` | Print an account's details |
| `user ban <username>` | Ban an account and sign it out |
| `user unban <username>` | Unban an account |
| `user delete <username> [--messages]` | Delete an account, and with `--messages` every message it wrote |
| `files clean [-t <threads>]` | Delete stored files nothing uses, on every thread unless `-t` is given |
| `ip list` | List blocked addresses |
| `ip block <address>` | Block an address |
| `ip unblock <address>` | Unblock an address |
| `stop`, `exit` | Shut down the server |
