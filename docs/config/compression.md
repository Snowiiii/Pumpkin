# Compression
Compression is used to reduce the size of packets. This is benefical to reduce bandwidth server side and also help players on slower internet connections.

## Configuring Compression

### `enabled`
Weather packet compression is enabled or not.

> [!TIP]
> It might be benefical to disable compression if the server is behind a proxy.

:::code-group
```toml [features.toml] {2}
[packet_compression]
enabled = true
```
:::

### `threshold`

The minimum packet size before the server attempts to compress the packet.

> [!CAUTION]
> Increasing this value can hurt players on slower connections.

:::code-group
```toml [features.toml] {2}
[packet_compression]
threshold = 256
```
:::

### `level`

A value between 0 to 9: 0 to disable compression, 1 being the fastest compression (at the cost of size), and 9 being maximum compression (at the cost of speed).

:::code-group
```toml [features.toml] {2}
[packet_compression]
level = 4
```
:::

## Default Config

By default, compression is enabled.

:::code-group
```toml [features.toml]
[packet_compression]
enabled = true
threshold = 256
level = 4
```
:::
