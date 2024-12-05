# Logging
Pumpkin allows you to customize what you want in your logs.

## Configuring Logging

#### `enabled`: Boolean
Whether logging is enabled or not.

:::code-group
```toml [features.toml] {2}
[logging]
enabled = true
```
:::

#### `level`: Enum
What should be logged. Possible values are:
- Off
- Error
- Warn
- Info
- Debug
- Trace

:::code-group
```toml [features.toml] {3}
[logging]
enabled = true
level = "Debug"
```
:::

#### `env`: Boolean
Whether to allow choosing the log level by setting the `RUST_LOG` environment variable or not.

:::code-group
```toml [features.toml] {3}
[logging]
enabled = true
env = true
```
:::

#### `threads`: Boolean
Whether to print threads in the logging message or not.

:::code-group
```toml [features.toml] {3}
[logging]
enabled = true
threads = false
```
:::

#### `color`: Boolean
Whether to print with color to the console or not.

:::code-group
```toml [features.toml] {3}
[logging]
enabled = true
color = false
```
:::

#### `timestamp`: Boolean
Whether to print the timestamp in the meessage or not.

:::code-group
```toml [features.toml] {3}
[logging]
enabled = true
timestamp = false
```
:::

## Default Config
By default, logging is enabled and will print with color, threads, and timestamp at the `Info` level. 

:::code-group
```toml [features.toml]
[logging]
enabled = true
level = "Info"
env = false
threads = true
color = true
timestamp = true
```
:::
