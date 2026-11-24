# Smaran: four games and a habit app, one login

Our Solana Blitz v10 entry. **Play:** [ch4itu.github.io/MagicOver/blitz](https://ch4itu.github.io/MagicOver/blitz/)

Smaran is for Web2 people. A player picks a username and a password and starts playing. They never install a wallet, write down a seed phrase or buy crypto, and they never pay a fee. Behind the scenes each player gets a Solana key, locked with their password and kept on Solana, so the same username and password work on any device and in every app with Smaran login. (The guides call the password a passphrase: at least 16 characters.)

| App | What it is |
| --- | --- |
| [Magic Over](https://ch4itu.github.io/MagicOver/blitz/magic-over/) | 3D cricket duels on MagicBlock's private rollup. Hit a four or a six in a match, win or lose, and claim the Boundary Hitter relic. MagicBlock VRF draws which relic you get. You can give a relic to another player. |
| [EternalBliss](https://ch4itu.github.io/MagicOver/blitz/eternalbliss/) | RPG with quests, battles, shared worlds and duels on MagicBlock's public rollup. Relics earned in Magic Over show in its menu. |
| [SmaranWorld](https://ch4itu.github.io/MagicOver/blitz/smaranworld/) | One shared world, the Alderlands. Signed-in players see each other walk and chat, on one shared clock, so everyone sees the same time, season, people and traffic. 144 villages kept by relic holders, each with a monument and a sign its keeper writes. |
| [Ludo Freeflow](https://ch4itu.github.io/MagicOver/blitz/ludo-freeflow/) | Four-colour Ludo with no turns. Online matches roll MagicBlock VRF dice on the rollup. |
| [Smaran Streaks](https://ch4itu.github.io/MagicOver/blitz/streaks/) | Not a game: it shows the same login works for everyday apps too. Keep up to three daily habits with one check-in a day each. The program sets the day from its own clock in the time zone you start with, checks your username against the registry, and keeps a public streak board. Missing a day starts the streak again. |

A relic records a deed, like a medal. It is not money: it cannot be bought or sold, there is no marketplace and no wagering. It is saved on Solana under its holder's Smaran key, which is why Magic Over, EternalBliss and SmaranWorld can all show it.

## Why we built it

Entertainment Finance, MagicBlock's theme for this Blitz, is apps where people collect, compete and win things they actually own. Those apps need players first, and today owning anything on Solana starts with a wallet, a seed phrase and some SOL. Smaran is the front door for everyone who stops there:

- **People who will never touch crypto.** A username and a password, in any modern browser, on a phone or a computer, with no app to install. No wallet, no seed phrase, nothing to buy and no fees, ever.
- **People where crypto is hard to use.** In many countries buying or holding crypto is banned, restricted, heavily taxed or legally unclear. Here players never buy, hold or send any coin or token, there is no KYC step, and nothing has cash value: no stakes, no money prizes, no trading. So players never do what most crypto rules are written for (buying, holding, trading or paying with coins or tokens), and the apps can be offered wherever their pages, Solana and MagicBlock are reachable. A few places block blockchain services or such sites, and whoever runs the apps still follows local rules on data protection, age limits and advertising. This is a design note, not legal advice.
- **What players earn stays theirs.** A relic is stored on Solana under the player's own key, not in one game's database, so any app can show it and it outlives the game that gave it.

## Who pays

- **Players pay nothing**, ever. Every game action is a MagicBlock rollup transaction, which costs 0 SOL in MagicBlock's current release, and players sign with no SOL.
- **Whoever runs the apps pays the chain costs**, mostly once, as account deposits, the way an app pays for its servers. Today that is us.
- **Non-intrusive ads only.** The running costs are to be covered by ads that never get in the way, such as a sponsor's name on a stadium board or a village sign. Never pop-ups, video ads or anything that interrupts play. There are no ads in this demo, and the plan is not proven yet.
- **Anyone can chip in, at one address.** Saves to Solana are paid from each account's save reserve: SOL kept in the account above its storage deposit. Anyone can send SOL from any wallet to the **save fund**, `9NLZkhdU3vG1kDi2RBW29pRyLcVW9MHSANWKMdE61myj`. The fund can only top up these apps' accounts (logins, relics, achievements, streaks, the leaderboard, EternalBliss's worlds and character books), only when one runs low, and at most 0.1 SOL a day. It has no withdraw, so its SOL cannot go anywhere else. It is a gift, not an investment: nothing comes back. How it works: [programs/save-fund](../programs/save-fund).
- No token, no packs, no marketplace, no fees on players.

## Built to outlive us

- **No server.** The pages are static files on GitHub Pages and talk straight to Solana and MagicBlock. The code is open source (MIT), so anyone can host a copy, and a copy works with the same programs and the same accounts.
- **Records stay on Solana.** Logins, relics, achievements, streaks and EternalBliss's worlds are stored in Solana accounts whose storage is paid up front, so they stay as long as Solana keeps them, whatever happens to us or this site. Players can still sign in with the same username and password and get the same key back if we stop.
- **Saving keeps going without us.** When an account's save reserve runs low, anyone can hand it back to Solana, have the save fund top it up and put it back on its rollup. Every program allows this once fewer than ten paid saves are left, and only with the settings we use, so nobody can move an account to another rollup or make it save on a timer. A small script does all three steps ([programs/save-fund](../programs/save-fund)). Whoever runs it needs a little SOL for fees and MagicBlock's deposit (about 0.003 SOL per account each time). We run it today, and anyone can. While nobody runs it, play goes on on the rollups and only the copies on Solana wait.
- **As long as Solana and MagicBlock run.** That is the aim, and the design covers it: the pages need no server, the records are on Solana, and saving needs only SOL in the save fund and someone to run the refill script.
- **What it still depends on:** Solana; MagicBlock's rollups for real-time play, new sign-ups, matches and relic claims (sign-in still works from the copy on Solana when the rollup is down); SOL in the save fund; and our upgrade key, which can still change the programs' rules until we give it up at mainnet.
- **Devnet today.** Devnet can be reset, so all of this holds only once the apps run on mainnet.

## What it holds today

Measured on Solana Devnet on 23 November 2026.

| App | Players at once | Notes |
| --- | --- | --- |
| Magic Over | 16 in friend matches, or 8 against the AI | 8 match tables, reused between matches. Leaderboard: 140 players. |
| Ludo Freeflow | 32 online (8 tables of 4) | The local game against bots needs no sign-in and works offline. |
| SmaranWorld | 42 signed-in players shown to each other | Guests can look around without being shown. Chat keeps the last 24 lines. 144 villages, 160 sign places. |
| EternalBliss | 160 walking (5 worlds of 32) | 4 duel tables per world. The program allows up to 64 worlds. |
| Smaran Streaks | No limit on check-ins at once | 316 people (4 groups of 79), up to 3 habits each. A group saves itself to Solana at most once an hour from its own 0.02 SOL reserve. |

**Across all apps:**

- **Smaran accounts:** 420 places open, 70 used. The same program opens more in steps, with no upgrade: 2,100 places for about 2.8 SOL more, up to 8,820 for about 14 SOL more.
- **Relics:** 364 places. **Achievements:** 424 places. More need new accounts.

**Save reserves today** (why it lasts: [Built to outlive us](#built-to-outlive-us)):

- MagicBlock saves each account to Solana 25 times per delegation for free, then charges 0.0001 SOL per save.
  - Registry groups carry 0.005 SOL each (50 paid saves). That is enough to fill every place.
  - Relic and achievement accounts carry 0.01 SOL each (100 paid saves). On 24 November all nine were down to one paid save; a key that is not ours ran the refill script and the save fund topped them up.
  - Streaks groups carry 0.02 SOL each, the leaderboards 0.025 and 0.005 SOL, EternalBliss's worlds 0.076 to 0.1 SOL and its character books 0.05 SOL.
  - The save fund holds about 0.41 SOL, enough for about 40 more refills, at most 0.1 SOL a day.
  - When a reserve runs out, play still works on the rollup and the copy on Solana waits until the next refill.
- SmaranWorld's presence, chat and signs live on the rollup and are not saved to Solana. Saving them costs nothing, but a rollup reset would clear them.

## What it costs

**Now** (Devnet SOL, read from the accounts):

| Item | SOL |
| --- | --- |
| 9 programs: registry, relics, Magic Over, result register, Ludo, SmaranWorld, EternalBliss, Streaks, save fund | 2.26 |
| Smaran registry, 12 groups (420 places) | 0.69 |
| Relic and achievement accounts | 0.52 |
| EternalBliss worlds (character books not counted) | 0.62 |
| SmaranWorld room, chat and signs | 0.15 |
| Magic Over tables and leaderboard | 0.12 |
| Ludo tables | 0.03 |
| Streaks, 4 groups (rent plus a 0.02 SOL save reserve each) | 0.29 |
| Save fund (SOL given for future saves) | 0.41 |
| **Total** | **about 5.1** |

Almost all of it is held as rent deposits (SOL that Solana keeps in an account so the account stays stored), not burned. Registry groups cannot be closed today, so count those as spent. Rent is the same on mainnet, where it is real SOL.

**After the bigger plans** (estimate, not built):

| Item | SOL, once |
| --- | --- |
| Registry steps 2 and 3 (2,100 places, so 1,024 village owners fit) | about 2.8 |
| 1,024 villages, about 1 KB each | about 6 |
| 4,096 relic and 4,096 achievement places | about 4.3 |
| Presence for about 1,000 players at once (25 rooms) | about 0.5 |
| Program growth | about 1 |
| MagicBlock delegation deposits for the new accounts | 0.3 to 3 |
| **Total** | **about 15 to 18** |

Running cost on top: 0.0001 SOL per save to Solana once an account passes its 25 free saves in a delegation. The apps save on sign-ups, password changes, relic claims and gifts, and would save on village edits. So the cost follows those events, not hours played: 10,000 of them cost roughly 1 to 2 SOL.

## Programs on Solana Devnet

| Program | Address | Rollup |
| --- | --- | --- |
| Smaran registry | [`CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P`](https://explorer.solana.com/address/CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P?cluster=devnet) | public (devnet-as) |
| Relics and achievements | [`HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B`](https://explorer.solana.com/address/HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B?cluster=devnet) | private (devnet-tee) |
| Magic Over | [`HwXp4A62RG3nBkmxTTaHbSi1wNj8vaVqZ6Kn5VkYBswT`](https://explorer.solana.com/address/HwXp4A62RG3nBkmxTTaHbSi1wNj8vaVqZ6Kn5VkYBswT?cluster=devnet) | private (devnet-tee) |
| Ludo Freeflow | [`2i2REZWzDeAqKuJBEypH9fBtVZB6jmZbBRX7YH4fiETp`](https://explorer.solana.com/address/2i2REZWzDeAqKuJBEypH9fBtVZB6jmZbBRX7YH4fiETp?cluster=devnet) | public (devnet-as) |
| SmaranWorld | [`HPEA1Ht9rBnqEQVdaEQ9QuqwiWAbY6fgCKs2ZPuQvcCW`](https://explorer.solana.com/address/HPEA1Ht9rBnqEQVdaEQ9QuqwiWAbY6fgCKs2ZPuQvcCW?cluster=devnet) | public (devnet-as) |
| EternalBliss | [`6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb`](https://explorer.solana.com/address/6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb?cluster=devnet) | public |
| Smaran Streaks | [`A98As1brXWi2DuJFChgat6bL9XbmaRRiF5vMFZswALoi`](https://explorer.solana.com/address/A98As1brXWi2DuJFChgat6bL9XbmaRRiF5vMFZswALoi?cluster=devnet) | public (devnet-as) |
| Save fund (send SOL to `9NLZkhdU3vG1kDi2RBW29pRyLcVW9MHSANWKMdE61myj`) | [`PRU4HhX5x3Kf5wpTCt7amrjTbwXZ88NxU3DwwnHrcuo`](https://explorer.solana.com/address/PRU4HhX5x3Kf5wpTCt7amrjTbwXZ88NxU3DwwnHrcuo?cluster=devnet) | none, Solana only |

Source: [`programs/`](../programs) (relics: [`programs/play`](../programs/play), which rebuilds byte for byte into the deployed binary; Streaks: [`programs/streaks`](../programs/streaks); save fund and refill script: [`programs/save-fund`](../programs/save-fund)) and [`eternalbliss/program`](../eternalbliss/program).

No server: the pages are static files on GitHub Pages and talk straight to Solana and MagicBlock.

## Limits

- **Devnet prototype.** Devnet can reset and erase test data.
- **A streak shows a check-in, not the habit.** Streaks run on trust, the same as any habit app.
- **One person can make many usernames.** A small proof of work slows sign-up bots but cannot stop them. A login tied to a key is not proof of a unique person.
- **The programs check what they can see,** such as a boundary in the match's own count. That is not the same as cheat-proof: a modified page can still play badly or skip recording a result.
- **No password reset.** The locked copy of each key is public, so a weak password can be guessed offline. Smaran accounts hold no money.
- **Our upgrade key** can still change the programs' rules.
- **The save fund can be used up by abuse, at most 0.1 SOL a day.** Someone who makes many real saves (many sign-ups, password changes or relic gifts) empties reserves, which the fund then refills, up to its daily limit. Play goes on; saves to Solana wait for the next day.
- **Built to outlive us, not promised to.** See [Built to outlive us](#built-to-outlive-us): today our upgrade key still ties the apps to us, and saving needs someone to run the refill script.

## Next (designed, not built)

- **Give up or share the upgrade key** once the rules are settled, so no one can change them alone.
- **Mainnet.**
- A village for every player, opened by the first relic they earn themselves. Villages cannot be transferred.
- More deeds to earn: Six Storm, Homecoming, Duel Victor.
- Relic perks: an emblem and conveniences outside duels in EternalBliss; cosmetic only in Ludo Freeflow.

More detail: [Magic Over](../README.md), [Smaran](../smaran/README.md), [Smaran for founders](../smaran/FOUNDERS.md), [Ludo Freeflow](../ludo-freeflow/README.md).
