# EternalBliss

[Play it](https://ch4itu.github.io/MagicOver/eternalbliss/)

EternalBliss is an RPG in 3D or 2D. Its shared world and its duels run on Solana Devnet through MagicBlock's public Ephemeral Rollup, and players sign in with Smaran: the same player name and passphrase as Magic Over.

- **Solo adventure**: no sign-in; progress saves on your device. There are two maps. The Eternal Kingdom is 300 × 300 tiles. The Farlands is a 10,000 × 10,000 realm with nine regions, nine waystations and 28 named places, made as you walk. Its Atlas pins a place or takes a caravan or ferry between waystations, and the First Survey sends you to three landmarks near the start. Both maps share one character and inventory.
- **Shared world**: the Eternal Kingdom, up to 32 characters. Enemies, treasure and shops are shared, and everyone sees the same season and time of day, set by the rollup's clock. A character keeps its slot and progress when it leaves.
- **Buildings** cover tiles by what they are: castles 3×3 (the island castle 3×2), temples and inns 2×2, shops and houses 2×1. In the Farlands the same sizes apply, and landmarks such as the mill and the beacons cover 2×2. Stand up to two tiles from a building and press Space or click it.
- **Trees** stand well above the heroes in 3D.
- **Duels**: walk up to another player and challenge them. Turn based, no stakes: attack, magic or a healing potion.
- **Saved to Solana**: every few minutes while people play, and when a player leaves, the page asks for the world to be saved to Solana. The world pays MagicBlock's fee from its own reserve, never the player. If the reserve runs low, anyone can take the world off the rollup, top it up on Solana and put it back.

## On chain (Devnet)

| | Address |
| --- | --- |
| Program | [`6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb`](https://explorer.solana.com/address/6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb?cluster=devnet) |
| Shared world | `GjYsCcq2r5KkVab7suzvStwYkAcJ3nTe9yXN8EcC5T9E` |
| Duel tables 0 to 3 | `HEy1reJephj1xp9WbFqbDFyb961AYeja1k3hKYAqexob`, `9vHQe2dvjcj6zyCPTWzuJyQZZpBeapJMe8uWcSmT9yh7`, `8yVx1guQXr2GnapNagQtftSq3JZyTvZ5tDZwkwXaRWkx`, `5ZzANFsX5B65rWQXVKmXjVgR23wEjYXPUTm2rLtGFfRg` |

## Files

- `index.html`: the whole game in one file.
- `program/`: the Rust program, its local checks and the deployed build. A clean build gives sha256 `1b7e2b455449e1bea1319bf3d9a21e3a91eda92da7abaa79f6e8da68449aca3a`, the bytes on Devnet. [program/README.md](program/README.md) lists what changed in each release and what was tested.

## Changes in R13

- **Entering buildings.** Reach is measured from the nearest tile a building covers, not from its own tile. After you click a button (Menu, zoom, Atlas) the keyboard goes back to the game, so Space reaches the building beside you instead of pressing that button again. Space uses the nearest building or NPC in reach.
- **The Farlands**, the second solo map described above.
- **Bigger houses, taller trees.** Houses cover 2×1 tiles. Trees are taller and fuller, and no tree, rock or lantern grows through a building any more.
- **Joining again right after leaving** the shared world works. Before, a join within 12 seconds of the last one could be refused as already processed.

The page bundles three.js r149, Solana web3.js 1.98.4, hash-wasm, noble-ciphers and TweetNaCl, under their own licences: see [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md). Everything here is on Devnet, with test SOL only.
