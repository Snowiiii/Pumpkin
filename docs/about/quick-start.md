# Quick Start

**Current Status:**

Pre-release: Currently under development and not yet ready for official release.

To get Pumpkin running, you first have to clone it:
```shell
git clone https://github.com/Snowiiii/Pumpkin.git
cd Pumpkin
```

You may also have to [install rust](https://www.rust-lang.org/tools/install) if you don't already have it.

**Optional:**

You can place a vanilla world into the Pumpkin/ directory if you want. Just name the World `world`

Then run:

> [!NOTE]
> This can take a while because we enabled heavy optimizations for release builds
>
> To apply further optimizations specific to your CPU and use your CPU features, you should set the target-cpu=native
> Rust flag.

```shell
cargo run --release
```

## Docker

Experimental Docker support is available.
The image is currently not published anywhere, but you can use the following command to build it:

```shell
docker build . -t pumpkin
```

To run it, use the following command:

```shell
docker run --rm -p 25565:25565 -v "./world:/pumpkin/world" pumpkin
```

## Test Server
Pumpkin has a Test server maintained by @kralverde. Its runs on the latest commit of Pumpkin

- **IP:** pumpkin.kralverde.dev

**Specs:**
- OS: Debian GNU/Linux bookworm 12.7 x86_64
- Kernel: Linux 6.1.0-21-cloud-amd64
- CPU: Intel Core (Haswell, no TSX) (2) @ 2.40 GHz
- RAM: 4GB DIMM