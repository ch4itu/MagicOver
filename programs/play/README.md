# Relics and achievements program

The program behind Magic Over's Boundary Hitter relic, shown in Magic Over, EternalBliss and SmaranWorld. Deployed on Solana Devnet as [`HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B`](https://explorer.solana.com/address/HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B?cluster=devnet). Its accounts run on MagicBlock's private rollup (devnet-tee).

**This folder is the deployed source.** `cargo build-bpf` here gives a file with sha256 `d0f98e331d5b134ad91f9da4b80ff6cf0483a4bc2feecf94dd62e4b0ccc719e6`. The deployed program, read with `solana program dump -u devnet HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B onchain.so`, is the same bytes plus zero padding: with trailing zeros removed, both hash to `2b40d4ea82f5b66d1ffcc3e4c46f7acaa01a5c1aabb24a776e86522127c4e487`.

## What it records

- **Achievements:** 4 accounts of 106 places. Boundary Hitter is claimed (instruction 7) by a player of a finished Magic Over match whose side hit at least one four or six, win, lose or tie. The program reads the match from the Magic Over table itself.
- **Relics:** 4 accounts of 91 places. For each achievement, MagicBlock VRF draws one relic (instruction 8). The relic records its holder's Smaran key.
- **Giving:** only the holder can give a relic to another key (instruction 10).
- **Saving to Solana:** after a change, the program asks MagicBlock to save the account in the same transaction. The account pays from its own reserve above rent (0.0001 SOL) through MagicBlock's fee vault. When the reserve is too low, the change stays on the rollup and anyone can save it later (instruction 2) once the reserve is refilled. Refilling means taking the account off the rollup (instruction 3), which only the operator key can do today; letting anyone do it once a reserve runs low, as the Smaran registry does, is planned.
- **Operator only:** set up and delegate accounts (0, 1), save and take an account off the rollup (3), register a game and set or freeze relic odds (4 to 6).
- Instructions 11 to 13 (`vault.rs`) keep relic ownership in base-layer vault accounts, which are never delegated.

A relic records a deed. It has no price, and nothing in the program sells, stakes or wagers it.

`operator.bin` holds the operator's public key, `HDbeKypeJHXMM5Xs2RHKj8XGQgJeP4jnQ1Jiugf8mXPn`. It is a public key, not a secret. To build for another operator, write that operator's 32-byte public key there.

## Build and test

- **Build:** run `cargo build-bpf` in this folder. It uses `programs/.cargo/config.toml`, the pinned nightly toolchain and `sbpf-linker 0.2.5`.
- **Tests:** run from outside `programs/` so the on-chain build settings are not picked up, for example `cd /tmp && cargo test --manifest-path <repo>/programs/play/Cargo.toml`. There are 61 host tests, including the claim rules: `claim_is_personal_to_a_player_who_hit_a_boundary`, `a_four_alone_is_a_boundary` and `claim_is_lifetime_and_once_per_player`.

Project code: [MIT](../../LICENSE).
