# Authentication

Servers authenthicate with Mojang's session servers in order to ensure the client is playing on a legitmate, paid account. Pumpkin allows you to fully configure authentication.

## Configuring Authentication

> [!WARNING]
> Most servers should not change the default authenthication configuration. Doing so may have unintended consequnces. **Only change these settings if you know what you are doing!**

#### `enabled`: Boolean

Whether authenthication is enabled or not.

:::code-group
```toml [features.toml] {2}
[authentication]
enabled = false
```
:::

#### `prevent_proxy_connections`: Boolean

Whether to block proxy connections or not.

:::code-group
```toml [features.toml] {3}
[authentication]
enabled = true
prevent_proxy_connections = true
```
:::

#### `auth_url`: String (optional)
The URL to authenthicate with. Uses Mojang's session servers to authenthicate if not specified. 

##### Placeholders
| Placeholder     | Description        |
| --------------- | ------------------ |
| `{username}`    | Player username    |
| `{server_hash}` | Hash of the server |

:::code-group
```toml [features.toml] {2}
[authentication]
auth_url = "[custom auth server here]"
```
:::

#### `prevent_proxy_connection_auth_url`: String (optional)
The URL to authenthicate with if `prevent_proxy_connections` is enabled. Uses Mojang's session servers to authenthicate if not specified.

##### Placeholders
| Placeholder     | Description              |
| --------------- | ------------------------ |
| `{username}`    | Player username          |
| `{server_hash}` | Hash of the server       |
| `{ip}`          | IP Address of the player |

:::code-group
```toml [features.toml] {2}
[authentication]
prevent_proxy_connection_auth_url = "[custom auth server here]"
```
:::

### Player Profile

#### `allow_banned_players`: Boolean
Allow players flagged by Mojang.

:::code-group
```toml [features.toml] {2}
[authentication.player_profile]
allow_banned_players = true
```
:::

#### `allowed_actions`: String Array
What actions are allowed if `allow_banned_players` is enabled.

:::code-group
```toml [features.toml] {3}
[authentication.player_profile]
allow_banned_players = true
allowed_actions = ["FORCED_NAME_CHANGE", "USING_BANNED_SKIN"]
```
:::

### Textures

#### `enabled`: Boolean
Whether to filter/validate player textures (e.g. Skins/Capes).

:::code-group
```toml [features.toml] {2}
[authentication.textures]
enabled = true
```
:::

#### `allowed_url_schemes`: String Array
Allowed URL Schemes for textures.

:::code-group
```toml [features.toml] {3}
[authentication.textures]
enabled = true
allowed_url_schemes = ["http", "https"]
```
:::

#### `allowed_url_domains`: String Array
Allowed URL domains for textures.

:::code-group
```toml [features.toml] {3}
[authentication.textures]
enabled = true
allowed_url_domains = [".minecraft.net", ".mojang.com"]
```
:::

### Texture Types

#### `skin`: Boolean
Whether to use player skins or not.

:::code-group
```toml [features.toml] {3}
[authentication.textures.types]
skin = true
```
:::

#### `cape`: Boolean
Whether to use player capes or not.

:::code-group
```toml [features.toml] {3}
[authentication.textures.types]
cape = true
```
:::

#### `elytra`: Boolean
Whether to use player elytras or not.

:::code-group
```toml [features.toml] {3}
[authentication.textures.types]
elytra = true
```
:::

## Default Config
By default, authentication is enabled and uses Mojang's servers. Here is the default config:
:::code-group
```toml [features.toml]
[authentication]
enabled = true
prevent_proxy_connections = false

[authentication.player_profile]
allow_banned_players = false
allowed_actions = ["FORCED_NAME_CHANGE", "USING_BANNED_SKIN"]

[authentication.textures]
enabled = true
allowed_url_schemes = ["http", "https"]
allowed_url_domains = [".minecraft.net", ".mojang.com"]

[authentication.textures.types]
skin = true
cape = true
elytra = true
```
:::
