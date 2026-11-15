# Overnight development checkpoint — 15/16 November 2026

User authorization: continue through multiple milestones autonomously, including architecture, vegetation/terrain/atmosphere and meaningful settlement behavior. This supersedes routine waits for the next visual prototype; visual review is still required before merge/deployment. Free play / non-intrusive-advertising-only policy remains; implement no monetization. No chain writes, merge or deployment authorized.

Starting stack: PRs #5–#8, latest commit `566953a`. Live baseline `77b0b430995c9b37b9369bab1399ae5ec706ef80`, rollback `6321565231a81204e4c92c801ef3b9eca3175648`. Preserve both. Use the existing checkout and feature branches.

Session began about 23:07 India time on 15 November. Requested end: 08:00 India time on 16 November (02:30 UTC). No supported scheduler/automation tool for resuming this Codex task is exposed. No background autonomous continuation is claimed. Work within the active session's available execution/usage limits; use this checkpoint and the append-only test ledger when resuming, without restarting completed checks.

No Alerith reference assets/link were found in the workspace. Do not claim reference fidelity. Improve the existing single-file art with original authored geometry, coherent materials and readable human-scale spaces.

## Working queue

1. Portrait first-person combat weapon visibility and impact feedback. New blade-projection check reproduced the old weapon outside the phone frustum. Candidate validation in progress; no pass yet.
2. Rework the existing Wayfarer's Rest inn as a crafted architectural asset, with unchanged footprint/entrance, shared geometry and bounded render cost. One building, no cloned settlements.
3. Improve Farlands surface transitions and vegetation/atmosphere without changing authoritative terrain or collision.
4. Add a meaningful activity/interaction to the existing settlement residents, preserving identities and using actual current NPC positions. Keep story rewards idempotent and local.
5. Continue to dungeon/exploration and remaining regressions as execution permits. No speculative release claims; stop expanding a milestone once its focused checks pass.

## Evidence discipline

Run one browser suite at a time. New logs/captures: `/workspace/.magic-over-setup/overnight-evidence`. Existing completed evidence stays under `tests/evidence.jsonl` and previous workspace directories. Append a milestone result, commit, PR and relevant source/test hashes after each milestone. Failed candidate/harness logs remain separate. Fixed clocks must be installed before app startup, in fresh profiles. Reuse the existing inn prototype baseline pictures where the inn rendering has not changed; do not recapture an unchanged baseline merely because the session resumed.

The historical matrices, natural full quest route, physical Android performance and live two-client shared-world behavior remain release gates unless new evidence explicitly closes them. A desktop/mobile-emulated render is not a physical-phone performance test. A staged fixture is not a natural quest playthrough.

## Milestone 1 — portrait combat feedback completed

Branch `feat/eternalbliss-mobile-combat-feedback`, HTML `9636243366c4a6d0a3a8cae0b8fd7a067c6d5545cc274f82e31a4e5ce426f28a`. Fifteen new real-render projection checks (five poses × portrait/landscape/desktop) and one new-candidate 3D/FP/2D smoke passed. Camera-local weapon scale/inset fits portrait; small enemy-impact hand recoil respects reduced motion. World FOV, player/world model scale, battle rules and input handlers are unchanged. Old suites reused, not rerun. `weapon-baseline.log` reproduces old phone blade x=2.13 NDC; first candidate still exceeded bounds; v2 passed. All logs retained in overnight-evidence. Proceed to architecture; do not rerun this unchanged candidate.

## Milestone 2 — crafted architecture completed

M1 is draft PR #9, commit `1c460f3`. M2 branch `feat/eternalbliss-crafted-inn`, final HTML `cfcd0d8bcb1ac19742c1c6f97de4942879e2470b6c36b6d86ca9f296adec72e1`: rebuilt only the existing inn in footprint units, with pitched roof/tile courses, timber frame, shutters, stone footing and supported entrance. Static geometry is baked into 28 shared material batches / 4,338 vertices, cached across revisits with independent instance transforms. Node geometry/cache budget checks passed; not an FPS claim. Summer noon/night FP/overhead captures passed finite-sky checks and were reviewed; early roof seams showed sky, corrected by an under-roof surface. Entrance/exit and all six blocked tiles passed on the first architecture candidate; those unchanged contracts were reused for the roof-only correction. Final new-candidate startup/3D/FP/2D smoke passed. Comparisons: `tests/review-images/crafted-inn-*.jpg`; earlier prototype baseline captures reused, not repeated. Proceed immediately to landscape rendering.

## Milestone 3 — surface and orchard rendering completed

M2 is draft PR #10, commit `f286a1f`. M3 branch `feat/eternalbliss-soft-landscape`, HTML `b27626886039f3b4fa803c70e495579ebc4beca013e2a6dd2ab301a5e9b16656`. Farlands soft ground shares cached corner tints; path-edge centre fans retain readable paths. Existing orchard tree locations/trunks now have connected asymmetric low-poly crowns. No new trees or collision/terrain edits. A new baseline check found colour discontinuity at 1,534 mixed corners (maximum channel difference 0.285). Full-FP candidate checks found zero difference at 1,534 desktop / 1,142 phone corners, with 107,226 / 71,574 ground vertices. Authoritative state was unchanged by rendering. New canopy instances cleared/restored in actual combat; new-candidate startup/3D/FP/2D smoke passed. The first candidate check read the pre-FP-radius mesh; it remains evidence for that smaller mesh. Explicit terrain-signature readiness was added for the full-radius desktop/phone cases. No FPS or real-phone claim. Proceed to a resident activity without rerunning completed rendering checks.

## Milestone 4 — Tamsin's work activity completed

M3 is draft PR #11, commit `a96e1d2`. M4 branch `feat/eternalbliss-resident-herb-work`, HTML `1ba5c67b457f50a0481f6ba812f4a23b998aa6b95798e13f15d8147ae8b4b794`: authored three-parcel herb lesson offered by existing Tamsin at her real daytime scheduled work spot. Wrong answers are feedback only; first completion grants one existing health potion and records `work-rest-herbs` in the existing journal. No save version/schema change or repeat farming. Idle sorting pose uses the actual NPC location and respects reduced motion. Desktop/phone actual Space/choices, 44px targets, reward once, unchanged gold/XP and reload idempotency passed. Evening, range, stubbed shared-active, after-hours and outside-Farlands guards passed. One affected Mira pending-relic case and one new-candidate startup/3D/FP/2D smoke passed; old matrices reused. A previously existing Mira screenshot was backed up before the new regression capture, then restored; the new capture is retained in overnight-evidence. Proceed to scoped sky atmosphere.
