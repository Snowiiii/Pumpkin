# Quick Start

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

## Docker

Experimental Docker support is available.
The image is currently not published anywhere, but you can use the following command to build it:

```shell
docker build . -t pumpkin
```

To run it use the following command:

```shell
docker run --rm -v "./world:/pumpkin/world" pumpkin
```
