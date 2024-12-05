# Query
Query protocol is an simple way to query the server about its status. Pumpkin fully supports the query protocol.

## Configuring Query

#### `enabled`: Boolean
Whether to listen for query protocol requests or not.

:::code-group
```toml [features.toml] {2}
[query]
enabled = true
```
:::

#### `port`: Integer (0-65535) (optional)
What port to listen to query protocol requests. If not specified, it uses the same port as the server.

:::code-group
```toml [features.toml] {3}
[query]
enabled = true
port = 12345
```
:::

## Default Config
By default query is disabled. It will run on the server port if enabled unless specified explicitly.

:::code-group
```toml [features.toml]
[query]
enabled = true
port = 25565
```
:::
