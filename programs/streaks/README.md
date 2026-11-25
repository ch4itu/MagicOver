# Smaran Streaks

A daily habit check-in for anyone with a Smaran username: up to three habits, one check-in a day each. It is the one app here that is not a game, to show that the same username and password work for everyday apps too. Deployed on Solana Devnet as [`A98As1brXWi2DuJFChgat6bL9XbmaRRiF5vMFZswALoi`](https://explorer.solana.com/address/A98As1brXWi2DuJFChgat6bL9XbmaRRiF5vMFZswALoi?cluster=devnet). Its four groups run on MagicBlock's devnet rollup (devnet-as), the same rollup as the Smaran registry. The page is [`blitz/streaks`](../../blitz/streaks/).

## Rules

- **One check-in per habit per day.** Up to three habits per person, each with its own streak, best streak and total.
- **The program sets the day** from the rollup's clock, in the time zone of the person's first check-in. That time zone is fixed after the first check-in, so changing it cannot win back a missed day.
- **Missing a day starts the streak again at 1.** The best streak is kept.
- **The username is checked on chain.** It must belong to the signing key in the Smaran registry, the same check EternalBliss makes.
- **Players need no SOL.** Each check-in is a zero-fee rollup transaction signed by the player's own Smaran key.
- **Saving to Solana:** a group saves itself at most once an hour, paid from its own reserve (0.02 SOL each) through MagicBlock's fee vault. A check-in still works if the reserve runs low; it just waits to be saved. Anyone can refill a group: once it has fewer than 10 paid saves left, anyone may take it off the rollup (instruction 8), have the [save fund](../save-fund) top it up and delegate it again (instruction 1), but only with no timed commits, the operator's setting. Above that line only the operator key can take a group off.
- **Removing:** a person can change or remove a habit, or remove their whole record (instruction 5).

A streak shows that someone checked in, not that they did the habit.

## How MagicBlock is used

- **Public rollup (devnet-as).** The four groups of 79 people are accounts delegated to MagicBlock's public rollup, next to the Smaran registry. Each check-in is a rollup transaction signed by the person's own Smaran key with 0 SOL.
- **The rollup's clock sets the day,** so changing the clock on a phone cannot add a check-in.
- **Saved to Solana at most once an hour,** paid by the group from its own reserve. The [save fund](../save-fund) refills a reserve that runs low.
- **The username is checked against the Smaran registry** on the same rollup.

## Build and test

- **Build:** run `cargo build-bpf` in this folder. It uses `programs/.cargo/config.toml` and the pinned toolchain. The on-chain target has no signed division, so the day maths uses unsigned arithmetic.
- **Tests:** run from outside `programs/`, for example `cd /tmp && cargo test --manifest-path <repo>/programs/streaks/Cargo.toml`. There are 20 host tests. They cover time zones, the one-a-day rule, streak resets, the fixed time zone, the registry name check, habit slots, removing a record, hourly saves, a low reserve, and the refill rules (hand-back by anyone once low, delegation by anyone only with the operator's setting).
- **Live test (23 to 24 Nov 2026):** two test players in two time zones signed in on the real page. They checked in, were refused a second same-day check-in from another tab, and changed and removed habits. A group save reached Solana Devnet. A fresh key claiming a registered username was refused. On 24 November, after an upgrade (build sha256 `0fdeff318c6e6dff8ba291789c74f73dac3124565d6f5eba6dd668c25959e271`; the deployed program with trailing zeros removed hashes like the build, to `3d1eb2d275feae0082d856350677025d861b0bcfb8b7617a27b25cf78e75532d`), a key that is not ours delegated group 3 again after the operator handed it back; a 30-second commit interval was refused (22) and the rollup copy matched the Solana copy.

Project code: [MIT](../../LICENSE).
