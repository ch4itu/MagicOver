# Smaran for founders

Smaran is free for players because the app pays. This page shows what that costs, measured on the live registry, and the ways an app can earn it back. It is for a founder deciding whether to use Smaran, and for anyone asking whether Smaran can pay for itself.

## The short answer

- **A player costs the app about 0.0017 SOL, once.** At about $92 per SOL (4 November 2026) that is about 15 US cents. There is no monthly fee and no charge per active user.
- **Playing costs nothing more.** Sign-up and every move on MagicBlock's rollup cost 0 SOL in MagicBlock's current release.
- **To break even, each player has to bring in about 15 cents over their whole time in the app.** For an app that earns from ads, items or sponsors that is a low bar, but it is not zero: a player who signs up and never comes back still costs 15 cents. That is why Magic Over lets people play as a guest first, and only creates a Smaran player when they want something that needs one, like a friend match.

## What the app pays

Measured on the live Devnet registry on 4 November 2026. Solana rent was checked and is the same on mainnet, and MagicBlock's Devnet rollup runs the same release (v1.1.1) as its mainnet rollup.

| Item | Cost | When |
| --- | --- | --- |
| Storage for a group of 35 names (one 10,144-byte account) | 0.0522 SOL | Once, when the sponsor opens the group |
| MagicBlock's delegation deposit for the group | about 0.0029 SOL | Once per delegation. MagicBlock keeps it when the group leaves the rollup |
| Save reserve in the group | 0.003 SOL (suggested) | Once. Used only from the 26th save |
| Sign-up, sign-in proof, passphrase change | 0 SOL | Rollup transactions |
| Saving the group to Solana, saves 1 to 25 of a delegation | 0 SOL | After each sign-up or passphrase change |
| Saving the group to Solana, from save 26 | 0.0001 SOL each | Paid from the group's reserve |
| **Per player** | **about 0.0017 SOL** | **Once** |

- Storage and the deposit are held in accounts, not burned as fees. But the registry has no way to close a group today, so plan as if they are spent.
- MagicBlock covers the first 10 saves of an account per delegation itself and refuses the 11th, unless the account pays for its own saves. Each Smaran group pays from its reserve, which lifts that limit. We tested one group to 28 saves: saves 1 to 25 cost nothing, and saves 26 and 27 cost 100,000 lamports (0.0001 SOL) each.
- An app that runs its own registry also pays once to deploy the registry program, about 0.15 SOL. An app that uses a registry someone else runs pays only for its players' places, or nothing if the registry's operator covers them.
- The cost is fixed in SOL, so the dollar cost moves with the SOL price.

## Growing in steps

The registry opens room in steps, and the sponsor pays for a step when it opens it. Names never move between steps.

| Step | Groups | New places | Cost to open | Places so far | Spent so far | At $92 per SOL |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 4 | 140 | 0.23 SOL | 140 | 0.23 SOL | $21 |
| 1 | 8 | 280 | 0.47 SOL | 420 | 0.70 SOL | $64 |
| 2 | 16 | 560 | 0.93 SOL | 980 | 1.63 SOL | $150 |
| 3 | 32 | 1,120 | 1.86 SOL | 2,100 | 3.49 SOL | $321 |
| 4 | 64 | 2,240 | 3.72 SOL | 4,340 | 7.21 SOL | $663 |
| 5 | 128 | 4,480 | 7.44 SOL | 8,820 | 14.66 SOL | $1,348 |

Because a step is paid for when it opens, a flood of bot sign-ups can only use up the step that is open, and the sponsor decides when to open the next one. The registry stops at 8,820 names. More would need a second registry or a program change.

## Compared with a hosted login service

Privy's published prices ([privy.io/pricing](https://privy.io/pricing), checked 4 November 2026): free up to 499 monthly active users, $299 a month for 500 to 2,499, $499 a month for 2,500 to 9,999, and above 10,000, $0.05 per monthly active user plus a $2,000 base fee.

Take 2,000 players who keep playing:

- **Hosted service:** $299 every month, $3,588 a year.
- **Smaran:** steps 0 to 3 (2,100 places) cost 3.49 SOL, about $321, once.

