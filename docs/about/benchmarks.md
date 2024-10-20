# Benchmarks

Here, common Minecraft servers are compared against Pumpkin.

> [!NOTE]
> All tests have been ran multiple times to guarantee consistent results.
> All players did not move when spawning, only the initial 8 chunks were loaded.
> All servers used their own terrain generation, no world was pre-loaded.

> [!IMPORTANT]
> `CPU Max` is usually higher with one player is as the initial chunks are being loaded.

> [!CAUTION]
> **This comparison is unfair.** Pumpkin currently has far fewer features than other servers, which might suggest it uses fewer resources.
> It's also important to consider that other servers have had years for optimization.
> Vanilla forks, which don’t need to rewrite the entire vanilla logic, can focus exclusively on optimizations.

![Screenshot From 2024-10-15 16-42-53](https://github.com/user-attachments/assets/e08fbb00-42fe-4479-a03b-11bb6886c91a)

## Specifications

#### Technical

**Software**

- Distribution: Manjaro Linux
- Architecture: x86_64 (64-bit)
- Kernel Version: 6.11.3-arch1-1

**Hardware**

- Motherboard: MAG B650 TOMAHAWK WIFI
- CPU: AMD Ryzen 7600X 6-Core
- RAM: Corsair 2x16GB DDR5 6000Mhz
- Storage: Samsung 990 PRO 1TB PCIe 4.0 M.2 SSD
- Cooling: be quiet Dark Rock Elite

**Rust**

- Toolchain: stable-x86_64-unknown-linux-gnu (1.81.0)
- Rust Compiler: rustc 1.81.0 (eeb90cda1 2024-09-04)

**Java**

- JDK Version: OpenJDK 23 64-Bit 2024-09-17
- JRE Version: OpenJDK Runtime Environment (build 23+37)
- Vendor: Oracle

#### Game

- Minecraft version: 1.21.1
- View distance: 10
- Simulated distance: 10
- Online mode: false
- Rcon: false

<sub><sup>online mode was disabled for easier testing with non-premium accounts</sup></sub>

## Pumpkin

Build: [8febc50](https://github.com/Snowiiii/Pumpkin/commit/8febc5035d5611558c13505b7724e6ca284e0ada)

Compile args: `--release`

Run args:

**File Size:** <FmtNum :n=12.3 />MB

**Startup time:** <FmtNum :n=8 />ms

**Shutdown time:** <FmtNum :n=0 />ms

| Players | RAM                   | CPU Idle         | CPU Max            |
| ------- | --------------------- | ---------------- | ------------------ |
| 0       | <FmtNum :n=392.2 />KB | <FmtNum :n=0 />% | <FmtNum :n=0 />%   |
| 1       | <FmtNum :n=24.9 />MB  | <FmtNum :n=0 />% | <FmtNum :n=4 />%   |
| 2       | <FmtNum :n=25.1 />MB  | <FmtNum :n=0 />% | <FmtNum :n=0.6 />% |
| 5       | <FmtNum :n=26 />MB    | <FmtNum :n=0 />% | <FmtNum :n=1 />%   |
| 10      | <FmtNum :n=27.1 />MB  | <FmtNum :n=0 />% | <FmtNum :n=1.5 />% |

<sub><sup>Pumpkin does cache already loaded chunks, resulting in no extra RAM usage besides player data and minimal CPU usage.</sup></sub>

#### Compile time
Compiling from Nothing:

**Debug:** <FmtNum :n=10.35 />sec
**Release:** <FmtNum :n=38.40 />sec

Recompilation (pumpkin crate):

**Debug:** <FmtNum :n=1.82 />sec
**Release:** <FmtNum :n=28.68 />sec

## Vanilla

Release: [1.21.1](https://piston-data.mojang.com/v1/objects/59353fb40c36d304f2035d51e7d6e6baa98dc05c/server.jar)

Compile args:

Run args: `nogui`

**File Size:** <FmtNum :n=51.6 />MB

**Startup time:** <FmtNum :n=7 />sec

**Shutdown time:** <FmtNum :n=4 />sec

| Players | RAM                | CPU idle                             | CPU Max |
| ------- | ------------------ | ------------------------------------ | ------- |
| 0       | 860MB              | <FmtNum n=0.1 /> - <FmtNum n=0.3 />% | 51%     |
| 1       | <FmtNum n=1.5 />GB | <FmtNum n=0.9 /> - 1%                | 41%     |
| 2       | <FmtNum n=1.6 />GB | 1 - <FmtNum n=1.1 />%                | 10%     |
| 5       | <FmtNum n=1.8 />GB | 2%                                   | 20%     |
| 10      | <FmtNum n=2.2 />GB | 4%                                   | 24%     |

## Paper

Build: [122](https://api.papermc.io/v2/projects/paper/versions/1.21.1/builds/122/downloads/paper-1.21.1-122.jar)

Compile args:

Run args: `nogui`

**File Size:** <FmtNum :n=49.4 />MB

**Startup time:** <FmtNum :n=7 />sec

**Shutdown time:** <FmtNum :n=3 />sec

| Players | RAM                 | CPU idle                               | CPU Max           |
| ------- | ------------------- | -------------------------------------- | ----------------- |
| 0       | <FmtNum :n=1.1 />GB | <FmtNum :n=0.2 /> - <FmtNum :n=0.3 />% | <FmtNum :n=36 />% |
| 1       | <FmtNum :n=1.7 />GB | <FmtNum :n=0.9 /> - <FmtNum :n=1.0 />% | <FmtNum :n=47 />% |
| 2       | <FmtNum :n=1.8 />GB | <FmtNum :n=1 /> - <FmtNum :n=1.1 />%   | <FmtNum :n=10 />% |
| 5       | <FmtNum :n=1.9 />GB | <FmtNum :n=1.5 />%                     | <FmtNum :n=15 />% |
| 10      | <FmtNum :n=2 />GB   | <FmtNum :n=3 />%                       | <FmtNum :n=20 />% |


## Purpur

Build: [2324](https://api.purpurmc.org/v2/purpur/1.21.1/2324/download)

Compile args:

Run args: `nogui`

**File Size:** <FmtNum :n=53.1 />MB

**Startup time:** <FmtNum :n=8 />sec

**Shutdown time:** <FmtNum :n=4 />sec

| Players | RAM                 | CPU idle                               | CPU Max           |
| ------- | ------------------- | -------------------------------------- | ----------------- |
| 0       | <FmtNum :n=1.4 />GB | <FmtNum :n=0.2 /> - <FmtNum :n=0.3 />% | <FmtNum :n=25 />% |
| 1       | <FmtNum :n=1.6 />GB | <FmtNum :n=0.7 /> - <FmtNum :n=1.0 />% | <FmtNum :n=35 />% |
| 2       | <FmtNum :n=1.7 />GB | <FmtNum :n=1.1 /> - <FmtNum :n=1.3 />% | <FmtNum :n=9 />%  |
| 5       | <FmtNum :n=1.9 />GB | <FmtNum :n=1.6 />%                     | <FmtNum :n=20 />% |
| 10      | <FmtNum :n=2.2 />GB | <FmtNum :n=2 /> - <FmtNum :n=2.5 />%   | <FmtNum :n=26 />% |

## Minestom

Commit: [0ca1dda2fe](https://github.com/Minestom/Minestom/commit/0ca1dda2fe11390a1b89a228bbe7bf78fefc73e1)

Compile args:

Run args:

**Language:** Benchmarks ran with Kotlin 2.0.0 (Minestom itself is made with Java)

**File Size:** <FmtNum :n=2.8 />MB (Library)

**Startup time:** <FmtNum :n=310 />ms

**Shutdown time:** <FmtNum :n=0 />ms

<sub>[Used example code from](https://minestom.net/docs/setup/your-first-server)</sub>

| Players | RAM                 | CPU idle                               | CPU Max          |
| ------- | ------------------- | -------------------------------------- | ---------------- |
| 0       | <FmtNum :n=228 />MB | <FmtNum :n=0.1 /> - <FmtNum :n=0.3 />% | <FmtNum :n=1 />% |
| 1       | <FmtNum :n=365 />MB | <FmtNum :n=0.9 /> - <FmtNum :n=1.0 />% | <FmtNum :n=5 />% |
| 2       | <FmtNum :n=371 />MB | <FmtNum :n=1 /> - <FmtNum :n=1.1 />%   | <FmtNum :n=4 />% |
| 5       | <FmtNum :n=390 />MB | <FmtNum :n=1.0 />%                     | <FmtNum :n=6 />% |
| 10      | <FmtNum :n=421 />MB | <FmtNum :n=3 />%                       | <FmtNum :n=9 />% |


Benchmarked at <FmtDateTime :d="new Date('2024-10-15T16:34Z')" />
