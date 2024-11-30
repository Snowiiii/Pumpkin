# Proxy
Many servers use proxies to manage connections and distribute players across servers. Pumpkin supports the following proxy protocols:

- [Velocity](https://papermc.io/software/velocity)
- [BungeeCord](https://www.spigotmc.org/wiki/bungeecord-installation/)

> [!TIP]
> Velocity is recommended for most server networks. Velocity is modern and more performant compared to BungeeCord.

## Configuring Proxy

#### `enabled`: Boolean

Enables support for proxies.

:::code-group
```toml [features.toml]{2}
[proxy]
enabled = true
```
:::

### Velocity

#### `enabled`: Boolean

Weather Velocity support is enabled or not.

:::code-group
```toml [features.toml]{2}
[proxy.velocity]
enabled = true
```
:::

#### `secret`: String 

The secret as configured in Velocity. 

:::code-group
```toml [features.toml]{3}
[proxy.velocity]
enabled = true
secret = "[proxy secret here]"
```
:::

### BungeeCord

#### `enabled`: Boolean
Weather BungeeCord support is enabled or not.

:::code-group
```toml [features.toml]{2}
[proxy.bungeecord]
enabled = true
```
:::

> [!CAUTION]
> Ensure that the server's firewall is correctly configured, as BungeeCord can't verify if player info is from your proxy or an imposter.

## Default Config
By default, proxy support is disabled. Here is the default config:

:::code-group
```toml [features.toml]
[proxy]
enabled = false

[proxy.velocity]
enabled = false
secret = ""

[proxy.bungeecord]
enabled = false
```
:::
