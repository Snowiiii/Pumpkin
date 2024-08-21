# Project Structure

## Overview

Pumpkin is split into multiple crates, thus having a set project structure between contributors is essential.

## Pumpkin-Core

The core crate has some special rules that only apply to it:

- It may not depend on any other pumpkin crate
- There may not be any files directly under src/, except for the mod.rs file (this is to help with organisation)

## Other crate rules

- [`pumpkin-protocol`](/pumpkin-protocol/) - contains definitions for packet types **and** their serialization (be it through serde, or manually implementing `ClientPacket`/`ServerPacket`), only the `pumpkin` crate may depend on this
- `pumpkin-macros` - similarly to `pumpkin-core`, it may not depend on any other pumpkin crate
