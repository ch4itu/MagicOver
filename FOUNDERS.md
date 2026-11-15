# Smaran for founders

Players pay nothing for Smaran. Whoever runs the registry (the operator) pays for their places, once. Below: that cost with MagicBlock's charges, who controls what, and how the running costs are paid. Measured on the live Devnet registry on 4 November 2026; Solana rent and MagicBlock's rollup release (v1.1.1) are the same on mainnet.

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

Names belong to the players who registered them: nobody, the admin included, can take, move or delete one. Anyone can open the next step by paying for it, and anyone can refill a group once its save reserve runs low, so saving never depends on the admin key. Beyond that, the admin key can only take a group off the rollup at any time and force a save. The real control is the program's upgrade key, which can change the rules.

## How it is paid for

Non-intrusive ads, and nothing else.

- Ad space sits where it does not get in the way of play, such as scrolling boards around the ground in Magic Over. No pop-ups, no ads that interrupt a match.
- The ads pay for running Smaran and the games: registry storage, MagicBlock's charges and hosting.
- No crypto, tokens or in-game coins are sold. No wagers, stakes, entry fees or prize pools.
- Players pay nothing, ever, to sign up or play.

## Plan

1. **Now:** Devnet, free, 420 places, real players.
2. **Smaran on mainnet:** about 0.38 SOL for the registry program and the first 140 places. Magic Over is the first app.
3. **Ads:** non-intrusive ad space in Magic Over to cover the running costs.
