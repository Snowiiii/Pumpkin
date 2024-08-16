<div align="center">

# Pumpkin

![CI](https://github.com/Snowiiii/Pumpkin/actions/workflows/rust.yml/badge.svg)
[![Discord](https://img.shields.io/discord/1268592337445978193.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/wT8XjrjKkf)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Current version)](https://img.shields.io/badge/current_version-1.21.1-blue)

</div>

Pumpkin is a Minecraft server built entirely in Rust, offering a fast, efficient,
and customizable experience. It prioritizes performance and player enjoyment while adhering to the core mechanics of the game.

![image](https://github.com/user-attachments/assets/7e2e865e-b150-4675-a2d5-b52f9900378e)

### What Pumpkin wants to achive:
- **Performance**: Leveraging multi-threading for maximum speed and efficiency.
- **Compatibility**: Supports the latest Minecraft server version and adheres to vanilla game mechanics.
- **Security**: Prioritizes security by preventing known exploits.
- **Flexibility**: Highly configurable with the ability to disable unnecessary features.
- **Extensibility**: Provides a foundation for plugin development.

### What Pumpkin will not:
- Provide compatibility with Vanilla or Bukkit servers (including configs and plugins).
- Function as a framework for building a server from scratch.

> [!IMPORTANT]
> Pumpkin is currently under heavy development.

### Features (WIP)
- [x] Configuration (toml)
- [x] Server Status/Ping
- [x] Login
- Player Configuration
  - [x] Registries (biome types, paintings, dimensions)
  - [x] Server Brand
  - [ ] Server Links
  - [x] Set Resource Pack
  - [ ] Cookies
- World
  - [x] World Joining
  - [x] Player Tab-list
  - [x] World Loading
  - [x] Entity Spawning
  - [x] Chunk Loading
  - [ ] World Generation
  - [ ] World Borders
  - [ ] World Saving
- Player
  - [x] Player Skins
  - [x] Player Client brand
  - [x] Player Teleport
  - [x] Player Movement
  - [x] Player Animation
  - [ ] Player Inventory
  - [x] Player Combat
- Server
  - [ ] Query
  - [x] RCON
  - [x] Inventories
  - [x] Particles
  - [x] Chat
  - [x] Commands
- Proxy
  - [ ] Velocity

Check out our [Github Project](https://github.com/users/Snowiiii/projects/12/views/3) to see current progress

## How to run
There are currently no release builds, because there was no release :D.

To get Pumpkin running you first have to clone it:
```
git clone https://github.com/Snowiiii/Pumpkin.git
cd Pumpkin
```
You also may have to [install rust](https://www.rust-lang.org/tools/install) when you don't already have.

For Now, until we don't have own chunk generation.
You need to pregenerate the world and place it inside of the Pumpkin/ directory.
Make sure to generate chunks close to (0,0) since that is where the player gets spawned by default.

Then run:
> [!NOTE]
> This can take a while. Because we enabled heavy optimations for release builds
```
RUSTFLAGS="-C target-cpu=native" cargo run --release
```

## Contributions
Contributions are welcome!. See [CONTRIBUTING.md](CONTRIBUTING.md)

## Communication
Consider joining our [discord](https://discord.gg/wT8XjrjKkf) to stay up-to-date on events, updates, and connect with other members.

### Thanks
A big thanks to https://wiki.vg/ for providing valuable information used in the development of this project.
