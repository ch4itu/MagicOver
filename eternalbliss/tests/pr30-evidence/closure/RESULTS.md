# PR 30 release-closure candidate

Gameplay commit `a9f7349899a0edd542efd58c9cd52934ab761f82`. Final HTML SHA-256 `abc17827a03e60d5594581b0c5b7273bf38aea87d36f3f2aa6f1344145e0d350`.

## Advancing-clock play

A fresh guest played **30 min 34 s**, then reloaded and verified the saved state by **30 min 49 s**. The clock ran naturally through four game-day boundaries, and the guest made **3,387 moves**. The route covered the Harvest Bell and Archive Warden, riding a purchased horse, Nessa’s orchard tally, and Hale’s new Mill Creek sounding. The guest read the causeway board, boarded a real boat through first-person Use, sailed to the reach, reloaded afloat, landed and collected the reward once. The final save still contained Hale’s completion flag. Of the 35 repeat travel legs, 21 ended with an actual conversation with Ivo, Tamsin or Wren as their schedules allowed. The route planner read positions and aimed the test camera at the boat hull. It used player input for movement, interaction, combat and rewards. No gameplay state or clock was injected.

- [Chronological journey evidence](journey/natural-harvest-events.jsonl) and [assertion ledger](journey/ledger.jsonl)
- [First-person gate after the fix](journey/natural-archive-gate-abc17827.png), [Archive Warden](journey/natural-warden-defeated-abc17827.png), [first ride](journey/natural-first-ride-abc17827.png)
- [Orchard tally](journey/natural-orchard-tally-1-abc17827.png), [sounding afloat](journey/natural-sounding-afloat-abc17827.png), [Hale reward and level-up](journey/natural-hale-reward-abc17827.png), [final reload](journey/natural-complete-reloaded-abc17827.png)

The first attempt exposed a 900 × 540 layout defect where the expanded minimap blocked Use in the archive. The corrected build has separate space for the controls. The 30-minute result is from a fresh session on the corrected build. [Before the fix](journey/minimap-blocked-before.png)

## Live acceptance boundaries

The public devnet HTTP APIs responded to read-only version requests. Two simultaneous read-only WebSocket clients could not connect from this workspace. A live join sends a signed shared-world transaction. Real concurrent play still needs a controlled environment and authorized test identities. [Read-only WebSocket results](magicblock-readonly.json)

Phone coverage uses native Chromium touch events in emulated portrait and landscape viewports. Hardware GPU frame pacing and physical-device acceptance remain open.

## Integrated regression

All **18 matrix jobs passed on the exact candidate HTML**. This includes quest compatibility, Mill Creek progress gates, old save migration and archived rollback, the fresh fixed-noon route, generated and authored moorings, both mobile layouts and first-person combat, startup and the shared-world source and save guards.

- [Exact-hash matrix and logs](matrix/pr30-matrix.json)
- [Short desktop controls](journey/smoke-short-first-person-abc17827.png), [portrait Atlas](journey/touch-atlas-portrait-abc17827.png), [landscape Atlas](journey/touch-atlas-landscape-abc17827.png)
- [Replay and rollback](REPLAY.md)

## Rendering

Chromium SwiftShader, Wayfarer’s Rest first person at 844 × 390 and fixed noon. Each run warmed up for 20 frames and sampled 120 draw callbacks.

| Build | Callbacks with a submission | Submission interval p95 / p99 | Draw callback p95 | CPU submission p95 |
| --- | ---: | ---: | ---: | ---: |
| Starting build | 120 | 1,390 / 2,051 ms | 1,389 ms | 10 ms |
| Fence trial on starting build | 94 | 724 / 747 ms | 691 ms | 7 ms |
| Final candidate | 97 | 711 / 821 ms | 694 ms | 6 ms |

When the previous SwiftShader submission is still pending, the candidate defers the next first-person submission while keeping camera, simulation and input active. The measured submission tail improved without changing graphics settings. The candidate skipped 23 callbacks and still had 33 draw intervals over 100 ms, the same count as the reference. These observations support software submission backpressure as a contributor. The timers do not measure compositor presentation or establish hardware GPU throughput. The hardware-only Chromium attempt at `--disable-software-rasterizer --use-angle=gl` could not create a WebGL2 context. The workspace exposed no GPU device nodes.

- [Reference](render/reference.json), [fence trial](render/fence-trial.json), [final data](render/candidate.json), [hardware probe](render/hardware.json)
- [Matched image before](render/rest-before.png), [after](render/rest-after.png)

Release acceptance remains open for the remaining frame stalls, physical hardware, and live concurrent play.
