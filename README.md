# Magic Over

**Six balls. Two wickets. Open the game and play—no wallet app, seed phrase or SOL required.**

Magic Over is a browser-based 3D cricket game on Solana Devnet and MagicBlock. Bat against a randomness-driven AI bowler immediately, or challenge a friend using a familiar name-and-password sign-in. The browser handles signing; players do not approve a wallet popup for every ball.

[Watch the demo and onboarding explanation](media/demo.mp4) · [Read the video transcript](media/demo-narration.txt) · [View the game program on Devnet](https://explorer.solana.com/address/HwXp4A62RG3nBkmxTTaHbSi1wNj8vaVqZ6Kn5VkYBswT?cluster=devnet)

![Magic Over demo](media/demo-poster.jpg)

## Start playing

Download this repository and open `index.html` in a modern browser with WebGL and an internet connection, or serve the repository with any static web host. The page includes its JavaScript dependencies; no package installation or application backend is needed to serve it. It still connects to Solana and MagicBlock to play.

For a local HTTP preview, run `python3 -m http.server 8000` from the repository folder and open `http://localhost:8000`.

- **Bat vs AI:** tap the button and start as a guest. No account creation.
- **Play a friend:** create a player name and strong password, or sign in with an existing one. Create a match and share its code; your friend joins with that code.
- **Bat:** time Defend, Drive, Cut, Pull, Loft or Leave as the ball reaches the bat. Keyboard controls are shown on the buttons.
- **Bowl:** choose pace, line, length and a hidden variation.

Each innings lasts six balls or two wickets. In a friend match, players swap batting and bowling, and the second batter chases the first score. Ties can trigger additional rounds, up to the program's four-round limit. Guest mode gives the batter a target to chase.

## Onboarding that feels familiar

The player should be able to enjoy cricket without first learning how to use a crypto wallet.

| What the player does | What Magic Over handles |
| --- | --- |
| Opens the game and chooses **Bat vs AI** | Creates a temporary guest signing key in the browser. |
| Chooses **Play a friend → New player** | Asks for a unique name and a strong password, with a 16-character minimum. |
| Creates their player ID | Generates a random key, encrypts a recovery copy locally and registers that encrypted copy on MagicBlock's public rollup. |
| Plays a shot or bowls a delivery | Signs and sends the game action automatically in the browser. No external wallet or repeated signing prompts. |
| Signs in on another device | Looks up the encrypted record using the name and unlocks it locally with the password. No seed phrase to copy across. |

Players do not buy SOL, fund an account or install an extension. The current Devnet flow charges the player **0 SOL for registration and play**. Infrastructure and sponsor costs still exist; player-free does not mean cost-free infrastructure.

### How the player keeps control

The signing key is generated randomly; it is not derived from the password. The password protects an encrypted recovery envelope using **Argon2id with 64 MiB of memory, three passes and parallelism one**. The envelope is bound to the normalized player name, owner public key and network genesis hash. The app publishes the encrypted envelope, not the password or plaintext key.

While signed in, the unlocked key is held in browser memory to sign actions. The app does not persist the private key in browser storage. The sound preference is stored locally. There is no separate Magic Over password server or custodial wallet provider. Access to MagicBlock's private rollup still uses its challenge-signing authentication service, which the app handles automatically.

A small program-checked proof of work adds friction to bulk name registration without showing a CAPTCHA. It does not establish that someone is human or prevent automated gameplay.

**Recovery has limits:** the encrypted record is public, so a strong, unique passphrase matters against offline guessing. There is no forgotten-password reset in this version. Recovery requires access to the registry; the browser can fall back to the last state committed to Solana. In-game registration requests that commit in the background without confirming base-chain durability, so a newly created record is not a guaranteed backup if the rollup disappears immediately.

## What MagicBlock does

| Feature | Use in Magic Over |
| --- | --- |
| **Public Ephemeral Rollup** | Hosts the delegated player registry, allowing name registration and signing proofs without asking players for SOL. |
| **Private Ephemeral Rollup** | Executes match creation, joining, bowling, shots and resolution against eight reusable match tables. |
| **Private account permissions** | Protect the bowler's hidden variation in a separate ball account. The arming transaction touches that private account; the public delivery exposes pace, line, length and release timing. The resolved result reveals the variation. |
| **Verifiable randomness** | Selects the AI bowler's deliveries and target. The game validates the expected VRF callback signer, table and sequence. Hidden variation also mixes a secret held in the private account. |
| **Zero-fee player actions on the rollup** | Lets the player's browser key sign moves while the player holds 0 SOL. |
| **Commitment and undelegation** | Program instructions support committing delegated state back to Solana Devnet and returning account control. The game browser does not automatically commit and undelegate a table after every match. |

The game uses the rollup's slot clock with a configured **10 ms per slot**. The program grades shot timing at execution: ±6 slots is perfect, ±13 good and ±24 an edge—equivalent to ±60, ±130 and ±240 ms under that configuration. These are timing windows, not a claim of 10 ms end-to-end latency or equal network conditions for every player.

The client checks the expected private-validator identity. It does not currently verify an Intel TDX hardware attestation quote itself. Privacy depends on MagicBlock's private-rollup service and its hardware trust model.

This build uses native Rust programs and direct browser RPC calls. The browser uses the official **Solana web3.js 1.98.4 SDK** for key generation, seed recovery and public-key handling. The recovery cryptography lives in a dedicated Solana identity module; the former Algorand SDK dependency has been removed. It does not use BOLT, MagicBlock session keys or Magic Actions. The browser signs with the player's key; those automatic signatures are distinct from a delegated session-key feature.

## Programs and source

| Program | Solana Devnet address |
| --- | --- |
| Cricket game | [`HwXp4A62RG3nBkmxTTaHbSi1wNj8vaVqZ6Kn5VkYBswT`](https://explorer.solana.com/address/HwXp4A62RG3nBkmxTTaHbSi1wNj8vaVqZ6Kn5VkYBswT?cluster=devnet) |
| Player registry | [`CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P`](https://explorer.solana.com/address/CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P?cluster=devnet) |

- [`index.html`](index.html): self-contained game page.
- [`web/`](web): editable UI, Solana identity, network, scene, character and sound modules, plus bundled dependencies. The identity source is [`chainkey-solana.js`](web/chainkey-solana.js).
- [`programs/magic-over/`](programs/magic-over): cricket program and its rule tests.
- [`programs/player-registry/`](programs/player-registry): encrypted player-registry program.
- [`docs/addresses.devnet.json`](docs/addresses.devnet.json): configured table and permission addresses.
- [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md): connection details and current limits.
- [`media/`](media): narrated explanation, subtitles, transcript and original phone gameplay clip.

### Rebuild the browser page

Run `python3 tools/build.py`, then `python3 tools/build.py --check`. This rebuild uses the checked-in sources and requires only Python 3. No Node.js, pip packages or external bundle paths are needed.

Run `python3 tools/verify_release.py` to check the build, preserved program hashes, SDK dependencies, branding and publication exclusions. Run `deno task test:identity` for the offline recovery and onboarding tests. The regular page rebuild still needs only Python; optional identity-source bundling is described in the [SDK migration notes](docs/SOLANA_SDK_MIGRATION.md). The Rust directories retain the supplied toolchain configuration for review; rebuilding or deploying those programs requires the separate Solana/Rust toolchain described in [deployment notes](docs/DEPLOYMENT.md).

## Current scope

This is a **Devnet prototype**, with eight concurrent match tables and a registry configured for 140 names. The AI is a weighted, randomness-driven bowler. This repository does not add wagering, tokens or prize payouts. Availability depends on the configured RPC and MagicBlock services; the programs are upgradeable. A program address alone does not prove that the deployed binary matches these sources.

This update replaces the legacy SDK key/account helpers with Solana SDK routines while preserving existing player recovery records. Compatibility tests cover the old envelope format and the current onboarding flow. See [the SDK migration](docs/SOLANA_SDK_MIGRATION.md) and [test results](docs/sdk-verification.json). Existing account seeds, program addresses, crate names, network identifiers and cryptographic domains retain their original names for compatibility. See [release notes](docs/RELEASE_NOTES.md) for exactly what was checked.

## License

Project code retains the repository's [MIT license](LICENSE). Bundled dependencies retain their own licenses; see [third-party notices](THIRD_PARTY_NOTICES.md).