This is not a like-for-like comparison. A hosted service gives email and social logins, recovery when a player forgets, wallets safe enough for real money, and a company to call. Smaran has none of that. And for small apps the cash comparison flips: under 500 monthly active users Privy is free, while Smaran still costs about 15 cents a player. What Smaran offers instead is no company in the middle, no monthly bill that grows as the app succeeds, and players who can take the same identity to other apps on the same registry.

## How a founder earns it back

These are options, not tested business models. The rates in the examples are assumptions to show the scale, not market data.

| Way to earn | What covers one player's 15 cents |
| --- | --- |
| **Cosmetic items** (skins, bats, kits, stadiums) | If one player in N buys an item that nets you $X after store fees, you earn X/N a player. An item that nets $1.50 needs one buyer in 10 players. One that nets $3 needs one buyer in 20. |
| **Ads** (rewarded or banner) | At an assumed $3 per thousand views, a player has to see 50 ads over their time in the app. |
| **Season pass or subscription** | If one player in 100 pays $1.99 a month ($1.39 after a 30% store share), that is 1.4 cents a player a month, which covers the 15 cents in under a year. |
| **Sponsors** | A sponsor pays for a tournament with prizes and no entry fee. $150 covers 1,000 players. |
| **Partner apps** | A player one app paid for can sign in to every other app on the same registry at no extra cost, so partner apps can share the cost of a registry. |

Two things keep the cost low whatever the earning model:

- **Spend only on players who stay.** Let people try the app as guests with a temporary key, and create a Smaran player only when they want something that needs one: friend matches, saved progress, a name on a leaderboard.
- **Open steps as they fill,** not all at once.

**India.** The Promotion and Regulation of Online Gaming Act, 2025 bans online money games, whether of skill, chance or both, advertising them, and payments for them. Its Rules took effect on 1 May 2026. E-sports and online social games are allowed, with registration for categories the government notifies. So in India: no entry fees with cash prizes and no wagers. Ads, cosmetic items, subscriptions and sponsor-paid prizes with no entry fee are the kinds of income that fit a social game, but take legal advice for your own case; this page is not legal advice. Source: Press Information Bureau, [A New Era of Online Gaming Governance](https://static.pib.gov.in/WriteReadData/specificdocs/documents/2026/apr/doc2026430859401.pdf), 30 April 2026.

## How Smaran itself can earn

The same idea works one level up, for whoever runs Smaran. None of this is built yet.

- **Hosted registries:** set up and run a registry for other apps, for a setup fee.
- **Per player above a free tier:** free for an app's first few hundred players, then a small one-time fee per new player. Still one time, not monthly.
- **Premium names:** short names (say 3 or 4 letters) for a price, while ordinary names stay free. This needs a program change; today every name is free and first come, first served.
- **Grants and programmes** for tools that bring new users to Solana, such as the prizes in MagicBlock Forge.

## A plan in stages, with Magic Over as the example

1. **Now, on Devnet, free.** 420 places open. Real players during MagicBlock Forge, to measure how many guests turn into players and how many come back.
2. **Mainnet launch, about 1.53 SOL (about $140).** About 0.94 SOL stays locked: the three programs, step 0 with 140 places, 8 match tables and the leaderboard. About 0.53 SOL comes back once the programs are deployed, and a little goes on transaction fees.
3. **Grow.** Open steps as they fill: 420 places for 0.70 SOL, 2,100 for 3.49 SOL, all 8,820 for 14.66 SOL. Switch on at least one way to earn before step 3, where the spend passes about $300.

## What could change these numbers

- **The SOL price.**
- **MagicBlock's prices.** Rollup transactions are free "in the current release", and MagicBlock can change that. Its pricing page does not list prices for its private rollup, which Magic Over's match tables use. Smaran itself uses only the public rollup.
- **Solana rent.** It is a network setting and it has been lowered before: an empty account used to need 890,880 lamports and now needs 650,240.
- **How many players come back.** A one-time cost is only cheap if the players stay.

How these numbers were measured: scripts against the live Devnet registry and Solana mainnet on 4 November 2026 (rent, deposits, the 10-save limit and the cost of each save), checked against MagicBlock's [fees page](https://docs.magicblock.gg/pages/ephemeral-rollups-ers/introduction/fees-and-commit-economics) and [pricing page](https://docs.magicblock.gg/pages/get-started/introduction/pricing).
