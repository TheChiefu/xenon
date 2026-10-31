# Configuration

Every key in `config.toml` and its default. Missing keys use their default. If
`version` does not match the server's, the server refuses to start.

## `[server]`

| Key | Default | Meaning |
|-|-|-|
| `name` | `Xenon Server` | Server name shown to clients |
| `kind` | `Development` | Type of server, such as `Development` or `Release` |
| `description` | `My custom Xenon server` | Description of the server |
| `log_file` | `xenon.log` | Log file, empty to log to the terminal only |
| `log_level` | `info` | `trace`, `debug`, `info`, `warn` or `error` |

## `[network]`

| Key | Default | Meaning |
|-|-|-|
| `ip` | `127.0.0.1` | Address the server listens on |
| `port` | `3000` | Port the server listens on |
| `certificate` | _unset_ | PEM certificate chain, serves HTTPS when set with `key` |
| `key` | _unset_ | PEM private key for `certificate` |
| `allowed_origins` | `[]` | Origins allowed to make cross-origin requests, `"*"` for any |
| `cors_max_age` | `3600` | Seconds a browser caches a cross-origin preflight |
| `trusted_proxies` | `["127.0.0.1", "::1"]` | Proxies whose `X-Forwarded-For` is read as the client IP |
| `public_url` | _unset_ | This server's public address, required by `[xbox]` and `[steam]` |
| `request_timeout_seconds` | `30` | Seconds a request to Xbox, Steam or a push service waits |

When `ip` is a loopback address, `localhost` and loopback origins are allowed on any port.

## `[files]`

| Key | Default | Meaning |
|-|-|-|
| `path` | `files` | Directory holding uploaded files |
| `file_size_max` | `"25MiB"` | Largest uploaded file, such as `"500KB"` or `"1GiB"` |
| `attachments_per_message_max` | `10` | Most files attached to one message |
| `cache_max_age` | `604800` (7 days) | Seconds a browser caches a downloaded file |

## `[database]`

| Key | Default | Meaning |
|-|-|-|
| `path` | `chat.db` | SQLite database file |
| `max_connections` | `5` | Most connections open at once |
| `min_connections` | `0` | Connections kept open while unused |
| `acquire_timeout_seconds` | `30` | Seconds a request waits for a free connection |
| `idle_timeout_seconds` | `600` | Seconds an unused connection stays open, `0` for never closing |
| `busy_timeout_seconds` | `5` | Seconds a write waits for the database lock |
| `broadcast_retries` | `2` | Retries for a connection when sending live events |
| `broadcast_retry_delay_ms` | `100` | Milliseconds between those retries |

## `[auth]`

| Key | Default | Meaning |
|-|-|-|
| `lifetime_days` | `30` | Days a session lasts without activity |
| `renew_after_days_elapsed` | `1` | Days between session renewals, less than `lifetime_days` |
| `registration_code_required` | `true` | New accounts need a registration code |
| `address_register_failures_max` | `5` | Failed registrations allowed per IP |
| `address_login_failures_max` | `20` | Failed logins allowed per IP |
| `account_login_failures_max` | `10` | Failed logins in a row before an account locks |
| `block_on_register_limit` | `true` | Block an IP at the registration limit |
| `block_on_login_limit` | `true` | Block an IP at the login limit |

## `[lengths]`

| Key | Default |
|-|-|
| `username_min` | `2` |
| `username_max` | `32` |
| `display_name_min` | `1` |
| `display_name_max` | `64` |
| `password_min` | `8` |
| `password_max` | `128` |
| `profile_description_max` | `2000` |
| `room_name_max` | `128` |
| `message_body_max` | `8000` |

## `[responses]`

| Key | Default | Meaning |
|-|-|-|
| `messages_per_page` | `200` | Messages per page |
| `search_results_per_page` | `50` | Search results per page |
| `rooms_per_page` | `200` | Rooms per page |
| `users_per_page` | `25` | Users per page |
| `users_per_lookup` | `500` | Most users looked up in one request |
| `unread_count_max` | `100` | Unread messages counted per room |

## `[sockets]`

| Key | Default | Meaning |
|-|-|-|
| `event_queue_size` | `32` | Live events queued per user |
| `participants_max` | `10` | Most people in a call, at least `2` |
| `reconnect_grace_seconds` | `30` | Seconds a dropped user keeps their place in a call |
| `signal_size_max` | `"16KiB"` | Largest call setup message, at least `"8KiB"` |
| `signals_per_second` | `50` | Call setup messages allowed per user per second |
| `signal_queue_size` | `256` | Call setup messages queued per user |
| `stun_url` | _unset_ | STUN server for calls, as `stun:host:port` |

## Optional services

These sections are never written by the server. Adding one turns that service on.

### `[push]`

| Key | Default | Meaning |
|-|-|-|
| `subject` | _required_ | Contact for push services, a `mailto:` or `https://` address |
| `ttl` | `86400` | Seconds a push service holds a notification for an offline device |

### `[xbox]`

Needs a Microsoft Entra app that accepts personal accounts, with
`<public_url>/xbox/redirect` as its redirect address.

| Key | Default | Meaning |
|-|-|-|
| `client_id` | _required_ | Entra app client ID |
| `client_secret` | _required_ | Entra client secret |
| `game_mappings` | empty | JSON file renaming games and their activity text |
| `poll_interval_seconds` | `60` | Seconds between presence checks |

Game mappings are keyed by the game's name in lowercase with spaces as `_`:

```json
{
    "halo_the_master_chief_collection": {
        "name": "Halo MCC",
        "activity": { "Halo R": "Reach" }
    }
}
```

### `[steam]`

| Key | Default | Meaning |
|-|-|-|
| `web_api_key` | _required_ | Steam Web API key |
| `poll_interval_seconds` | `60` | Seconds between presence checks |
