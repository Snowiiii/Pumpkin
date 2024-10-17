## Benchmarks

Here, I compare common Minecraft servers against Pumpkin.

Is this comparison fair? Not really. While Pumpkin currently has far fewer features than other servers, which might suggest it uses fewer resources, it's important to consider that other servers have had years for optimization. Especially vanilla forks, which don’t need to rewrite the entire vanilla logic, can focus exclusively on optimizations.

ALL TESTS HAVE BEEN RAN MULTIPLE TIMES TO GUARANTEE CONSISTENT RESULTS

ALL PLAYERS DID NOT MOVE WHEN SPAWNING, ONLY THE INITIAL 8 CHUNKS WERE LOADED, THAT'S ALSO THE REASON CPU MAX IS USUALLY HIGH ON THE FIRST PLAYER
ALL SERVERS USED THEIR OWN TERRAIN GENERATION, NO WORLD WAS PRE-LOADED

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

Compile args:`--release`

Run args:

**File Size:** 12,3MB

**Startup time:** 8ms

**Shutdown time:** 0ms

| Players | RAM     | CPU idle | CPU Max |
| ------- | ------- | -------- | ------- |
| 0       | 392,2KB | 0,0%     | 0,0%    |
| 1       | 24,9MB  | 0,0%     | 4,0%    |
| 2       | 25,1MB  | 0,0%     | 0,6%    |
| 5       | 26,0MB  | 0,0%     | 1,0%    |
| 10      | 27,1MB  | 0,0%     | 1,5%    |

<sub><sup>pumpkin does cache already loaded chunks, resulting in no extra RAM usage besides player data and minimal CPU usage</sup></sub>

#### Compile time
Compiling from Nothing

**Debug:** 10.35sec
**Release:** 38.40sec

Recompilation (pumpkin crate)

**Debug:** 1.82sec
**Release:** 28.68sec

## Vanilla

Release: [1.21.1](https://piston-data.mojang.com/v1/objects/59353fb40c36d304f2035d51e7d6e6baa98dc05c/server.jar)

Compile args:

Run args: `nogui`

**File Size:** 51,6MB

**Startup time:** 7sec

**Shutdown time:** 4sec

| Players | RAM   | CPU idle | CPU Max |
| ------- | ----- | -------- | ------- |
| 0       | 860MB | 0,1-0,3% | 51,0%   |
| 1       | 1.5GB | 0,9-1%   | 41,0%   |
| 2       | 1.6GB | 1,0-1,1% | 10,0%   |
| 5       | 1.8GB | 2,0%     | 20,0%   |
| 10      | 2,2GB | 4,0%     | 24,0%   |

## Paper

Build: [122](https://api.papermc.io/v2/projects/paper/versions/1.21.1/builds/122/downloads/paper-1.21.1-122.jar)

Compile args:

Run args: `nogui`

**File Size:** 49,4MB

**Startup time:** 7sec

**Shutdown time:** 3sec

| Players | RAM   | CPU idle | CPU Max |
| ------- | ----- | -------- | ------- |
| 0       | 1.1GB | 0,2-0,3% | 36,0%   |
| 1       | 1.7GB | 0,9-1,0% | 47,0%   |
| 2       | 1.8GB | 1-1-1,0% | 10,0%   |
| 5       | 1.9GB | 1.5%     | 15,0%   |
| 10      | 2GB   | 3,0%     | 20,0%   |

## Purpur

Build: [2324](https://api.purpurmc.org/v2/purpur/1.21.1/2324/download)

Compile args:

Run args: `nogui`

**File Size:** 53,1MB

**Startup time:** 8sec

**Shutdown time:** 4sec

| Players | RAM   | CPU idle | CPU Max |
| ------- | ----- | -------- | ------- |
| 0       | 1.4GB | 0,2-0,3% | 25,0%   |
| 1       | 1.6GB | 0,7-1,0% | 35,0%   |
| 2       | 1.7GB | 1,1-1,3% | 9,0%    |
| 5       | 1.9GB | 1.6%     | 20,0%   |
| 10      | 2.2GB | 2-2,5,0% | 26,0%   |

## Minestom

Commit: [0ca1dda2fe](https://github.com/Minestom/Minestom/commit/0ca1dda2fe11390a1b89a228bbe7bf78fefc73e1)

Compile args:

Run args:

**Language:** Java

**File Size:** 2,8MB (Library)

**Startup time:** 310ms

**Shutdown time:** 0ms

<sub>[Used example code from](https://minestom.net/docs/setup/your-first-server)</sub>

| Players | RAM   | CPU idle | CPU Max |
| ------- | ----- | -------- | ------- |
| 0       | 228MB | 0,1-0,3% | 1,0%    |
| 1       | 365MB | 0,9-1,0% | 5,0%    |
| 2       | 371MB | 1-1,1%   | 4,0%    |
| 5       | 390MB | 1,0%     | 6,0%    |
| 10      | 421MB | 3,0%     | 9,0%    |

Benchmarked at 15.10.2024 18:34 (UTC+2)
