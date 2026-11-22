# Multiplayer reality check

The checked-in sBPF binary was exercised in LiteSVM with two signed test identities on one simulated ledger. The two-browser test substituted the session bootstrap and transport, and let the shipped client decode the account, predict/reconcile movement, update the remote roster and process room and leave transitions. The browser contexts had separate storage. The reload reused the simulated identity in its original context. This does not exercise public sign-in, client transaction signing, WebSocket failover or public seat recovery.

## Shared and local

| Feature | Where state lives | Evidence |
| --- | --- | --- |
| Seats, positions, room membership and per-character inventory/progress | Shared world and character book | Compiled-program tests and the local two-browser fixture for seats, position and rooms |
| Encounter HP, respawn, loot claim and cooldown, and shop stock | Shared account | Compiled-program tests with two signed players |
| NPC and enemy patrol presentation | Client derives positions from the shared clock and fixed routes | Client-source audit |
| Camera, avatar facing without a move, battle menus and animations | Client presentation | Client-source audit |
| Farlands story quests, relationships, local errands, mounted travel and personal saves | Solo Farlands | Separate solo systems and shared-active save guard |
| First-person, diagonal steps and horses in shared mode | Shared mode exposes overhead, cardinal moves and no horses | Two-browser fixture and client-source audit |

## Network boundary

Read-only Devnet RPC and two WebSocket account subscriptions were probed separately. No real player registration, join, movement or leave transactions were submitted to the public network. The public preview request from the workspace returned a proxy error after Sites confirmed publication of version 11. Public movement latency and real-device rendering are unmeasured.

The local fixture's delivery time runs from the browser's request at the bridge to peer account ingestion. It excludes public transport, signing latency and the remote interpolation delay. The update rate is the scripted exercise rate, with wait time for browser assertions. It is not a network throughput limit. Frame timing is from headless software-rendered Chromium with both contexts open. Payload sizes distinguish binary account, Base64 field and full JSON HTTP response.

## Reproduction

Install solders 0.29.0 into a development Python environment, then run the following from the repository root with the repository's Playwright dependency available:

```sh
node eternalbliss/tests/release/two-clients.cjs
```

The fixture never sends public writes. The final run also held one client offline during a peer move, then verified catch-up after restoring the fixture's account feed. The exact-hash matrix passed all 18 cases and all four Clean View profiles. The compiled-program suite passed 18 checks. The optional official MagicBlock delegation CPI check was skipped because its binary was unavailable. The failed-leave branch was also reproduced by isolating the original and corrected production leave functions with a rejected transport. The original dropped the shared session and cleared its status. The corrected branch retained the session and reported that departure was unconfirmed.
