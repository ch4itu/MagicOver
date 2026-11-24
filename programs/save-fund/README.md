# Smaran save fund

One address anyone can send SOL to, so the Smaran apps keep saving to Solana without us.

- **Fund address (send SOL here):** [`9NLZkhdU3vG1kDi2RBW29pRyLcVW9MHSANWKMdE61myj`](https://explorer.solana.com/address/9NLZkhdU3vG1kDi2RBW29pRyLcVW9MHSANWKMdE61myj?cluster=devnet), Solana Devnet. Any wallet can send to it with a plain transfer.
- **Program:** [`PRU4HhX5x3Kf5wpTCt7amrjTbwXZ88NxU3DwwnHrcuo`](https://explorer.solana.com/address/PRU4HhX5x3Kf5wpTCt7amrjTbwXZ88NxU3DwwnHrcuo?cluster=devnet). The fund is its account `["smaran-save-fund"]`.

## Why it exists

The apps keep their shared accounts on MagicBlock rollups, where play is free and instant: Smaran logins, relics, achievements, streaks, Magic Over's leaderboards, EternalBliss's worlds and character books. Each account pays MagicBlock for its own saves to Solana from a small reserve of SOL above its storage deposit. The first 25 saves of each delegation are free, then each save costs 0.0001 SOL. When a reserve runs out, play goes on, but the copy on Solana stops updating.

The save fund keeps those reserves topped up from one place. Anyone who wants the apps to keep going can help by sending SOL to one address. It is a gift, not an investment: nothing comes back to whoever sends SOL.

## What it can do, and nothing else

- **Top up one listed account,** only while that account is on Solana (not on a rollup) and low: under 0.001 SOL above rent, which means fewer than 10 paid saves left. It tops the account up to 0.01 SOL above rent, which is 100 paid saves.
- **Pay at most 0.1 SOL a day** (UTC) in all, and always keep its own rent.
- **Nothing else.** There is no withdraw, no admin and no other instruction. Its SOL can only go into the accounts below.

| App | Accounts the fund may top up |
| --- | --- |
| Smaran logins | registry groups `["sabha-reg", group]` and `["sabha-reg", step, group]` of `CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P` |
| Relics and achievements | `["play-reg"]`, `["play-ach", n]`, `["play-rel", n]` of `HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B` |
| Smaran Streaks | groups `["smaran-streaks", n]` of `A98As1brXWi2DuJFChgat6bL9XbmaRRiF5vMFZswALoi` |
| Magic Over leaderboards | `["mo-board", n]` of `BcS2Gb1Gihfzqs1UkViNnbQzjGcCGyvXu81VhwsFrUzU` |
| EternalBliss | worlds `["eb-land", n]` and character books `["eb-book", n]` of `6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb` |

The other apps' accounts (Magic Over's tables, Ludo Freeflow's tables, SmaranWorld's rooms and signs) do not pay for saves, so they need no reserve.

## How a refill happens

1. **Hand back.** On the rollup, anyone may hand a low account back to Solana. Each program allows that only once fewer than 10 paid saves are left; above that, only our key can.
2. **Top up.** On Solana, anyone may ask the fund to refill it (instruction 1, `[fund, account]`). The caller pays only the transaction fee.
3. **Put back.** Anyone may delegate the account to its rollup again, but only to the same validator and with the same settings we use. So nobody can move an account to another rollup or make it save on a timer. The caller pays MagicBlock's delegation deposit, about 0.003 SOL.

The refill script in [`refill/`](refill) does all three for every listed account. Without `--apply` it only reports.

```
cd refill
npm install
node refill.js                                 # report every account's reserve
node refill.js --apply --keypair my-key.json   # hand back, refill and put back where needed
```

Whoever runs it needs a little SOL for fees and deposits: about 0.000005 SOL a Solana transaction and about 0.003 SOL each time an account goes back on a rollup. We run it today. Anyone can run it, from any computer, as often as they like, without our key or permission.

## Honest limits

- **Someone has to run the script.** A Solana program runs only when a transaction calls it, so the fund cannot act on its own. While no one runs the script, play goes on on the rollups and only the copies on Solana wait.
- **Abuse can use up a day's refills.** Someone who makes many real saves (many sign-ups, password changes or relic gifts) empties reserves, which the fund then refills, up to 0.1 SOL a day. Play is not affected. Saves to Solana wait for the next day.
- **Our upgrade key can still change this program,** and the others. We plan to give it up at mainnet. Until then this rests on trusting us not to.
- **Devnet can be reset.**

## Checked

- **Host tests, 10 of 10:** the fund is created once at its own address, and SOL sent before that is kept; a low listed account is topped up to 100 saves; exactly 10 saves left is not low; only the listed address, and only while it is on Solana; every list entry derives its program's address; at most 0.1 SOL a day, starting again the next day; the fund keeps its rent; only the real fund pays; there is no other way out. Every successful call is also checked for the runtime's rules: lamports are conserved, and only the fund or a signing payer loses any.
- **Live on Devnet, 24 November 2026.** A key that is not ours ran the refill script. All 9 relic and achievement accounts were down to 1 paid save. Each was handed back, refilled to 100 paid saves and put back on the private rollup, and its rollup copy matched its Solana copy. A save after the refill reached Solana. The fund refused a delegated account (error 3), seeds naming another account (9), an unknown entry (40), a healthy account on Solana (41) and any other instruction (11). The same key put a registry group, a Streaks group and a leaderboard back on their rollups after our key handed them back, and was refused other settings (16, 22 and 2).

## Build and test

- **Build:** `cargo build-bpf` in this folder. It uses `programs/.cargo/config.toml`, the pinned toolchain and `sbpf-linker 0.2.5`. The build has sha256 `6530dd487a90c26e8b7bc14840a2aeee3238e710810b32e54311bf4f79305039`. The deployed program (`solana program dump -u devnet PRU4HhX5x3Kf5wpTCt7amrjTbwXZ88NxU3DwwnHrcuo onchain.so`) is the same bytes plus zero padding: with trailing zeros removed, both hash to `33f0b331d5b249463396a18378d28c5e7a76691bc1e836aae0d428552028e66f`.
- **Tests:** run from outside `programs/`, for example `cd /tmp && cargo test --manifest-path <repo>/programs/save-fund/Cargo.toml`.

Project code: [MIT](../../LICENSE).
