# Public MagicBlock Devnet multiplayer

Tested the exact HTML of gameplay commit `1c5e07573b697d4c3948218ba080aa3965645f47`, SHA-256 `e4f81a1263d70aa469291e3c01df3dccca35bd1fd18d1e6c1930b113b1128db5`. The game ran unchanged in two separate Chromium contexts with fresh local storage. Each context used the game's Smaran signup form on `https://devnet-as.magicblock.app`, then joined world `GjYsCcq2r5KkVab7suzvStwYkAcJ3nTe9yXN8EcC5T9E`.

| Smaran test name | Public address |
| --- | --- |
| ebproof-cac6be757b-a | `FEEWonZryrE5GpTuYigaCPJ9Us5npfX9r5zF9iYVL4QM` |
| ebproof-cac6be757b-b | `6gmZ14KEib51QpUHC1oQRsJMjtfTN61dW5K2TXjibZHB` |

The network finalized all 49 selected gameplay and registration writes. The receipt set includes 2 registrations, 8 joins, 28 movement batches, 4 room transitions and 7 leaves. Each reported a 0-lamport fee and 0-lamport signer balances before and after execution. Both Solana Devnet balance reads also returned zero. A separate read-only client collected [transaction signatures, slots and fee receipts](public-receipts.json) using `getTransaction` and `getSignatureStatuses` on the public MagicBlock endpoint.

The clients observed each other's tile updates in both 2D and first person. Diagonal inputs produced cardinal-only batches. Both entered the shop at (67, 100), became visible to each other indoors, and returned outside. The observer recovered a peer update with its original page open after an intentional network cut. Its avatar element changed position within 1,286 ms of restoring traffic. A separate browser reload recovered the same account and tile. The peers left, disappeared from each other's roster, rejoined with their original addresses, then left again. Early room test timeouts were harness errors involving a decorative house without a room layout and an incorrect predicate.

| Measurement | Result |
| --- | --- |
| HTTP submit to RPC response, 49 selected writes | 514 ms median, 755 ms p95 |
| Submit to peer state, two 2D steps | 1,038 and 1,192 ms |
| Submit to peer state, two first-person steps | 6,892 and 7,853 ms |
| Two-context software Chromium frame intervals, 24 samples per mode | 100/133 ms median/p95 in 2D, 1,383/1,517 ms in first person |
| Browser world account RPC responses, 584 samples | 507 ms median, 996 ms p95 |

The Cloud proxy CA was available to Node but absent from the Chromium trust store. The browser harness forwarded the exact RPC requests with Playwright HTTP transport, which validated upstream TLS with the Cloud CA. The game's Devnet guards, registry, signing, polling and state handling were used. Gameplay exercised HTTP polling. Two separate read-only public WebSocket subscriptions acknowledged in 1,884 and 1,876 ms. Browser WebSocket delivery, physical devices and hardware-accelerated first-person timing remain unmeasured. The first-person observations include headless rendering and observer delay, in addition to public RPC transport.

Nine automatic save attempts received existing program responses for an early save or no pending changes. Thirteen browser world reads were deliberately cut. One additional read did not return during the run. The [browser event log](public-events.jsonl) records these separately from the successful gameplay writes and the initial native TLS errors. [Screenshot after reconnection](two-public-players.webp).

The unchanged live-proof HTML passed all 18 release checks (one landscape horse-control timeout passed on retry), four Clean View profiles and the existing two-client local fixture. The compiled program passed 18 checks. Its optional delegation CPI check requires the official delegation ELF, which was unavailable.
