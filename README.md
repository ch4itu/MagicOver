# Magic Over :: Cricket on MagicBlock ER · PER · VRF

**Six balls. Two wickets. Open the game and play. No wallet app, seed phrase or SOL required.**

Magic Over is a browser cricket game on **Solana Devnet**. Try **Bat vs AI** without an account, or play a friend with a familiar username and passphrase. The browser signs the moves for you. MagicBlock handles the public player registry, private matches and verifiable randomness for the AI.

[Watch the silent, captioned demo](media/demo.mp4) · [Pitch and demo transcript](media/transcript.md)

## Easy onboarding is the starting point

We want someone who has never used a wallet to enter Web3 just by playing cricket.

| Player experience | What happens underneath |
| --- | --- |
| **Bat vs AI:** open and play | The browser creates a temporary guest signing key. No registration. |
| **Play a friend:** choose a username and strong passphrase | A random key is generated in the browser. It is not derived from the password. |
| Create the player ID | A recovery copy is encrypted locally; its passphrase lock uses **Argon2id, 64 MiB, three passes**. The encrypted record is registered on MagicBlock's public ER. The browser requests a registry commit back to **Solana Devnet**. The password and plaintext key are not published. |
| Bowl or play a shot | The browser signs and sends the action. No extension or repeated wallet prompts. |
| Move to another device | The username locates the encrypted record; the passphrase unlocks the same key locally. |

**Players need 0 SOL for this Devnet registration and play flow.** Sponsors still cover setup, rent and applicable infrastructure/service costs. No external wallet installation, funding step or seed-phrase handling is required.

The registry accounts start on Solana Devnet and are delegated to the public ER. New registrations are written on the ER first; a registry commit saves that state back to Solana Devnet. The current onboarding requests this commit in the background without verifying completion, so a just-created record must not be described as already confirmed on Solana.

Usernames are normalized: `Alice` and `alice` identify the same name. Duplicate full name hashes are rejected. Different names landing in the same storage slot are handled by linear probing and a full 256-bit hash comparison, without overwriting an existing player. Complete hash/key collisions are extraordinarily unlikely, not mathematically impossible. Four shards hold **up to 140 names**; a full assigned shard can refuse another name even if another shard has room.

Argon2id derives the passphrase wrapping key; XChaCha20-Poly1305 provides authenticated encryption of the recovery material. The signing key itself is generated independently of the passphrase.

The unlocked key stays in memory while playing; the app does not save the private key in browser storage. Recovery depends on the registry or its last Solana commit. There is no forgotten-passphrase reset. Use a strong, unique passphrase: the encrypted record is public. The recovery envelope is bound to the normalized name, owner public key and network.

There is no separate Magic Over password server or custodial wallet provider. MagicBlock's private RPC still uses challenge-signing authentication, handled by the app. A small proof of work adds a cost to bulk registration without a CAPTCHA prompt; it does not prove a player is human or stop all bots.

## What MagicBlock does

| Feature | Concrete use |
| --- | --- |
| **ER — Ephemeral Rollups** | The delegated player registry runs on the public rollup, enabling the name-and-passphrase onboarding flow without player funding. |
| **PER — Private Ephemeral Rollup** | Eight reusable match tables execute create, join, bowl, shot and resolution actions on the private rollup. |
| **Private account permissions** | A separate private ball account holds the bowler's hidden variation. Its arming transaction touches only that account. Pace, line, length and release timing are public; the resolved result reveals the variation. |
| **VRF — verifiable randomness** | Selects AI targets and deliveries. The game validates the expected callback signer, table and sequence. Hidden variation also mixes a private secret, so the VRF value alone does not reveal it. |
| **0 SOL player actions** | Players sign rollup moves using unfunded keys. This removes the wallet-funding step. |
| **Saving back to Solana** | Each newly recorded friend result requests a commit of leaderboard totals to Solana Devnet. Cricket tables remain delegated for reuse. Their program supports committing and undelegating them; the browser does not automatically undelegate tables after each match. Registration requests a registry commit in the background without verifying completion. |

Shot timing uses the program's configured **10 ms slots**: perfect within ±60 ms, good within ±130 ms, and an edge within ±240 ms. Timing is judged when the action executes. These are scoring windows, not a promise of 10 ms end-to-end latency; network delay still matters.

The client pins the expected private-validator identity. It does not independently verify an Intel TDX attestation quote. Privacy relies on MagicBlock's private-rollup service and hardware trust model.

The browser uses the official **Solana web3.js 1.98.4 SDK** for key generation, seed recovery and public keys. Recovery uses a dedicated crypto module. The runtime does not use the Algorand SDK. We do not use BOLT, Magic Actions or MagicBlock session keys; automatic browser signatures are not the session-key feature.

## Play, finish and play again

Open `index.html` in a modern WebGL browser with internet access, or serve this folder on a static host. No package installation or application login backend is required. It connects to the configured Solana and MagicBlock services.

