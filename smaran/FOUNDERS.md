# Smaran for founders

Players pay nothing for Smaran. Whoever runs the registry (the operator) pays for their places, once. Below: that cost with MagicBlock's charges, who controls what, and how it is paid for. Measured on the live Devnet registry on 4 November 2026; Solana rent and MagicBlock's rollup release (v1.1.1) are the same on mainnet.

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

Names belong to the players who registered them: nobody, the admin included, can take, move or delete one. Anyone can open the next step by paying for it, and anyone can refill a group once its save reserve runs low, with SOL from the [save fund](../programs/save-fund) that anyone can send SOL to, so saving never depends on the admin key. Beyond that, the admin key can only take a group off the rollup at any time and force a save. The real control is the program's upgrade key, which can change the rules.

## How it is paid for

- **Players pay nothing,** and nothing is sold to them: no tokens, no names, no fees.
- **Non-intrusive ads only.** Whoever runs the registry and its apps covers the chain costs with ads that never get in the way, such as a sponsor's name on a stadium board or a village sign. Never pop-ups, video ads or anything that interrupts play. There are no ads in this demo, and the plan is not proven yet.
- **Anyone can chip in, at one address.** Anyone can open the registry's next step by paying for it, and anyone can send SOL to the save fund, `9NLZkhdU3vG1kDi2RBW29pRyLcVW9MHSANWKMdE61myj`. The fund can only top up low save reserves of the Smaran apps' accounts, at most 0.1 SOL a day, and has no withdraw. A gift, not an investment: nothing comes back. So the registry keeps saving even if nobody earns from it.

## Why an app would use Smaran

1. **Lower costs.** No wallet, seed phrase or SOL at sign-up, and no monthly login bill. Hosted logins such as Privy charge by monthly active users: $299 a month for 500 to 2,499 ([privy.io/pricing](https://privy.io/pricing), 4 November 2026).
2. **Players it did not pay for.** Players that other apps brought to the same registry sign in to it at no cost. An app pays only for the players it adds, so every app on a shared registry gains from the others.
3. **Its own registry, if it wants one.** The code is MIT, so an app can run its own. Its players then do not carry over to apps on other registries, unless the kit learns to read more than one registry. Not built.

Other apps trust a shared registry more when its operator holds less control. Already true: anyone can open the next step, and anyone can refill a low reserve from the save fund. Not done yet: the upgrade key shared with partner apps or given up.

## Plan

1. **Now:** Devnet, free, 420 places.
2. **Mainnet:** about 0.38 SOL for the registry program and the first 140 places. Magic Over is the first app.
3. **More apps on the same registry,** each paying only for the players it adds.
4. **Less control for us:** the upgrade key shared with partner apps or given up.
