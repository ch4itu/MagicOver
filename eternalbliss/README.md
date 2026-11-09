# EternalBliss

[Play it](https://ch4itu.github.io/MagicOver/eternalbliss/)

EternalBliss is an RPG in 3D or 2D. Its shared world and its duels run on Solana Devnet through MagicBlock's public Ephemeral Rollup, and players sign in with Smaran: the same player name and passphrase as Magic Over.

- **Solo adventure**: no sign-in; progress saves on your device. There are two maps. The Eternal Kingdom is 300 × 300 tiles. The Farlands is a 10,000 × 10,000 realm with nine regions, nine waystations and 28 named places, made as you walk. Its Atlas pins a place or takes a caravan or ferry between waystations, and the First Survey sends you to three landmarks near the start. Both maps share one character and inventory.
- **Shared worlds**: the Eternal Kingdom, 32 characters in each world. When a world is full, new players start in the next one, and you always return to the world where your character lives. Five worlds are open (the program allows 64), and players in different worlds don't see each other. In a world, enemies, treasure and shops are shared, and everyone sees the same season and time of day, set by the rollup's clock. A character keeps its slot and progress when it leaves.
- **Buildings** cover tiles by what they are: castles 3×3 (the island castle 3×2), temples and inns 2×2, shops and houses 2×1. In the Farlands the same sizes apply, and landmarks such as the mill and the beacons cover 2×2. Stand up to two tiles from a building and press Space or click it.
- **Trees** stand well above the heroes in 3D.
- **Duels**: walk up to another player and challenge them. Turn based, no stakes: attack, magic or a healing potion.
- **Saved to Solana**: every few minutes while people play, and when a player leaves, the page asks for the world to be saved to Solana. The world pays MagicBlock's fee from its own reserve, never the player. If the reserve runs low, anyone can take the world off the rollup, top it up on Solana and put it back.

## On chain (Devnet)

| | Address |
| --- | --- |
| Program | [`6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb`](https://explorer.solana.com/address/6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb?cluster=devnet) |
| Shared world 0 | `GjYsCcq2r5KkVab7suzvStwYkAcJ3nTe9yXN8EcC5T9E` |
| Shared worlds 1 to 4 | `4BjzEs6z9hyN8bLFFS72D1xAX6QmrBdpCaHiM17PcNGt`, `4HWDd69H9QFJF7XbafHb9iCdqFefJwvARtyrgAtJJcm9`, `DA2v98URwgZtJgnxtkbk5NdyeJ9domyAur9KCSD4NE8p`, `4fKyyYJWpux3KLoEAe4o9PM2FwxnZwYgneeqZzN2vh5Z` |
| World 0's duel tables 0 to 3 | `HEy1reJephj1xp9WbFqbDFyb961AYeja1k3hKYAqexob`, `9vHQe2dvjcj6zyCPTWzuJyQZZpBeapJMe8uWcSmT9yh7`, `8yVx1guQXr2GnapNagQtftSq3JZyTvZ5tDZwkwXaRWkx`, `5ZzANFsX5B65rWQXVKmXjVgR23wEjYXPUTm2rLtGFfRg` |

World n is the account at `["eb-land", n]`. World w's duel tables are 4w to 4w+3, each at `["eb-duel", table]`; [program/README.md](program/README.md) lists them all.

## Files

- `index.html`: the whole game in one file.
- `program/`: the Rust program, its local checks and the deployed build. A clean build gives sha256 `ad998ffced3e81cbdc2c8465ab194cdc11046a88a24d8e75be7e0ca9ba883954`, the bytes on Devnet. [program/README.md](program/README.md) lists what changed in each release and what was tested.

## Changes in R13

- **More worlds.** A world holds 32 characters, and each one keeps its slot after leaving, so the first world fills for good. The program now allows up to 64 worlds, each with its own four duel tables, and five are open. The page puts you in the world that has your character, or, if you're new, in the first world with room. If someone takes the last slot just before you, the page looks again. The badge reads "Shared world 2" and so on in later worlds. A duel code from another world says so.
- **Entering buildings.** Reach is measured from the nearest tile a building covers, not from its own tile. After you click a button (Menu, zoom, Atlas) the keyboard goes back to the game, so Space reaches the building beside you instead of pressing that button again. Space uses the nearest building or NPC in reach.
- **The Farlands**, the second solo map described above.
- **Bigger houses, taller trees.** Houses cover 2×1 tiles. Trees are taller and fuller, and no tree, rock or lantern grows through a building any more.
- **Joining again right after leaving** the shared world works. Before, a join within 12 seconds of the last one could be refused as already processed.
- **Smoother walking in the shared world.** Another player's avatar used to stop every half second or so while they walked. Their position arrives in short bursts, because the page sends the next steps only after the last ones are confirmed, and the avatar finished each burst before the next one arrived. It now plays their walk slightly behind, by a buffer that follows that player's own update rhythm: 1.25 times the usual gap between their updates, between 0.15 and 0.8 s. Measured on the live rollup from a machine about 350 ms away, the avatar stood still in 3.5 to 10.5% of frames while walking, against 37 to 50% before, and each tile appeared about 0.3 s later. It still starts moving as soon as the first update arrives.
- **Steps no longer wait for routine checks.** Before sending a step, the page sometimes first fetched a new blockhash (every 12 seconds) or checked again that the rollup is MagicBlock's (every 20 seconds). That step then reached the other players one round trip later: with a step every 1.6 seconds, one step in four. Both are now renewed in the background before they run out, between steps rather than alongside one. Measured on the live rollup over 40 single steps, the slowest tenth went from about 1.0 s to 0.6 s from a machine about 350 ms from the rollup. The Smaran kit bundled in this page differs from the kit in `../smaran/` only by `renewGuards()`, which the page calls for that renewal.

The page bundles three.js r149, Solana web3.js 1.98.4, hash-wasm, noble-ciphers and TweetNaCl, under their own licences: see [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md). Everything here is on Devnet, with test SOL only.
