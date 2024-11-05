# Advanced Configuration

## Proxy

`proxy`

Whether Proxy Configuration is Enabled

```toml
enabled=false
```

### Velocity

`proxy.velocity`

Whether [Velocity](https://papermc.io/software/velocity) Proxy is enabled

```toml
enabled=false
```

#### Velocity Secret

This secret is used to ensure that player info forwarded by Velocity comes from your proxy and not from someone pretending to run Velocity

```toml
secret=
```

### Bungeecord
`proxy.bungeecord`

```toml
enabled=false
```

## Authentication

`authentication`

Whether Authentication is enabled

```toml
enabled=false
```

### Authentication URL

The Authentication URL being used

> [!IMPORTANT]
> {username} | The Username from the requested player
>
> {server_hash} | The SHA1 Encrypted hash

```toml
auth_url="https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}"
```

### Prevent Proxy Connections

Prevent proxy connections

```toml
prevent_proxy_connections=false
```

### Prevent Proxy Connections URL

The Authentication URL being used

> [!IMPORTANT]
> {username} | The Username from the requested player
>
> {server_hash} | The SHA1 Encrypted hash
>
> {ip} | The IP of the requested Player

```toml
prevent_proxy_connection_auth_url = "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={server_hash}&ip={ip}"
```

### Player Profile

`authentication.player_profile`

#### Allow Banned Players

Allow players flagged by Mojang (banned, forced name change)

```toml
allow_banned_players=false
```

#### Allowed Actions

Depends on the value above

```toml
allowed_actions=["FORCED_NAME_CHANGE", "USING_BANNED_SKIN"]
```

```toml
FORCED_NAME_CHANGE
USING_BANNED_SKIN
```

### Textures

`authentication.textures`

Whether to filter/validate player textures (e.g. Skins/Capes)

```toml
enabled=true
```

#### Allowed URL Schemes

Allowed URL Schemes for Textures

```toml
allowed_url_schemes=["http", "https"]
```

#### Allowed URL Domains

Allowed URL domains for Textures

```toml
allowed_url_domains=[".minecraft.net", ".mojang.com"]
```

### Texture Types

`authentication.textures.types`

#### Skin

Use player skins

```toml
skin=true
```

#### Cape

Use player capes

```toml
cape=true
```

#### Elytra

Use player elytras
(I didn't know myself that there are custom elytras)

```toml
elytra=true
```

## Compression

`packet_compression`

Whether Packet Compression is enabled

```toml
enable=true
```

### Compression Info

#### Threshold

The compression threshold is used when compression is enabled

```toml
threshold=256
```

#### Level

The Compression Level

> [!IMPORTANT]
> A value between 0..9
>
> 1 = Optimize for the best speed of encoding.
>
> 9 = Optimize for the size of data being encoded.

```toml
level=4
```

## Resource Pack

`resource_pack`

Whether a Resource Pack is enabled

```toml
enable=false
```

### Resource Pack URL

The download URL of the resource pack

```toml
resource_pack_url=
```

### Resource Pack SHA1

The SHA1 hash (40) of the resource pack

```toml
resource_pack_sha1=
```

### Prompt Message

Custom prompt Text component, Leave blank for none

```toml
prompt_message=
```

### Force

Will force the Player to accept the resource pack

```toml
force=false
```

## Commands

`commands`

### Use Console

Are commands from the Console accepted

```toml
use_console=true
```

### Log Console

Should commands from players be logged into the console?

```toml
log_console=true
```

## RCON Config

`rcon`

Whether RCON is enabled

```toml
enable=false
```

### Address

The network address and port where the RCON server will listen for connections

```toml
address=false
```

### Password

The password required for RCON authentication

```toml
password=
```

### Maximum Connections

The maximum number of concurrent RCON connections allowed

If 0 there is no limit

```toml
max_connections=0
```

### RCON Logging

`rcon.logging`

#### Logged Successfully

Whether successful RCON logins should be logged

```toml
log_logged_successfully=true
```

#### Wrong Password

Whether failed RCON login attempts with incorrect passwords should be logged

```toml
log_wrong_password=true
```

#### Commands

Whether all RCON commands, regardless of success or failure, should be logged

```toml
log_commands=true
```

#### Disconnect

Whether RCON client quit should be logged

```toml
log_quit=true
```

## PVP

`pvp`

Whether PVP is enabled

```toml
enable=true
```

### Hurt Animation

Do we want to have the Red hurt animation & fov bobbing

```toml
hurt_animation=true
```

### Protect Creative

Should players in creative be protected against PVP

```toml
protect_creative=true
```

### Knockback

Has PVP Knockback (Velocity)

```toml
knockback=true
```

### Swing

Should players swing when attacking?

```toml
swing=true
```

## Logging

`logging`
Whether Logging is enabled

```toml
enable=true
```

### Level

At which level should be logged

```toml
level=Info
```

```toml
Off
Error
Warn
Info
Debug
Trace
```

### Env

Enables the user to choose log level by setting the `RUST_LOG=<level>` environment variable

```toml
env=false
```

### Threads

Should threads be printed in the message

```toml
threads=true
```

### Color

Should color be enabled for logging messages

```toml
color=true
```

### Timestamp

Should the timestamp be printed in the message

```toml
timestamp=true
```

## Query

### Enabled

Should clients be able to query the server for info, using the query protocol?

```toml
enabled=true
```

### Port (optional)

If enabled, what port should the server listen to for query requests?

```toml
# By default query will listen on the same port the server is running on
port=25565
```
