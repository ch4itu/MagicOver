# Smaran Streaks program

A daily habit check-in for anyone with a Smaran username. Deployed on Solana Devnet as [`A98As1brXWi2DuJFChgat6bL9XbmaRRiF5vMFZswALoi`](https://explorer.solana.com/address/A98As1brXWi2DuJFChgat6bL9XbmaRRiF5vMFZswALoi?cluster=devnet). Its four groups run on MagicBlock's devnet rollup (devnet-as), the same rollup as the Smaran registry. The page is [`blitz/streaks`](../../blitz/streaks/).

## Rules

- **One check-in per habit per day.** Up to three habits per person, each with its own streak, best streak and total.
- **The program sets the day** from the rollup's clock, in the time zone of the person's first check-in. That time zone is fixed after the first check-in, so changing it cannot win back a missed day.
- **Missing a day starts the streak again at 1.** The best streak is kept.
- **The username is checked on chain.** It must belong to the signing key in the Smaran registry, the same check EternalBliss makes.
- **Players need no SOL.** Each check-in is a zero-fee rollup transaction signed by the player's own Smaran key.
- **Saving to Solana:** a group saves itself at most once an hour, paid from its own reserve (0.02 SOL each) through MagicBlock's fee vault. A check-in still works if the reserve runs low; it just waits to be saved.
- **Removing:** a person can change or remove a habit, or remove their whole record (instruction 5).

A streak shows that someone checked in, not that they did the habit.

## Build and test

- **Build:** run `cargo build-bpf` in this folder. It uses `programs/.cargo/config.toml` and the pinned toolchain. The on-chain target has no signed division, so the day maths uses unsigned arithmetic.
- **Tests:** run from outside `programs/`, for example `cd /tmp && cargo test --manifest-path <repo>/programs/streaks/Cargo.toml`. There are 18 host tests. They cover time zones, the one-a-day rule, streak resets, the fixed time zone, the registry name check, habit slots, removing a record, hourly saves and a low reserve.
- **Live test (23 to 24 Nov 2026):** two test players in two time zones signed in on the real page. They checked in, were refused a second same-day check-in from another tab, and changed and removed habits. A group save reached Solana Devnet. A fresh key claiming a registered username was refused.

Project code: [MIT](../../LICENSE).
