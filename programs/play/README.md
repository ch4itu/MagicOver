# Relics and achievements program

The program behind Magic Over's Boundary Hitter relic, shown in Magic Over, EternalBliss and SmaranWorld. Deployed on Solana Devnet as [`HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B`](https://explorer.solana.com/address/HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B?cluster=devnet). Its accounts run on MagicBlock's private rollup (devnet-tee).

**This folder is the deployed source.** `cargo build-bpf` here gives a file with sha256 `82d04f214807d25307697b86113b3c7b98e1536c2fa6a0bbf7dcbc7a60880105`. The deployed program, read with `solana program dump -u devnet HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B onchain.so`, is the same bytes plus zero padding: with trailing zeros removed, both hash to `f9802020b4c132d2163e4bf4a9fa61ab43403579651b1deaee77f167217e6ece` (upgrade of 24 November 2026).

## What it records

- **Achievements:** 4 accounts of 106 places. Boundary Hitter is claimed (instruction 7) by a player of a finished Magic Over match whose side hit at least one four or six, win, lose or tie. The program reads the match from the Magic Over table itself.
- **Relics:** 4 accounts of 91 places. For each achievement, MagicBlock VRF draws one relic (instruction 8). The relic records its holder's Smaran key.
- **Giving:** only the holder can give a relic to another key (instruction 10).
- **Saving to Solana:** after a change, the program asks MagicBlock to save the account in the same transaction. The account pays from its own reserve above rent (0.01 SOL, 100 paid saves, since the refill of 24 November 2026) through MagicBlock's fee vault. When the reserve is too low, the change stays on the rollup and anyone can save it later (instruction 2) once the reserve is refilled.
- **Anyone can refill:** once an account has fewer than 10 paid saves left, anyone may take it off the rollup (instruction 3), have the [save fund](../save-fund) top it up and delegate it again (instruction 1), but only to the same private rollup validator with a 30-second commit interval. A forced save of an account with nothing new (instruction 2 with the force byte) is open to anyone once a day per account, so it cannot burn the reserve; the operator may force one every 30 seconds.
- **Operator only:** set up accounts (0), take an account with a healthy reserve off the rollup (3), delegate with other settings (1), register a game and set or freeze relic odds (4 to 6).
- Instructions 11 to 13 (`vault.rs`) keep relic ownership in base-layer vault accounts, which are never delegated.

A relic records a deed. It has no price, and nothing in the program sells, stakes or wagers it.

`operator.bin` holds the operator's public key, `HDbeKypeJHXMM5Xs2RHKj8XGQgJeP4jnQ1Jiugf8mXPn`. It is a public key, not a secret. To build for another operator, write that operator's 32-byte public key there.

## How MagicBlock is used

- **Private rollup (devnet-tee).** The relic and achievement accounts are delegated to MagicBlock's private rollup, the same rollup as Magic Over's match tables. A claim reads the finished match straight from its Magic Over table there, so the program sees the match's own count of fours and sixes.
- **VRF.** For each achievement, MagicBlock VRF draws which relic the player gets (instruction 8).
- **Players pay nothing.** Claims and gifts are rollup transactions signed with the player's Smaran key.
- **Saved to Solana in the same transaction** (see Saving to Solana above), so apps that read Solana, such as EternalBliss and SmaranWorld, see every relic and its holder.

## Build and test

- **Build:** run `cargo build-bpf` in this folder. It uses `programs/.cargo/config.toml`, the pinned nightly toolchain and `sbpf-linker 0.2.5`.
- **Tests:** run from outside `programs/` so the on-chain build settings are not picked up, for example `cd /tmp && cargo test --manifest-path <repo>/programs/play/Cargo.toml`. There are 63 host tests, including the claim rules (`claim_is_personal_to_a_player_who_hit_a_boundary`, `a_four_alone_is_a_boundary`, `claim_is_lifetime_and_once_per_player`) and the refill rules (`anyone_can_hand_back_an_account_whose_reserve_is_low`, `a_stranger_can_delegate_an_account_back_to_the_same_validator`, `forced_commit_is_idempotent_rate_limited_and_permissionless`).

Project code: [MIT](../../LICENSE).
