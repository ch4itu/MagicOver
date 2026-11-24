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

A relic records a deed, like a medal. It is not money: it cannot be bought or sold, there is no marketplace and no wagering. It is saved on Solana under its holder's Smaran key, which is why every app can show it.

## Who pays

- **Players pay nothing**, ever. Every game action is a MagicBlock rollup transaction, which costs 0 SOL in MagicBlock's current release, and players sign with no SOL.
- **The operator pays the chain costs**, mostly once, as account deposits, the way an app pays for its servers. Today that is us.
- **Plan:** sponsorship and non-intrusive placements, such as a sponsor's name on a stadium board or a village sign. Never ads that interrupt play. There are no ads in this demo, and the plan is not proven yet.
- No token, no packs, no marketplace, no fees on players.

## Where it can be used

Players never buy, hold or send crypto. There is no wallet, no KYC step and nothing with cash value: no stakes, no money prizes, no trading. So rules about buying or trading crypto do not come into play for players, and Smaran works wherever its web pages, Solana and MagicBlock are reachable. A few places restrict blockchain services or block such sites, and an operator still follows local rules on data protection, age limits and advertising. This is a design note, not legal advice.

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

**How long it lasts:**

- Every account is paid for with a deposit, so it stays as long as the network keeps it. Devnet can be reset; lasting records need mainnet.
- MagicBlock saves each account to Solana 25 times per delegation for free, then charges 0.0001 SOL per save.
  - Registry groups carry 0.005 SOL each (50 paid saves). That is enough to fill every place, and anyone can refill them.
  - Relic and achievement accounts carry 0.0001 SOL each (one paid save). Right now the four relic accounts have about 95 saves left between them, roughly 90 more claims or gifts. New achievements fill one account first, which has about 18 saves left.
  - Past that, claims and gifts still work on the rollup. Their copy on Solana waits until the operator refills the reserves.
- SmaranWorld's presence, chat and signs live on the rollup and are not saved to Solana. Saving them costs nothing, but a rollup reset would clear them.

## What it costs

**Now** (Devnet SOL, read from the accounts):

| Item | SOL |
| --- | --- |
| 8 programs: registry, relics, Magic Over, result register, Ludo, SmaranWorld, EternalBliss, Streaks | 2.10 |
| Smaran registry, 12 groups (420 places) | 0.69 |
| Relic and achievement accounts | 0.43 |
| EternalBliss worlds (character books not counted) | 0.62 |
| SmaranWorld room, chat and signs | 0.15 |
| Magic Over tables and leaderboard | 0.12 |
| Ludo tables | 0.03 |
| Streaks, 4 groups (rent plus a 0.02 SOL save reserve each) | 0.29 |
| **Total** | **about 4.4** |

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

Source: [`programs/`](../programs) (relics: [`programs/play`](../programs/play), which rebuilds byte for byte into the deployed binary; Streaks: [`programs/streaks`](../programs/streaks)) and [`eternalbliss/program`](../eternalbliss/program).

No server: the pages are static files on GitHub Pages and talk straight to Solana and MagicBlock.

## Limits

- **Devnet prototype.** Devnet can reset and erase test data.
- **A streak shows a check-in, not the habit.** Streaks run on trust, the same as any habit app.
- **One person can make many usernames.** A small proof of work slows sign-up bots but cannot stop them. A login tied to a key is not proof of a unique person.
- **The programs check what they can see,** such as a boundary in the match's own count. That is not the same as cheat-proof: a modified page can still play badly or skip recording a result.
- **No password reset.** The locked copy of each key is public, so a weak password can be guessed offline. Smaran accounts hold no money.
- **Our upgrade key** can still change the programs' rules.
- **Built to outlive us, not promised to.** Logins and relics live on Solana and the pages are open source, but new sign-ups, matches and relic claims need MagicBlock's rollups.

## Next (designed, not built)

- A village for every player, opened by the first relic they earn themselves. Villages cannot be transferred.
- More deeds to earn: Six Storm, Homecoming, Duel Victor.
- Relic perks: an emblem and conveniences outside duels in EternalBliss; cosmetic only in Ludo Freeflow.

More detail: [Magic Over](../README.md), [Smaran](../smaran/README.md), [Smaran for founders](../smaran/FOUNDERS.md), [Ludo Freeflow](../ludo-freeflow/README.md).
