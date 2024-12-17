# Resource Pack
Servers can send resource packs to the client in order to change the appearance of the game on the client. Pumpkin allows you to fully configure the resource pack.

> [!TIP]
> Minify your resource pack using [PackSquash](https://packsquash.aylas.org/)! This can help clients download the resource pack faster.

## Configuring Resource Pack

#### `enabled`: Boolean
Weather a resource pack is enabled or not.

:::code-group
```toml [features.toml] {2}
[resource_pack]
enabled = true
```
:::

#### `resource_pack_url`: String
The direct download URL to the resource pack. 

> [!TIP]
> You can host the resource pack for free at [MCPacks](https://mc-packs.net/).

:::code-group
```toml [features.toml] {3}
[resource_pack]
enabled = true
resource_pack_url = "[your download URL here]"
```
:::

#### `resource_pack_sha1`: String
The hash of the resource pack, using the SHA1 algorithm.

> [!IMPORTANT]
> Although not required to specify, you should specify this field because the client will otherwise redownload the resource pack every time they join the server, even if there are no changes to the resource pack.

> [!WARNING]
> Make sure to update this field if the resource pack is modified.

::: details How do I get the SHA1 hash of my resource pack?
::: code-group
```powershell [Windows (PowerShell)]
Get-FileHash [file] SHA1
```
```shell [Mac OS]
shasum -a 1 [file]
```
```shell [Linux]
sha1sum [file]
```
:::

:::code-group
```toml [features.toml] {3}
[resource_pack]
enabled = true
resource_pack_sha1 = "[your hash here]"
```
:::

#### `prompt_message`: String
The message to show to the user when prompted to download the resource pack.

:::code-group
```toml [features.toml] {3}
[resource_pack]
enabled = true
prompt_message = "[your message here]"
```
:::

#### `force`: Boolean
Weather to force the client to download the resource pack or not. If the client declines the download, they will be kicked from the server.

:::code-group
```toml [features.toml] {3}
[resource_pack]
enabled = true
force = false
```
:::

## Default Config
By default, resource pack is disabled.

:::code-group
```toml [features.toml]
[resource_pack]
enabled = false
resource_pack_url = ""
resource_pack_sha1 = ""
prompt_message = ""
force = false
```
:::
