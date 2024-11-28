# PVP
PVP is a core part of vanilla mechanics, with even the smallest change affecting gameplay. Pumpkin allows you to fully configure PVP.

## Configuring PVP

#### `enabled`: Boolean
Weather PVP is enabled or not.

:::code-group
```toml [features.toml] {2}
[pvp]
enabled = true
```
:::

#### `hurt_animation`: Boolean
Weather to show red hurt animation and FOV bobbing or not.

:::code-group
```toml [features.toml] {2}
[pvp]
hurt_animation = true
```
:::

#### `protect_creative`: Boolean
Weather to protect players in creative againest PVP or not.

:::code-group
```toml [features.toml] {2}
[pvp]
protect_creative = true
```
:::

#### `knockback`: Boolean
Weather attacks should have knockback or not.

:::code-group
```toml [features.toml] {2}
[pvp]
knockback = true
```
:::

#### `swing`: Boolean
Weather players should swing when attacking or not.

:::code-group
```toml [features.toml] {2}
[pvp]
swing = true
```
:::

## Default Config
By default all PVP options are enabled to match vanilla behavior.

:::code-group
```toml [features.toml] {2}
[pvp]
enabled = true
hurt_animation = true
protect_creative = true
knockback = true
swing = true
```
:::
