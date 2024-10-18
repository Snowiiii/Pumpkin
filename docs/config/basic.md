# Basic Configuration

Representing `configuration.toml`

## Server Address

The address to bind the server to

```toml
server_address=0.0.0.0
```

## Seed

The seed for world generation

```toml
seed=
```

## Max players

The maximum number of players allowed on the server

```toml
max_players=10000
```

## View distance

The maximum view distance for players

```toml
view_distance=10
```

## Simulation distance

The maximum simulation distance for players

```toml
simulation_distance=10
```

## Default difficulty

The default game difficulty

```toml
default_difficulty=Normal
```

```toml
Peaceful
Easy
Normal
Hard
```

## Allow nether

Whether the Nether dimension is enabled

```toml
allow_nether=true
```

## Hardcore

Whether the server is in hardcore mode.

```toml
hardcore=true
```

## Online Mode

Whether online mode is enabled. Requires valid Minecraft accounts

```toml
online_mode=true
```

## Encryption

Whether packet encryption is enabled

> [!IMPORTANT]
> Required when online mode is enabled

```toml
encryption=true
```

## Motd

The server's description displayed on the status screen.

```toml
motd=true
```

## Favicon

The path to the server's favicon

```toml
favicon_path=
```

## Default gamemode

The default game mode for players

```toml
default_gamemode=Survival
```

```toml
Undefined
Survival
Creative
Adventure
Spectator
```

## IP Scrubbing

Whether to scrub player IPs from logs

```toml
scrub_ips=true
```
