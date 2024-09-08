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

## What Pumpkin wants to achieve

- **Performance**: Leveraging multi-threading for maximum speed and efficiency.
- **Compatibility**: Supports the latest Minecraft server version and adheres to vanilla game mechanics.
- **Security**: Prioritizes security by preventing known exploits.
- **Flexibility**: Highly configurable with the ability to disable unnecessary features.
- **Extensibility**: Provides a foundation for plugin development.

## What Pumpkin will not

- Provide compatibility with Vanilla or Bukkit servers (including configs and plugins).
- Function as a framework for building a server from scratch.

> [!IMPORTANT]
> Pumpkin is currently under heavy development.

## Features (WIP)

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
  - [x] Chunk Generation
  - [ ] World Borders
  - [ ] World Saving
- Player
  - [x] Player Skins
  - [x] Player Client brand
  - [x] Player Teleport
  - [x] Player Movement
  - [x] Player Animation
  - [x] Player Inventory
  - [x] Player Combat
- Server
  - [x] Plugins
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

```shell
git clone https://github.com/Snowiiii/Pumpkin.git
cd Pumpkin
```

You also may have to [install rust](https://www.rust-lang.org/tools/install) when you don't already have.

You can place a vanilla world into the Pumpkin/ directory when you want. Just name the World to `world`

Then run:

> [!NOTE]
> This can take a while. Because we enabled heavy optimizations for release builds
>
> To apply further optimizations specfic to your CPU and use your CPU features. You should set the target-cpu=native
> Rust flag.

```shell
cargo run --release
```

### Docker

Experimental Docker support is available.
The image is currently not published anywhere, but you can use the following command to build it:

```shell
docker build . -t pumpkin
```

To run it use the following command:

```shell
docker run --rm -v "./world:/pumpkin/world" pumpkin
```

## Contributions

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md)

## Communication

Consider joining our [discord](https://discord.gg/wT8XjrjKkf) to stay up-to-date on events, updates, and connect with other members.

## License
Pumpkin is licensed under the [Pumpkin License](LICENSE). By using Pumpkin, you agree to the terms of this license.

Key Provisions:
- You must give credit to the original author(s) of Pumpkin and provide a link to the project's repository.
- You may create derivative works based on Pumpkin, but they must also be distributed under the Pumpkin License.
- You may distribute Pumpkin or derivative works, either for free or commercially.
- If you a hosting company. you are required to pay a 5% royalty fee on that revenue.
- If you are not a hosting company which generating over €500. you are required to pay a 5% royalty fee on that revenue.

## Thanks

A big thanks to [wiki.vg](https://wiki.vg/) for providing valuable information used in the development of this project.
