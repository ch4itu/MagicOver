# Magic Over :: Cricket on MagicBlock ER · PER · VRF

**Six balls. Two wickets. Open the game and play. No wallet app, seed phrase or SOL required.**

[Play Magic Over](https://ch4itu.github.io/MagicOver/) · [Watch the pitch and demo](https://cdn.jsdelivr.net/gh/ch4itu/MagicOver@main/media/pitch.mp4)

Magic Over is a browser cricket game on **Solana Devnet**. Try **Bat vs AI** immediately, or play a friend using a username and strong passphrase. Bowl a hidden variation, time your shot, swap roles for the chase, and choose a rematch with the same friend. Recorded friend results feed a shared persistent leaderboard.

## Easy onboarding

- **Try first:** Bat vs AI needs no registration. The browser creates a temporary guest signing key.
- **Create a lasting player ID:** a registered username restores the same player key across visits and devices. Your username and passphrase recover it locally.
- **Play without wallet prompts:** the browser signs moves automatically. Players need **0 SOL** in the current Devnet registration and play flow; sponsors cover infrastructure costs.

The player key is random and independent of the passphrase. The browser encrypts a recovery copy locally using **XChaCha20-Poly1305**, with a wrapping key derived by **Argon2id: 64 MiB, three passes**. The password and plaintext key are not published. The unlocked key stays in memory; there is no forgotten-passphrase reset.

**Registration is automatic, in two stages.** Empty registry accounts already exist on Solana Devnet and are delegated to MagicBlock. A new encrypted username record is written to the **public ER first**. The browser then requests a separate commit back to **Solana Devnet** in the background while the player enters the game. The current onboarding confirms ER registration, but does not verify the later Solana confirmation. Recovery reads the ER first and can fall back to the last Solana copy. Registered names persist within this Devnet registry; Devnet resets can erase test data.

## How we use MagicBlock

| Feature | Use in Magic Over |
| --- | --- |
| **Public ER** | Runs the delegated username registry, enabling registration without player funding. |
| **Private ER / PER** | Executes create, join, bowl, shot and resolution actions across eight reusable match tables. |
| **Private account permissions** | A separate ball account holds the bowler's hidden variation. Pace, line and length are visible; the hidden variation is revealed after resolution. |
| **VRF** | Selects AI targets and deliveries. The program checks the expected callback signer, table and sequence. Hidden variation also mixes a private secret. |
| **Zero-SOL player actions** | Unfunded player keys sign rollup moves automatically. Sponsor costs still exist. |
| **Solana commits** | Each newly recorded friend result requests a save of leaderboard totals to Solana Devnet. Registration also requests a registry commit. Match tables remain delegated for fast reuse. |

The game grades timing in configured **10 ms slots**: perfect within **±60 ms**, good within **±130 ms**, and an edge within **±240 ms**. These are program execution windows; network delay affects the player. They are not a promise of 10 ms end-to-end latency.

The browser checks the expected private-validator identity. It does not independently verify a hardware-attestation quote. The three native Rust programs are included below for review; the browser uses the Solana web3.js SDK.

## Play a friend and keep playing

Create a match and share its code, or select an open match from the lobby list. After the result, **both players choose “Rematch with this friend.”** The original host reopens the same table after its reset period, and the friend joins that exact next match.

The **shared leaderboard** stores recorded totals by player key in a separate result register. Reusing a match table does not erase those totals. Wins give 3 points; draws give 1. AI practice is separate. Completed forfeits count, and duplicate records do not count twice.

The leaderboard refreshes automatically every **five seconds while open** and catches up when the player returns to the tab. It shows Solana saving status and keeps the last displayed results during temporary read failures. The page tries to save a finished friend result before reusing its table.

## Programs and integration proof

| Program | Solana Devnet explorer |
| --- | --- |
| Cricket game | [`HwXp4A62RG3nBkmxTTaHbSi1wNj8vaVqZ6Kn5VkYBswT`](https://explorer.solana.com/address/HwXp4A62RG3nBkmxTTaHbSi1wNj8vaVqZ6Kn5VkYBswT?cluster=devnet) |
| Player registry | [`CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P`](https://explorer.solana.com/address/CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P?cluster=devnet) |
| Result register | [`BcS2Gb1Gihfzqs1UkViNnbQzjGcCGyvXu81VhwsFrUzU`](https://explorer.solana.com/address/BcS2Gb1Gihfzqs1UkViNnbQzjGcCGyvXu81VhwsFrUzU?cluster=devnet) |

## Source and running

- `index.html`: self-contained browser game; open it in a modern browser with internet access, or serve it from a static host.
- `programs/`: game, player registry and result-register source, with shared pinned build configuration.
- `media/pitch.mp4`: narrated, captioned pitch with the supplied desktop and mobile gameplay excerpts.
- `LICENSE` and `THIRD_PARTY_NOTICES.md`: project and bundled dependency licences.

No browser package installation or application login backend is needed. The game connects to Solana and MagicBlock services. To build the Rust programs, use the pinned toolchain and `sbpf-linker 0.2.5`, then run `cargo build-bpf` from `programs/`.

## Scope and roadmap

This is a **Devnet prototype** with eight tables, up to 140 registry names subject to shard capacity, and 140 ranked player keys. Only recorded results persist; overwritten earlier games cannot be reconstructed. Custom clients can bypass recording, and Devnet itself may reset. Leaderboard saves use a bounded sponsor reserve.

Next: better cricket motion from real shot references, teams and player skins, clearer cameras, useful replays, stadium atmosphere and smooth mobile play. **No wagers, stakes, entry fees, tokens or prize payouts are included in the game.**

Project code: [MIT](LICENSE). Dependencies: [third-party notices](THIRD_PARTY_NOTICES.md).
