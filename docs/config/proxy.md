# Proxy
Many servers use proxies to manage connections and distribute players across servers. Pumpkin supports the following proxy protocols:

- [Velocity](https://papermc.io/software/velocity)
- [BungeeCord](https://www.spigotmc.org/wiki/bungeecord-installation/)

> [!TIP]
> Velocity is recommended for most server networks. Velocity is modern and more performant compared to BungeeCord.

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

## Configuring Proxy
To enable proxy support, set `enabled` to true:

:::code-group
```toml [features.toml]{2}
[proxy]
enabled = true
```
:::

Then enable the respective proxy protocol you wish to use.

### Velocity

For Velocity, set `enabled` to true and set `secret` to the secret configured in Velocity.

:::code-group
```toml [features.toml]{2-3}
[proxy.velocity]
enabled = true
secret = "[proxy secret here]"
```
:::

### BungeeCord
For Bungeecord, set `enabled` to true.

:::code-group
```toml [features.toml]{2}
[proxy.bungeecord]
enabled = true
```
:::

> [!CAUTION]
> Ensure that the server’s firewall is correctly configured, as BungeeCord can’t verify if player info is from your proxy or an imposter.
