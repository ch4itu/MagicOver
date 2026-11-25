# Smaran: one login for every app

> **Smaran: one username and password for every Solana app. No wallet, no seed phrase, no crypto.**

## Problem

To use a Solana app, or own anything in it, a person first needs a wallet, a seed phrase and some SOL. Most people stop there. In many countries crypto is also hard or risky to use.

## Solution

Smaran gives each person a Solana key, locked with their password and kept on Solana itself. The same username and password open it in every app that supports Smaran, games or not, on any device. What someone earns in one app (a relic) stays theirs and shows in the others. Here it runs four games and a daily habit app.

## Sustainability

People never pay. Running costs are meant to come from non-intrusive ads only. One [save fund](../programs/save-fund), an address anyone can send SOL to, keeps the apps saving to Solana. There is no server to switch off, so the apps keep working as long as Solana and MagicBlock run ([what that depends on](#built-to-outlive-us)).

## How MagicBlock is used

| App | On MagicBlock |
| --- | --- |
| Smaran login | The registry of usernames lives on the public rollup. Sign-up and password changes are rollup transactions signed with 0 SOL, then saved to Solana. |
| Magic Over | Every match runs on the private rollup, so the bowler's hidden variation stays hidden from the batter until the ball is played. VRF picks the AI's choices. |
| Relics and achievements | On the private rollup. The program reads the finished match from Magic Over's table there, and VRF draws which relic a player gets. |
| EternalBliss | Shared worlds, duels and character books on the public rollup, saved to Solana every few minutes. |
| SmaranWorld | Where players stand, what they say and each village's sign, on the public rollup. Who keeps a village comes from the relic records. |
| Ludo Freeflow | Every table on the public rollup, and every die from VRF. |
| Smaran Streaks | Check-ins on the public rollup, saved to Solana at most once an hour. |
| Save fund | On Solana itself: it refills the accounts that pay MagicBlock for their saves to Solana. |

Every action people take is a rollup transaction that costs them nothing. Each app's README explains its part.

**Play:** [ch4itu.github.io/MagicOver/blitz](https://ch4itu.github.io/MagicOver/blitz/) · **Video:** [Smaran demo](https://ch4itu.github.io/MagicOver/media/smaran-blitz-demo.mp4)

Make one account with a username and a password, and use it in every app below, on a phone or a computer. There is no wallet to install, no seed phrase to write down, no crypto to buy and no fees. What you win is yours: a relic you earn in the cricket game also shows in the adventure game and in the shared world, and it stays with you even if a game closes.

| App | What you do there |
| --- | --- |
| [Magic Over](https://ch4itu.github.io/MagicOver/blitz/magic-over/) | Bat in a 3D cricket duel against the computer or a friend. Sign in, hit one four or six in any match, win or lose, and claim a relic. You can give a relic to another player. |
| [EternalBliss](https://ch4itu.github.io/MagicOver/blitz/eternalbliss/) | An adventure game with quests, battles, shared worlds and duels. Your relics show in its menu. |
| [SmaranWorld](https://ch4itu.github.io/MagicOver/blitz/smaranworld/) | A shared world, the Alderlands: 144 villages spread across fields, forests, hills, lakes, desert and tundra, where signed-in players see each other walk and chat, on one clock for everyone. Each village is kept by a relic holder, with a monument and a sign the keeper writes. |
| [Ludo Freeflow](https://ch4itu.github.io/MagicOver/blitz/ludo-freeflow/) | Ludo with no turns: everyone rolls and moves at once. Pick a table online, and every die is a fair random draw. |
| [Smaran Streaks](https://ch4itu.github.io/MagicOver/blitz/streaks/) | Not a game: keep up to three daily habits, one check-in a day. It shows the same login works for everyday apps too. |

A relic is like a medal: a record of something you did. It is not money. It cannot be bought or sold, there is no marketplace and no betting.

**How it works, in one paragraph.** Each account is a key on Solana. Your browser locks it with your password and keeps the locked copy on Solana itself, so the same username and password unlock it in any app, on any device, and there is no login server that could shut down. The games run on MagicBlock, which makes every move instant and free for the player. Details for judges and developers are [further down](#for-the-judges-magicblock-solana-blitz-v10).

## Why we built it

Games where people collect, compete and win things they really own (MagicBlock calls this Entertainment Finance) need players first. Today owning anything on Solana starts with a wallet, a seed phrase and some SOL, and most people stop there. Smaran is the front door for them:

- **People who will never touch crypto.** A username and a password, in any modern browser, on a phone or a computer, with no app to install. No wallet, no seed phrase, nothing to buy and no fees, ever.
- **People where crypto is hard to use.** In many countries buying or holding crypto is banned, restricted, heavily taxed or legally unclear. Here players never buy, hold or send any coin or token, there is no KYC step, and nothing has cash value: no stakes, no money prizes, no trading. So players never do what most crypto rules are written for (buying, holding, trading or paying with coins or tokens), and the apps can be offered wherever their pages, Solana and MagicBlock are reachable. A few places block blockchain services or such sites, and whoever runs the apps still follows local rules on data protection, age limits and advertising. This is a design note, not legal advice.
- **What players earn stays theirs.** A relic is stored on Solana under the player's own key, not in one game's database, so any app can show it and it outlives the game that gave it.

## Who pays

- **Players pay nothing**, ever. Every action in every app is a MagicBlock rollup transaction, which costs 0 SOL in MagicBlock's current release, and players sign with no SOL.
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

## For the judges: MagicBlock Solana Blitz v10

**New in this hackathon (20 to 27 November 2026)**

- **Relics and achievements** ([`programs/play`](../programs/play)): one program that records what a player did and which relic they hold. Magic Over checks the match on the private rollup and MagicBlock VRF draws which relic you get. Relics can be given to another player.
- **Relics shown across apps:** EternalBliss's menu and SmaranWorld's villages read the same relic records, under the same Smaran key.
- **SmaranWorld:** the shared world, its 144 villages in open country, chat and shared clock (built on our earlier city-builder engine). The land is made from one seed on every page, so it needs no map file and can grow past 1,000 villages ([SmaranWorld README](smaranworld/README.md)).
- **Ludo Freeflow online:** tables of four on MagicBlock with VRF dice, sign-in on the page and a list of every table (the local game is older).
- **Smaran Streaks**, the non-game app.
- **The save fund** ([`programs/save-fund`](../programs/save-fund)): one address anyone can fund, so saves to Solana keep going without us, and the rule changes that let anyone refill a low account.
- **One sign-in card and one look** across all apps.

**Reused from earlier work:** Magic Over and its result register, the Smaran login and its registry (MagicBlock Forge), EternalBliss's game on MagicBlock, and Ludo Freeflow's local game.

**How MagicBlock is used**

- **Private rollup (devnet-tee):** Magic Over's matches (the bowler's variation stays hidden from the batter until the ball is played), relics and achievements, and the leaderboard.
- **Public rollup (devnet-as):** Smaran logins, EternalBliss, SmaranWorld's rooms and chat, Ludo Freeflow's tables and Streaks.
- **VRF:** which relic a player gets, and every Ludo die.
- **Free play:** every player action is a rollup transaction that costs the player nothing. Each shared account pays MagicBlock for its own saves to Solana from a small reserve, which the save fund refills.

**Try it in two minutes:** open [Magic Over](https://ch4itu.github.io/MagicOver/blitz/magic-over/), press Sign in and create an account, play Bat vs AI and hit a four or a six, then claim the relic on the result screen. Open [SmaranWorld](https://ch4itu.github.io/MagicOver/blitz/smaranworld/) or [EternalBliss](https://ch4itu.github.io/MagicOver/blitz/eternalbliss/) and sign in with the same username and password: the relic is there.

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
- SmaranWorld past 144 villages: the map already grows ring by ring (a 1,024-village world is made and tested, and the first 144 villages keep their names and places). The SmaranWorld program still limits signs to 144 villages and stores the village number in one byte, so it needs a small upgrade first.
- More deeds to earn: Six Storm, Homecoming, Duel Victor.
- Relic perks: an emblem and conveniences outside duels in EternalBliss; cosmetic only in Ludo Freeflow.

More detail: [Magic Over](../README.md), [Smaran](../smaran/README.md), [Smaran for founders](../smaran/FOUNDERS.md), [EternalBliss](../eternalbliss/README.md), [SmaranWorld](smaranworld/README.md), [Ludo Freeflow](../ludo-freeflow/README.md), [Smaran Streaks](../programs/streaks/README.md), [relics](../programs/play/README.md), [save fund](../programs/save-fund/README.md).
