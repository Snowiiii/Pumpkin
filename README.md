<div align="center">

# Pumpkin

![CI](https://github.com/Snowiiii/Pumpkin/actions/workflows/rust.yml/badge.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Current version)](https://img.shields.io/badge/current_version-1.21.1-blue)

</div>

Pumpkin is a Minecraft server built entirely in Rust, offering a fast, efficient, 
and customizable experience. It prioritizes performance and player enjoyment while adhering to the core mechanics of the game.

![image](https://github.com/user-attachments/assets/7e2e865e-b150-4675-a2d5-b52f9900378e)


Pumpkin is currently under heavy development.

### What Pumpkin wants to achive:
- **Performance**: Leveraging multi-threading for maximum speed and efficiency.
- **Compatibility**: Supports the latest Minecraft server version and adheres to vanilla game mechanics.
- **Security**: Prioritizes security by preventing known exploits.
- **Flexibility**: Highly configurable with the ability to disable unnecessary features.
- **Extensibility**: Provides a foundation for plugin development.

### What Pumpkin will not be:
- A direct replacement for Vanilla or Bukkit servers.
- A framework for building a server from scratch.

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
  - [x] Player Skin
  - [x] Player Client brand
  - [x] Player Teleport
  - [x] Player Movement
  - [x] Player Animation
  - [ ] Player Inventory
  - [ ] Player Attack
- Server
  - [x] RCON
  - [x] Inventories
  - [x] Chat
  - [x] Commands

Check out our [Github Project](https://github.com/users/Snowiiii/projects/12/views/3) to see current progress

## How to run
There are currently no release builds, because there was no release :D.

To get Pumpkin running you first have to clone it:
```
git clone https://github.com/Snowiiii/Pumpkin.git
cd Pumpkin
```
You also may have to [install rust](https://www.rust-lang.org/tools/install) when you don't already have.

You need to pregenerate the world and place it inside of the Pumpkin/ directory.
Make sure to generate chunks close to (0,0) since that is where the player gets spawned by default.

Then run:
```
cargo run --release
```

## Contributions
Contributions are welcome!. See [CONTRIBUTING.md](CONTRIBUTING.md)

## Communication
Consider joining our [discord](https://discord.gg/wT8XjrjKkf) to stay up-to-date on events, updates, and connect with other members.

### Thanks
A big thanks to https://wiki.vg/ for providing valuable information used in the development of this project.
