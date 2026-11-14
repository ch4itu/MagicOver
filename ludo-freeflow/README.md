# Ludo Freeflow

Four colours on one board, and no turns: every player rolls and moves whenever their own die is played.

- **Local preview** (no sign-in): you are red, three bots play the other colours, dice come from the browser. Fully offline. The bots roll only as often as you do; a bonus roll after a six or a capture is one they do not match.
- **Online match** (sign in with Smaran): two to four people, one colour each. Every die comes from MagicBlock VRF and is applied on chain before the page shows it. Nobody needs SOL, a wallet or a seed phrase.

## How an online match works

| Part | What it is |
|---|---|
| Program | `2i2REZWzDeAqKuJBEypH9fBtVZB6jmZbBRX7YH4fiETp` on Solana Devnet (source: `programs/ludo`) |
| Rollup | MagicBlock devnet rollup `devnet-as.magicblock.app` (validator `MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57`) |
| Tables | 8 accounts, PDA `["lf-table", i]`, delegated to the rollup |
| Dice | MagicBlock VRF (`Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz`, rollup queue `5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc`) |

- **Seats belong to Smaran keys.** Create and Join are approved once in Smaran's sign-in window. They name a *session key* that this page makes for the match, and that key signs every roll and move. The Smaran key never enters this page. "Continue here" on another device (one approval) gives the seat a new session key, and the old one stops working at once. A reloaded tab keeps playing with its session key.
- **Rolls.** A roll asks MagicBlock VRF for randomness tied to the table, the match, the seat and the seat's action number. The die exists only once the VRF program's signed callback for exactly that request has been applied. A seat waiting for its die never blocks another seat. If no answer comes within 8 seconds, the seat may roll again: the new request replaces the old one, whose late answer is then refused.
- **Fair dice.** The die is the first randomness byte below 252, mod 6, plus 1. Each face gets exactly 42 of those 252 byte values.
- **No double actions.** Every roll and move carries the seat's next action number, so a retry, a replay or the same player on two devices can never apply an action twice. A move also names the token's current square: if a capture sent that token home first, the move is refused instead of becoming a different move.
- **Same rules as the preview.** A six enters a token; a token walks 51 squares from its start square round to its column entrance, then its 5 column squares, then home, with the exact count. Landing alone on an opponent's single token off the stars and start squares sends it home. Two tokens of one colour block other colours, except on the stars and start squares, which every colour shares. A third six in a row is forfeited. The program's rules were compared with this page's own JavaScript over 400 random four-colour games (281,564 actions). They agree on every state.

### Choices made for online play (please confirm)

1. A match needs two to four people. A colour with nobody in it stays empty (there are no online bots). Anyone seated can start once two have joined, and four joined start the match automatically.
2. Leaving mid-match takes that colour off the board (except tokens already home). If only one colour is left, it wins.
3. "Pass" is allowed only when the waiting die has no legal move left, for example when someone has just formed a block.
4. The bonus after a six or a capture is shown, as in the preview. With no turns, everyone may always roll again once their die is played.
5. A finished match stays on its table for 20 seconds. A lobby or match with no action for 15 minutes can be replaced by a new one.

## Checks

- `programs/ludo/src/tests.rs`: rule tests and random games (host).
- `programs/ludo/check/ludo-diff.js`: the program's rules against this page's JavaScript, game by game.
- `programs/ludo/check/check_ludo.py`: instruction-level checks on a local ledger (LiteSVM).
- `programs/ludo/check/ludo-er-test.js`, `ludo-vrf-batch.js`, `ludo-vrf-audit.js`: real matches on the rollup with VRF dice.
- `ludo-freeflow/tests/ludo-browser.js`: two browsers (desktop and phone) through Smaran's real window. Credentials come from the environment.
