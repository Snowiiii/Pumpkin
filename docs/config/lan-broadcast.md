# LAN Broadcast
Pumpkin can broadcast the server across the network in order to make it easier for local players to connect to the server easier.

## Configuring LAN Broadcast

#### `enabled`: Boolean
Whether LAN Broadcast is enabled or not.

:::code-group
```toml [features.toml] {2}
[lan_broadcast]
enabled = true
```
:::

#### `motd`: String (optional)
The MOTD to broadcast out to clients. Will use server's MOTD by default.

> [!CAUTION]
> LAN broadcast MOTD does not support multiple lines, RGB colors, or gradients. Pumpkin does not verify the MOTD before broadcasted. If the server MOTD is using these components, consider defining this field so that clients see a proper MOTD.

:::code-group
```toml [features.toml] {3}
[lan_broadcast]
enabled = true
motd = "[your MOTD here]"
```
:::

#### `port`: Integer (0-65535) (optional)
What port to bind to. If not specified, will bind to port 0 (any available port on the system).

> [!IMPORTANT]
> The protocol defines what port to broadcast to. This option only exists to specify which port to bind to on the host. This option purely exists so that the port can be predictable.

:::code-group
```toml [features.toml] {3}
[lan_broadcast]
enabled = true
port = 46733
```
:::

## Default Config
By default LAN broadcast is disabled.

:::code-group
```toml [features.toml]
[lan_broadcast]
enabled = false
motd = "[server MOTD here]"
port = 0
```
:::
