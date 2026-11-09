# EternalBliss

[Play it](https://ch4itu.github.io/MagicOver/eternalbliss/)

EternalBliss is an RPG on a 300 × 300 map, in 3D or 2D. Its shared world and its duels run on Solana Devnet through MagicBlock's public Ephemeral Rollup, and players sign in with Smaran: the same player name and passphrase as Magic Over.

- **Solo adventure**: no sign-in; progress saves on your device.
- **Shared world**: up to 32 characters. Enemies, treasure and shops are shared, and everyone sees the same season and time of day, set by the rollup's clock. A character keeps its slot and progress when it leaves.
- **Buildings** cover tiles by what they are: castles 3×3, temples and inns 2×2, shops 2×1, houses one tile. Walk up to one to enter it.
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
- `program/`: the Rust program, its local checks and the deployed build. A clean build gives sha256 `f689375a8797370a9872a05d935705e6393d5a9849bd815132df463b12bdfd99`, the bytes on Devnet. [program/README.md](program/README.md) lists what changed in each release and what was tested.

The page bundles three.js r149, Solana web3.js 1.98.4, hash-wasm, noble-ciphers and TweetNaCl, under their own licences: see [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md). Everything here is on Devnet, with test SOL only.
