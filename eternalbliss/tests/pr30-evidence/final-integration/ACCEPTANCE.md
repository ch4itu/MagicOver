# Final candidate verification

All 17 matrix jobs passed on gameplay commit `2ec2d67fa9cb6b5b92999f35470934e79c77a06f` and HTML SHA-256 `efd0588467394bbb5bd96271b7913ff0f5d7a02db7caa178a52c5f1f44b4f0bd`. The matrix records the fingerprint, elapsed dates and log for each job. Earlier PR 30 runs remain in the historical evidence folders.

| Area | Verified on the candidate |
| --- | --- |
| Coastal geography | Dry settlement cores and roads, walkable approaches, connected channels and rendered hull clearance at Amberport, Lantern Quay and Shellwick. Pearl Banks reaches the shared landing. |
| Save migration and rollback | The original v1 bytes were archived. The v2 load and archived-v1 rollback preserved the hero and room position, horse, parked and taken boats, terrain edits, pin, quest flags and gold. An incompatible future-version record remained intact after movement and a save attempt. |
| Residents and quests | On-shift poses, work talk and Pearl Banks Atlas lead. Existing quest contracts, fresh guest Millwright’s Tally and Harvest Bell routes, and Shellwick’s actual navigation, report and once-only reward. |
| Transport | Generated Mill Creek mooring checked in three profiles. The trapped pocket remains empty. Horse buy, mount, move, dismount and reload passed. Camera ray hits, actual boat Use, embark, sailing, reload and docking passed. Authored coast validation also exercised Lantern Quay in three profiles. |
| Mobile and 3D | Native Chromium touch input on 390 × 844 portrait and 844 × 390 landscape. Ground taps in 2D and overhead 3D, swipe rejection, circular D-pad, Atlas, first-person Look and Use, and combat passed. Desktop startup covered overhead 3D, first person and 2D. |
| Shared world | Unchanged protocol scripts and stubbed active-world local-save protection. The run never required a wallet, real chain transaction or concurrent players. |

The draft is ready for review of the integrated Chromium results. Release acceptance remains open because SwiftShader still shows multi-second frame intervals and physical-device performance and live multiplayer are unverified.

- [Matrix fingerprints and status](matrix/pr30-matrix.json) and [individual logs](matrix/)
- [Fresh guest route, work interactions and screenshots](GAMEPLAY.md)
- [Rendering measurements and matched screenshots](PERFORMANCE.md)
- [Native portrait combat](gameplay/combat-phone-false-guard.png), [native landscape combat](gameplay/combat-landscape-false-guard.png), [portrait Lantern Quay](gameplay/lantern-first-person-portrait-efd05884.png), [landscape Lantern Quay](gameplay/lantern-first-person-landscape-efd05884.png)
- [Cumulative browser and rendering ledger](../ledger.jsonl)
- [Replay and rollback instructions](REPLAY.md)
