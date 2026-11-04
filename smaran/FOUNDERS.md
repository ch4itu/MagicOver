# Smaran for founders

Players pay nothing for Smaran. Whoever runs the registry (the operator) pays for their places, once. Below: that cost with MagicBlock's charges, who controls what, and how apps and the operator each earn. Measured on the live Devnet registry on 4 November 2026; Solana rent and MagicBlock's rollup release (v1.1.1) are the same on mainnet.

## What it costs

| | Cost |
| --- | --- |
| Storage for a group of 35 places | 0.0522 SOL, once |
| MagicBlock's delegation deposit, per group | about 0.003 SOL, once per delegation |
| Save reserve, per group (suggested) | 0.003 SOL, once |
| Sign-up, sign-in proof, passphrase change | 0 SOL |
| **Per player** | **about 0.0017 SOL, once** (about 15 US cents at $92 per SOL) |

MagicBlock's charges, from its [fees page](https://docs.magicblock.gg/pages/ephemeral-rollups-ers/introduction/fees-and-commit-economics):

- Rollup transactions: 0 SOL in the current release.
- 0.003 SOL per delegation, plus 0.001 SOL per save to Solana after the first. Both are taken from the delegation deposit when a group leaves the rollup, and never more than the deposit.
- From a group's 26th save in a delegation, 0.0001 SOL per save from its reserve. A group that did not pay for its own saves would be refused its 11th. Both measured.
- Smaran uses only the public rollup. The private rollup, which Magic Over's matches use, has no published price.

A registry program costs about 0.15 SOL once. Room opens in steps: 140 places for 0.23 SOL, 420 for 0.70 SOL, 2,100 for 3.49 SOL, up to 8,820 for 14.66 SOL. A bot flood can only fill the step that is open.

## Who controls what

Names belong to the players who registered them: nobody, the admin included, can take, move or delete one. Anyone can open the next step by paying for it. The admin key can only take a group off the rollup (to refill its reserve) and force a save. The real control is the program's upgrade key, which can change the rules.

## How apps that use Smaran earn

Smaran is not something an app sells. It cuts costs and brings players.

1. **Lower costs on its own income.** No wallet, seed phrase or SOL at sign-up, and no monthly login bill. Hosted logins such as Privy charge by monthly active users: $299 a month for 500 to 2,499 ([privy.io/pricing](https://privy.io/pricing), 4 November 2026).
2. **Players it did not pay for.** Players that other apps brought to the same registry sign in to it at no cost. An app pays only for the players it adds, so every app on a shared registry gains from the others.
3. **Reselling short names,** if the registry sells them at a fixed price on-chain that any app can pay for a player. The app sets its own price and keeps the difference, like a web-address reseller. Needs the short-names change.
4. **Grants and prizes,** open to any builder, operators included, such as MagicBlock Forge's.
5. **Its own registry.** The code is MIT, so an app can run its own and earn as an operator. But its players then do not carry over to apps on other registries, unless the kit learns to read more than one registry. Not built.

## How the operator earns

1. **Charge apps per new player,** say $0.30 against a cost of about $0.15: an app with 2,000 players pays $600 once instead of $299 a month. Needs a program change, because today any app adds players free.
2. **Run registries for other apps,** for a setup fee plus the cost of their steps. Needs the registry address as a kit setting instead of fixed.
3. **Sell short names** at a fixed price, while ordinary names stay free. Needs a program change.

Other apps trust a shared registry more when its operator holds less control: prices fixed on-chain, steps anyone can open (already true), and later the upgrade key shared with partner apps or given up. Not built.

## Plan

1. **Now:** Devnet, free, 420 places, real players during Forge.
2. **Smaran on mainnet:** about 0.39 SOL for the registry program and the first 140 places. Magic Over is the first app.
3. **First outside app:** run a registry for it (operator's way 2).
4. **Per-player billing (operator's way 1),** once apps ask for it.
