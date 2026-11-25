# SmaranWorld

A shared world on Smaran login: the Alderlands, 144 villages in open country, where signed-in players see each other walk and talk, on one clock for everyone.

[Play it](https://ch4itu.github.io/MagicOver/blitz/smaranworld/) · More on all the Smaran apps: [blitz/README.md](../README.md)

- **Walk and talk.** Sign in with the same username and password as the other Smaran apps. Signed-in players see each other move, and what they say shows over their heads and in the chat. Guests can look around without being shown.
- **144 villages.** Each village has a square with a monument stone in the middle, lanes, houses and, in farming country, fields. The villages lie ring by ring around the heart of the world: 4 at the heart, then rings of 12, 20, 28, 36 and 44.
- **Open country between them.** Meadows, farmland, forests, pine forests, hills, mountains, lakes, rivers, marsh, steppe, desert, tundra and the sea. Neighbouring villages are about 44 tiles apart (24 at the closest), and roads and bridges join every village to the rest, so all 144 can be reached on foot. The Villages list (by ring, with a search) and the world atlas take you to any of them at once.
- **Kept by relic holders.** A relic earned in Magic Over keeps a village: the n-th relic ever made belongs to village n (counting from the heart), and its holder's monument and welcome sign stand in the village square. Give the relic away and the village goes with it. Once all 144 have a relic, a new relic goes to village n mod 144, kept by whoever holds that village's highest-points relic.
- **One clock.** The time of day, the season, the residents' walks and the weather follow one UTC clock, so every page shows the same world at the same moment.

## How the land is made

The whole map is made in the browser from one fixed seed when the page opens, in about a third of a second on a computer and a little longer on a phone, and it is the same land, villages and names on every page. There is no map file to download, so the page got smaller (about 3 MB, from 3.6 MB).

- **Climate:** colder towards the top of the screen (tundra, pine forest), warmer towards the bottom (steppe, desert), with forests, meadows, farmland and marsh between, where the ground is wetter or drier. Height makes hills and mountain ranges, low ground holds lakes, and rivers run down from the hills.
- **Villages:** every village gets level, dry ground, a square, lanes, five to eight buildings that suit its land (windmills and farmhouses in farmland, watchtowers in the hills, temples in the desert) and a name from its land. The first twelve keep the old names of the Alderlands: Mossbank, Alderwick, Lantern Quay, Cedar Hollow and the rest.
- **Drawing:** the page draws only what is on screen. Zoomed far out it draws the land as plain colours and leaves out plants and stones, so a bigger world costs memory and start-up time, not frame rate.

**Growing past 144 villages.** The map takes the number of villages and adds rings around the edge. A 1,024-village world (1,476 tiles across) is made in about a second on a computer and has been tested: the first 144 villages keep their names and places, and only land near the old coast changes as the sea moves out. Two things come first: the SmaranWorld program accepts signs for 144 villages and stores the village number in one byte, so it needs a small upgrade; and the relic rule would count to the new number of villages instead of 144.

## How MagicBlock is used

- **Public rollup (devnet-as).** Where each signed-in player stands, the last 24 chat lines and the village signs live in accounts delegated to MagicBlock's public rollup. Each step, line and sign is a rollup transaction signed by the player's own Smaran key with 0 SOL, and every page, a guest's included, reads them from the rollup.
- **Names checked.** A name shows over a player only after the Smaran registry, on the same rollup, confirms that it belongs to the key that wrote it. Anything else shows as "Visitor".
- **Keepers from the relic records.** Which relic keeps which village comes from the relic records that Magic Over writes on MagicBlock's private rollup, read from their finalized copy on Solana.
- **Not saved to Solana.** Positions, chat and signs stay on the rollup and pay for no saves, so a rollup reset would clear them. The villages themselves need no account: they come from the map's seed.

## Who it is for, who pays, how long it lasts

- **For people who will never touch crypto,** and for places where buying or holding it is banned, restricted, heavily taxed or legally unclear. Players use a username and a password in any browser, on a phone or a computer. They never buy, hold or send any coin or token, there is no KYC step, and nothing has cash value. This is a design note, not legal advice.
- **Who pays.** Players pay nothing, ever. Whoever runs the apps pays the chain costs, mostly once as account deposits, to be covered by non-intrusive ads only: a sponsor's name on a village sign, never anything that interrupts play. There are no ads in this demo, and the plan is not proven yet. Anyone can also send SOL to the [save fund](../../programs/save-fund), one address that can only pay for saving these apps' accounts to Solana. It is a gift, not an investment: nothing comes back.
- **How long it lasts.** As long as Solana and MagicBlock run. No server: the page is static and open source, so anyone can host it, and the land needs nothing but the page. Devnet can be reset, so lasting records need mainnet. Details: [Built to outlive us](../README.md#built-to-outlive-us).

## On chain (Devnet)

| | |
| --- | --- |
| Program | [`HPEA1Ht9rBnqEQVdaEQ9QuqwiWAbY6fgCKs2ZPuQvcCW`](https://explorer.solana.com/address/HPEA1Ht9rBnqEQVdaEQ9QuqwiWAbY6fgCKs2ZPuQvcCW?cluster=devnet), source in [`programs/city`](../../programs/city) |
| Players | `["city-room", 0]`: 42 players shown to each other at once |
| Chat | `["city-chat", 0]`: the last 24 lines |
| Village signs | `["city-sign", 0]` to `["city-sign", 4]`: 160 sign places |

## Limits

- 42 signed-in players are shown to each other at once. A 43rd can still walk around, but is shown to the others only after someone leaves.
- Chat keeps the last 24 lines, and signs are short (56 characters).
- 144 villages until the program upgrade above.
- Built on our earlier city-builder engine; the land, the villages and the plants are new for this world.