- **AI practice:** choose Bat vs AI and chase the target.
- **Friend match:** create/sign in to a player ID, create a match and share its code. Your friend joins the code.
- **Batting:** time Defend, Drive, Cut, Pull, Loft or Leave as the ball reaches the bat.
- **Bowling:** choose pace, line, length and a hidden variation.
- **Rematch:** after the result, **both players choose “Rematch with this friend.”** The original host reopens the same table after its existing reset period, and the friend joins that exact new match. Players are no longer sent into separate matches. If either player leaves or someone else takes the table, return to the lobby and share a new code.

Each side has six balls and two wickets. Players swap roles for the chase. Ties can trigger further rounds, up to the program's four-round limit. The table reset lasts up to 15 seconds after the result; the rematch screen shows the wait.

### Shared persistent leaderboard

Open **Leaderboard** from the home screen or friend lobby. A separate result-register program stores recorded friend-match totals by public key. **Reusing a match table or restarting the browser does not erase those totals.** A win gives 3 points, a draw gives 1, and a loss gives 0. Ties use wins, then a deterministic player-key order. Names are verified against the player registry for display; identical display names do not merge different keys. Completed forfeits count. AI practice is separate.

The updated client records completed results and records a finished table's result before reusing it. The register checks the actual game program, table address, match ID, finished phase and players. Duplicate submissions do not add another win or trigger another save. Each new record requests a commit of the totals back to **Solana Devnet**. The page distinguishes a recorded result from confirmed Solana storage and can fall back to the last committed totals if the rollup is unavailable.

This prototype supports 140 player keys. It starts with recorded results from this release; overwritten historical games cannot be reconstructed. Custom clients that bypass the result recorder can leave gaps, and an on-chain result is not proof of a unique human or protection against collusive forfeits. Devnet can undergo resets, so this is persistence across games on a testing network, not a promise of permanent production storage.

A bounded sponsor reserve pays applicable leaderboard save costs. Players still need 0 SOL. If the reserve or capacity is exhausted, saving reports an error and the updated client avoids overwriting the unsaved finished result. Additional sponsor top-ups require separate authorization.

## Programs

| Program | Solana Devnet explorer |
| --- | --- |
| Cricket game | [`HwXp4A62RG3nBkmxTTaHbSi1wNj8vaVqZ6Kn5VkYBswT`](https://explorer.solana.com/address/HwXp4A62RG3nBkmxTTaHbSi1wNj8vaVqZ6Kn5VkYBswT?cluster=devnet) |
| Player registry | [`CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P`](https://explorer.solana.com/address/CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P?cluster=devnet) |
| Result register | [`BcS2Gb1Gihfzqs1UkViNnbQzjGcCGyvXu81VhwsFrUzU`](https://explorer.solana.com/address/BcS2Gb1Gihfzqs1UkViNnbQzjGcCGyvXu81VhwsFrUzU?cluster=devnet) |

Base RPC: `https://api.devnet.solana.com` · registry ER: `https://devnet-as.magicblock.app` · match PER: `https://devnet-tee.magicblock.app`.

Expected validators: public `MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57`; private `MTEWGuqxUpYZGFJQcp8tLN7x5v9BSeoFHYWQQ3n3xzo`.

The three native Rust programs are included for review. The existing game and registry source files, seeds and recovery domains are unchanged. The new result-register deployment was verified byte-for-byte against its build. A program address alone does not prove source/binary equivalence. This prototype has eight concurrent tables and up to 140 registry names, subject to the per-shard limit; custom clients can bypass the UI's registration flow. It is not a completed anti-cheat or public-launch security audit.

## A small repository

- `index.html` — the complete game, including browser code and bundled dependencies.
- `programs/` — the game, player registry and result register with a shared build configuration.
- `media/` — the silent captioned demo and the short-pitch/timecoded-demo transcript.
- `LICENSE` and `THIRD_PARTY_NOTICES.md` — project and dependency licenses.

The self-contained page is the runnable browser artifact; readable modules and verification tools are retained in the separate maintenance archive supplied with this update. The public repository does not need duplicate browser sources, generated reports, old clips, keys or compiled program binaries.

For the Rust programs, install the pinned toolchain and `sbpf-linker 0.2.5`. From `programs/`, use `cargo build-bpf`; host rule tests use `cargo test --target x86_64-unknown-linux-gnu -Z build-std=std,panic_unwind --release`. Building is separate from deployment; the existing game and registry programs were not upgraded for this release.

## Roadmap

- Improve cricket motion using real shot references: footwork, balance, bat path, contact and follow-through.
- Add teams, custom kit colours and cosmetic player skins.
- Improve delivery readability, camera framing, useful replays and stadium atmosphere.
- Keep controls accessible and mobile performance smooth as graphics improve.

These are planned enhancements. **There are no wagers, stakes, entry fees, tokens or prize payouts in this build, and wagering is not on our roadmap.**

## Test-network costs

Players need no SOL for the current registration and play flow. Operators fund program deployment, account storage and applicable save fees. Devnet uses test tokens, separate from real Mainnet SOL, and may reset. See [Solana’s cluster documentation](https://solana.com/docs/references/clusters). A sponsor deployment budget is an authorization ceiling, not a player fee or a fixed MagicBlock price.

## License

Project code: [MIT](LICENSE). Bundled dependency notices: [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
