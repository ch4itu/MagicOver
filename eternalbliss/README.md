# EternalBliss

[Play it](https://ch4itu.github.io/MagicOver/eternalbliss/) · [Smaran version, with relics and the shared sign-in](https://ch4itu.github.io/MagicOver/blitz/eternalbliss/)

EternalBliss is an RPG in 3D or 2D. Its shared world and its duels run on Solana Devnet through MagicBlock's public Ephemeral Rollup, and players sign in with Smaran: the same username and password as Magic Over and the other Smaran apps.

- **Solo adventure**: no sign-in; progress saves on your device. There are two maps, and a new player starts in the Farlands. The Eternal Kingdom is 300 × 300 tiles. The Farlands is a 10,000 × 10,000 realm with nine regions, nine waystations and 28 named places, made as you walk. Its Atlas pins a place or takes a caravan or ferry between waystations, and the First Survey sends you to three landmarks near the start. Both maps share one character and inventory.
- **Shared worlds**: the Eternal Kingdom, up to 32 players in a world at a time. Characters are kept in character books on the rollup, and a world seat is taken only while you play, so everyone online plays in the first world until 32 are there together; then the next world opens. Five worlds are open, so there are seats for 160 players at once, and two character books of 80 hold 160 characters. The program allows 64 worlds and 256 books, and the page finds new ones by itself, so opening more needs no change to the program or the page. Players in different worlds don't see each other. In a world, enemies, treasure and shops are shared, and everyone sees the same season and time of day, set by the rollup's clock. Leaving writes your character back to its book with all its progress.
- **Buildings** cover tiles by what they are: castles 3×3 (the island castle 3×2), temples and inns 2×2, shops and houses 2×1. In the Farlands they are larger where the ground allows: inns, shops and temples up to 3×2, houses up to 2×2, and landmarks such as the mill and the beacons 2×2. Stand up to two tiles from a building and press Space or click it.
- **Trees** stand well above the heroes in 3D.
- **Duels**: walk up to another player and challenge them. Turn based, no stakes: attack, magic or a healing potion.
- **Saved to Solana**: every few minutes while people play, and when a player leaves, the page asks for the world to be saved to Solana, and after leaving, the player's character book too. Each pays MagicBlock's fee from its own reserve, never the player. If a reserve runs low, anyone can take that world or book off the rollup, top it up on Solana from the [save fund](../programs/save-fund) and put it back.

## Who it is for, who pays, how long it lasts

- **For people who will never touch crypto,** and for places where buying or holding it is banned, restricted, heavily taxed or legally unclear. Players use a username and a password in any browser, on a phone or a computer. They never buy, hold or send any coin or token, there is no KYC step, and nothing has cash value: duels have no stakes. This is a design note, not legal advice.
- **Who pays.** Players pay nothing, ever. Whoever runs the game pays the chain costs, mostly once as account deposits, to be covered by non-intrusive ads only, never anything that interrupts play. There are no ads in this demo, and the plan is not proven yet. Anyone can also send SOL to the [save fund](../programs/save-fund), which tops up the save reserves of the worlds, the character books and the Smaran login groups once they run low (see Saved to Solana above). It is a gift, not an investment: nothing comes back.
- **How long it lasts.** As long as Solana and MagicBlock run, once the save fund has SOL, someone runs its refill script (anyone can, with a little SOL for fees) and our upgrade key is given up, which we plan for mainnet. No server: the page is static and open source, so anyone can host it. Worlds, character books and logins are stored on Solana with their storage paid up front. Devnet can be reset, so lasting records need mainnet. Details: [Built to outlive us](../blitz/README.md#built-to-outlive-us).

## On chain (Devnet)

| | Address |
| --- | --- |
| Program | [`6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb`](https://explorer.solana.com/address/6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb?cluster=devnet) |
| Shared world 0 | `GjYsCcq2r5KkVab7suzvStwYkAcJ3nTe9yXN8EcC5T9E` |
| Shared worlds 1 to 4 | `4BjzEs6z9hyN8bLFFS72D1xAX6QmrBdpCaHiM17PcNGt`, `4HWDd69H9QFJF7XbafHb9iCdqFefJwvARtyrgAtJJcm9`, `DA2v98URwgZtJgnxtkbk5NdyeJ9domyAur9KCSD4NE8p`, `4fKyyYJWpux3KLoEAe4o9PM2FwxnZwYgneeqZzN2vh5Z` |
| World 0's duel tables 0 to 3 | `HEy1reJephj1xp9WbFqbDFyb961AYeja1k3hKYAqexob`, `9vHQe2dvjcj6zyCPTWzuJyQZZpBeapJMe8uWcSmT9yh7`, `8yVx1guQXr2GnapNagQtftSq3JZyTvZ5tDZwkwXaRWkx`, `5ZzANFsX5B65rWQXVKmXjVgR23wEjYXPUTm2rLtGFfRg` |
| Character books 0 and 1 | `4XbAoJvMbNiCiAFPsvgJNcVNK7tpkURKmei1RYCQWqCk`, `GKrd68nQSad5wtBVCxKubya1hBkdpm8oc6TZWnuYPAXM` |

World n is the account at `["eb-land", n]`. World w's duel tables are 4w to 4w+3, each at `["eb-duel", table]`; [program/README.md](program/README.md) lists them all. Character book b is the account at `["eb-book", b]` and holds 80 characters.

## Files

- `index.html`: the whole game in one file.
- `program/`: the Rust program, its local checks and the deployed build. A clean build gives sha256 `0a1ccaf1c9783671ad1d90afe9dd863e7bc38d5f28ac4068c007ba31951671dd`, the bytes on Devnet. [program/README.md](program/README.md) lists what changed in each release and what was tested.

## Changes in R15.56

Wayfarer's Rest, the first settlement of the Farlands, is laid out by hand. Solo Farlands only.

- **Streets and a square.** The sand plain where seven roads met is now a flagstone square, two worn streets, footpaths and a village green. Water and the roads beyond the village are unchanged.
- **Buildings.** The inn, the Exchange, the chapel and the hall stand on the same tiles. The chapel and the hall turn their doors to the square. The cottage has its own plot with a fenced garden.
- **The square and the landing.** A well, an oak with a bench, an orchard stall, a signpost naming Millbend, the Old Tollhouse and Bell Orchard, a wayside shrine, and a jetty with crates and mooring posts. The well, stall, signpost and shrine can be read with Space or Interact.
- **Residents with a day.** The same seven people live here, now all named. Mira stays by the well. The other six walk to different places through the day, pause for a hero standing close, and go indoors at night. Two of them speak differently once the Harvest Bell is back.
- **Lamps** throw warm light on the ground at night.
- **Sound.** With Sound on, the river is heard near water, with a low wind and a few birds by day. A volume control sits beside the Sound button. Sound stays off until switched on.

## Changes in R15.55

R15.55 adds the best parts of a later candidate (R15.54) to the live game. The live game stays the base, so everything from R15.33 to R15.37 is kept.

- **The Silent Harvest, a story quest in the Farlands.** Mira at Wayfarer's Rest tells it. Find the ward key at Millbend, go down into the vault under the Old Tollhouse, open the iron gate, defeat the Archive Warden, take the Harvest Bell and bring it back to Mira for 65 gold and 85 XP. Progress is saved at every step.
- **The goal tag follows the quest.** Until Mira tells it, the tag shows the First Survey as before. After that it shows the next step of the quest. Tapping it pins the place, so the card shows its distance and direction, and the pin moves on as the quest does. The Explorer's Journal shows the quest too.
- **First person.** In the 3D view, solo, the eye button switches to an eye-level view. Movement follows where you look. Turn with the Look buttons, by dragging, or with Q and E. Use acts on what is in front of you. A battle is shown by the battle camera and the view returns to first person afterwards. First person is not offered in shared worlds in this release.
- **Larger buildings in the Farlands.** Inns, shops and temples take up to 3×2 tiles and houses up to 2×2 where the ground allows, with the entrance marked on the model. The walls block exactly the tiles the building covers.
- **Fixed from the candidate.** The quest could not be finished there: the Warden's defeat was not recorded inside the vault, and walking into the gate spent the key without opening it for good. Both are repaired. Look left and Look right were swapped and are corrected.

## Changes in R15.37

- **Battles happen in the world.** In the 3D view a solo battle no longer opens a full-screen window. Compact controls sit above the stats bar: the enemy's name and HP, the latest message, and Attack, Magic, Heal and Flee. The panels over the world step aside and the camera moves in, more on short screens such as landscape phones.
- **What you see.** The hero and the enemy turn to face each other side by side across the screen. To attack, the hero steps in, winds up and cuts, and the blade meets the enemy as its HP drops. Swordsmen strike back the same way; wolves, dragons and crawlers pounce and bite. Whoever is hit flinches. A defeated enemy falls and sinks away, and a defeated hero falls before being revived. An enemy from a random encounter walks in. Trees between the fight and the camera are taken out, and buildings there are faded, until the battle ends.
- **On horseback** the horse turns so the enemy stands at its shoulder on the sword side, and the rider cuts down from the saddle. You stay mounted.
- **Same rules.** Damage, HP, MP, XP, gold, potions, fleeing and defeat work exactly as before. In the world, the buttons wait while the enemy replies and stay off once the battle is decided.
- **The battle window stays** in shared worlds, in the 2D view, and for a battle that starts while sailing.

## Changes in R15.36

R15.36 brings the solo work of R14 to R15.32 to the live page, with the fixes and the phone layout tested since.

- **The Farlands is the default solo map.** A new player starts at Wayfarer's Rest. "Return to Eternal Kingdom" in the menu goes to the original map, and a player who chooses it stays there after a reload. Shared worlds stay on the Eternal Kingdom, the map the program checks moves against.
- **A goal on screen.** In the Farlands the card shows the next journal landmark, for example "First Survey 0/3 · Millbend". Tapping it pins that place, so the card shows its distance and direction; tapping it again opens the Explorer's Journal.
- **Phones.** In the Farlands the location card is compact: Play Together, the Atlas, routes from a waystation, the errand and buying the horse sit behind "Actions", while boarding, disembarking, mounting and leaving a building stay one tap. Landscape phones get a shorter card and a one-line stats bar, and text on the card and bars is at least 12 px. On desktop and in the Eternal Kingdom every action stays on the card.
- **Horse.** Buy one at a waystation for 75 gold and ride faster on land. It follows you, and buying and mounting use in-game dialogs.
- **Boats.** Moored boats wait at ports and on shores. Tap one to board and sail on real water; your horse rides aboard. Disembark at the shore, and the boat stays where you left it for that session.
- **More to find.** Coastal and river settlements, local errands, waterways and routes on the Atlas, and an Explorer's Journal with four chapters.
- **Smoother solo walking** in both maps.
- **Music.** The main menu has a music switch. It starts off.
- **Fixes.** Buying the horse uses the gold you see. The horse can't be mounted while sailing. A boat taken from its mooring no longer leaves a copy behind. A diagonal step past a corner no longer launches a boat or uses a pickaxe. The journal button sits in its tool row, and the location card no longer runs under the minimap.

## Changes in R13

- **Character books.** A character used to keep its world seat for good, so a world of 32 filled up for good and players were split by when they signed up. Characters now live in character books on the rollup, 80 to a book (two books are open; the program allows 256). A seat is taken only while you play: leaving writes your character back to its book with everything it has, and frees the seat. Everyone online plays in the first world until 32 are there at once. If a world is full, a seat whose player has been away for 45 seconds can go to a newcomer, and that player's character is written back to their own book first, so nothing is lost. The rankings list every character, from the books and the worlds. If a tab loses its seat (away too long, or you left in another tab), it says so. Characters from before books keep their seat and move into a book the first time they leave.
- **More worlds.** The program allows up to 64 worlds, each with its own four duel tables, and five are open. The page puts you in the world where your character is, otherwise the first world with room. If someone takes the last seat just before you, the page looks again. The badge reads "Shared world 2" and so on in later worlds. A duel code from another world says so.
- **The game's own messages when the rollup refuses something.** MagicBlock's rollup reports a refused transaction with the program's error code only inside its message, which the page did not read. Players saw text like "custom program error: 0x20" instead of "This spot is recharging.", and a join refused because a world had just filled never looked again. The page now reads the code from the message.
- **Entering buildings.** Reach is measured from the nearest tile a building covers, not from its own tile. After you click a button (Menu, zoom, Atlas) the keyboard goes back to the game, so Space reaches the building beside you instead of pressing that button again. Space uses the nearest building or NPC in reach.
- **The Farlands**, the second solo map described above.
- **Bigger houses, taller trees.** Houses cover 2×1 tiles. Trees are taller and fuller, and no tree, rock or lantern grows through a building any more.
- **Joining again right after leaving** the shared world works. Before, a join within 12 seconds of the last one could be refused as already processed.
- **Smoother walking in the shared world.** Another player's avatar used to stop every half second or so while they walked. Their position arrives in short bursts, because the page sends the next steps only after the last ones are confirmed, and the avatar finished each burst before the next one arrived. It now plays their walk slightly behind, by a buffer that follows that player's own update rhythm: 1.25 times the usual gap between their updates, between 0.15 and 0.8 s. Measured on the live rollup from a machine about 350 ms away, the avatar stood still in 3.5 to 10.5% of frames while walking, against 37 to 50% before, and each tile appeared about 0.3 s later. It still starts moving as soon as the first update arrives.
- **Steps no longer wait for routine checks.** Before sending a step, the page sometimes first fetched a new blockhash (every 12 seconds) or checked again that the rollup is MagicBlock's (every 20 seconds). That step then reached the other players one round trip later: with a step every 1.6 seconds, one step in four. Both are now renewed in the background before they run out, between steps rather than alongside one. Measured on the live rollup over 40 single steps, the slowest tenth went from about 1.0 s to 0.6 s from a machine about 350 ms from the rollup. The Smaran kit bundled in this page differs from the kit in `../smaran/` only by `renewGuards()`, which the page calls for that renewal.

The page bundles three.js r149, Solana web3.js 1.98.4, hash-wasm, noble-ciphers and TweetNaCl, under their own licences: see [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md). Everything here is on Devnet, with test SOL only.
