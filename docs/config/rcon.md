# RCON
RCON is a protocol that allows you to remotely manage the server from a different device. Pumpkin has full support for RCON.

## Configuring RCON

#### `enabled`: Boolean

:::code-group
```toml [features.toml] {2}
[rcon]
enabled = true
```
:::

#### `address`: String
The address and port that RCON should listen to.

:::code-group
```toml [features.toml] {3}
[rcon]
enabled = true
address = "0.0.0.0:25575"
```
:::

#### `password`: String
The password to use for RCON authentication.

:::code-group
```toml [features.toml] {3}
[rcon]
enabled = true
password = "[your safe password here]"
```
:::

#### `max_connections`: Integer
The max number of RCON connections allowed at a single time. Set to 0 to disable a limit.

:::code-group
```toml [features.toml] {3}
[rcon]
enabled = true
max_connections = 5
```
:::

### Logging
#### `log_logged_successfully`: Boolean
Weather successful logins should be logged to console or not.

:::code-group
```toml [features.toml] {2}
[rcon.logging]
log_logged_successfully = true
```
:::

#### `log_wrong_password`: Boolean
Weather wrong password attempts should be logged to console or not.

:::code-group
```toml [features.toml] {2}
[rcon.logging]
log_logged_successfully = true
```
:::

#### `log_commands`: Boolean
Weather to log commands ran from RCON to console or not.

:::code-group
```toml [features.toml] {2}
[rcon.logging]
log_commands = true
```
:::

#### `log_quit`: Boolean
Whether RCON client quit should be logged or not.

:::code-group
```toml [features.toml] {2}
[rcon.logging]
log_quit = true
```
:::

## Default Config
By default RCON is disabled.

:::code-group
```toml [features.toml]
[rcon]
enabled = false
address = "0.0.0.0:25575"
password = ""
max_connections = 0

[rcon.logging]
log_logged_successfully = true
log_wrong_password = true
log_commands = true
log_quit = true
```
:::
