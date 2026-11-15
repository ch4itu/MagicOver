# EternalBliss agent rules

These rules apply only to `eternalbliss/`. R15.56 live baseline: commit `77b0b430995c9b37b9369bab1399ae5ec706ef80`. R15.55 rollback: `6321565231a81204e4c92c801ef3b9eca3175648`.

## Product and architecture

- EternalBliss is an existing **single-file HTML** RPG at `eternalbliss/index.html`, not a new app. Retain the original maps, named places, saved identities, quests, existing MagicBlock/Smaran protocols, and main menu.
- Preserve third-person/3D, solo Farlands first-person, 2D, movement pacing, Clean View, mobile D-pad, touch controls, day/night, seasons, boats, horses, interactions, battles, and local/chain persistence. Do not silently introduce server, login service, TURN relay, payment, or new dependencies.
- Solo Farlands additions must not alter the on-chain/shared-world rules. Existing shared worlds must remain isolated from solo NPC and terrain simulation.
- Do not fix visual scale by stretching models independently of collision footprints, door positions, furniture, entrances or interactions. Do not add clones of settlements, buildings, or residents to create fake density.
- No fake NPC movement: rendered, interactable and collision positions must refer to the same authoritative local NPC state.
- In future phases prefer purposeful authored landmarks and careful visuals to procedural repetition. Do not redesign the entire world as part of one village task.

## Workflow: prevent repeated costs

- **Stay on a feature branch**. Do not deploy, merge, push to main, run migrations, or write live on-chain data without explicit approval.
- Read `eternalbliss/R15.56-NEXT.md` at the start of each task. Inspect the code and review previous evidence before running tests.
- **Do not restart an already-passed regression suite when resuming a session, after context compaction, or after a credit limit.** If the exact candidate code and test inputs have not changed, reuse the last recorded result. If either changed, rerun only relevant suites; run the full matrix once on the final release candidate.
- Keep a compact append-only test ledger: candidate commit or SHA-256, script/test version, browser/profile, test result, screenshots and time. Distinguish source review, syntax, simulated browser, actual browser, real phone and live multiplayer evidence. No fabricated passes.
- Run one browser suite at a time. For every code change, first run focused tests for that change and one short 3D/FP smoke check. Screenshots at consistent cameras precede visual acceptance.
- Never disguise an untested hotfix as stable. Preserve rollback SHA and exact diff, and stop if a failure could affect saved state or shared-world protocol.
- Finish a task with concise changes, precise tests newly run (not historical totals), skipped checks and known risks. The user's rule is: **No regressions. No hallucinations. No idea worms.**

## Revenue and monetization policy

- The player experience is free. Revenue, if introduced, comes **only from non-intrusive advertisements** that do not interrupt matches or exploration; see root `README.md` and `FOUNDERS.md`.
- Do not introduce token or coin sales, entry fees, stakes, wagering, prizes, pay-to-win purchases, login charges, subscriptions, or additional ad SDKs without explicit authorization. No monetization feature is requested by this documentation change.
- The costs in `FOUNDERS.md` are a Devnet measurement dated 4 November 2026 and must be rechecked before mainnet commitments; never silently treat them as guaranteed prices.

## Scope and starting point

First address the unverified R15.56 interaction between Mira and nearby residents at the well. Then improve visual quality **one asset or authored location at a time**, on isolated branches. The living-world work must preserve the progress already made at Wayfarer's Rest.
