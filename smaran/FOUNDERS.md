# Smaran for founders

Players pay nothing for Smaran. The founder who runs the registry pays for their places, once. This page covers that cost, MagicBlock's charges included, and how a founder can earn from it.

Measured on the live Devnet registry on 4 November 2026. Solana rent and MagicBlock's rollup release (v1.1.1) are the same on mainnet.

## What it costs

| | Cost |
| --- | --- |
| Storage for a group of 35 places | 0.0522 SOL, once |
| MagicBlock's delegation deposit, per group | about 0.003 SOL, once per delegation |
| Save reserve, per group (suggested) | 0.003 SOL, once |
| Sign-up, sign-in proof, passphrase change | 0 SOL |
| **Per player** | **about 0.0017 SOL, once** (about 15 US cents at $92 per SOL) |

MagicBlock's charges, from its [fees page](https://docs.magicblock.gg/pages/ephemeral-rollups-ers/introduction/fees-and-commit-economics):

- Rollup transactions cost 0 SOL in the current release.
- A session charge of 0.003 SOL per delegation, plus 0.001 SOL per save to Solana after the first. Both come out of the delegation deposit when the group leaves the rollup, so together they never cost more than the deposit.
- From the 26th save in a delegation, 0.0001 SOL per save, taken at once from the group's reserve. This works because each group pays for its own saves; otherwise MagicBlock refuses the 11th save. Measured: the 11th save was refused when the group did not pay, and when it paid, saves 26 and 27 cost 100,000 lamports each.
- Smaran uses only MagicBlock's public rollup. The private rollup, which Magic Over's matches use, has no published price.

Running your own registry adds about 0.15 SOL once for the program. Room opens in steps: 140 places for 0.23 SOL, 420 for 0.70 SOL, 2,100 for 3.49 SOL, up to 8,820 for 14.66 SOL. A bot flood can only fill the step that is open.

## How a founder earns from it

Smaran's cost is per player and one time. Hosted logins such as Privy charge by monthly active users: Privy is $299 a month for 500 to 2,499 monthly active users ([privy.io/pricing](https://privy.io/pricing), 4 November 2026). That gap is the room to earn.

1. **Charge apps per new player.** Other apps use your registry and pay you once for each player they sign up, for example $0.30 against your cost of about $0.15. An app with 2,000 players would pay $600 once instead of $299 a month. Needs a program change first: today any app can add players to an open step for free, so the registry would have to record which app created each player.
2. **Run registries for other apps.** Deploy and run a separate registry for an app, for a setup fee plus the cost of its steps. The program needs no change; the kit just needs the registry address as a setting instead of fixed.
3. **Sell short names.** A name works in every app on the registry, so a short one is worth something, like a short web address. Ordinary names stay free. Needs a program change.
4. **Grants and prizes** for bringing new users to Solana, such as MagicBlock Forge's prizes.

Two habits keep the cost down: create a Smaran player only when someone needs one (Magic Over lets guests play the AI with a temporary key), and open steps only as they fill.

## Plan

1. **Now:** Devnet, free, 420 places, real players during Forge.
2. **Smaran on mainnet:** about 0.39 SOL for the registry program and the first 140 places, with Magic Over as the first app.
3. **First outside app:** a registry run for it (way 2), which needs no program change.
4. **Per-player billing (way 1),** once apps ask for it.
