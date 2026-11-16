# PR #24 release-candidate validation checkpoint

Candidate under validation: original integrated PR #24 head `043f4b3`, HTML `f9faf30e118695aa3d93e1f058b0806978b250d58c6287a816fcc1575a61df08`, plus the focused render-path correction below. No merge/deployment/chain write is authorized.

The controlled active-renderer gap reproduced a six-tile snap on both live R15.56 and original PR #24. Six actual accepted key steps were preserved while drawing was suspended. A bounded, render-only waypoint queue now follows accepted solo steps (including turns) instead of classifying rendering lag as a relocation. Real local travel time is capped at 250ms; animation/shared/legacy timesteps remain unchanged. Queue data is never serialized or sent over a network, clears on map/position discontinuity or view/scope changes, and is capped at 128 points.

Corrected HTML: `b0c9cd4243fe595ebcc002d66a939390e8ffd88ba8445452bdb7e6b1c70c26dd`. Focused results: first draw advances 2.0825 rather than 6 tiles, then catches up; six accepted inputs/inventory preserved. Node contracts cover turn-path geometry, normal accepted-input cadence (48 matching old/new frames), existing gait/movement behavior (72 matching old/new frames), relocation/map/shared/legacy/inactive/reduced-motion/queue-limit guards. Startup 3D/FP/2D smoke passed. This is controlled browser draw suspension, not physical-phone performance.

The full matrix uses `tests/release/run-matrix.cjs`, one browser child at a time, a read-only HTML snapshot, exact source/script/helper/input fingerprints and per-job checkpoints. It skips passed identical entries on resume and continues independent checks after a failure. New logs go to `/workspace/.magic-over-setup/rc24`; overnight evidence is preserved. Final report will distinguish passed, reused, failed and unavailable checks. Do not treat this checkpoint as release approval.

PR #24 shares its feature head with PR #23. Validation commits advance that draft feature head only; original implementation commits remain in history and main is untouched. The complete matrix is to run against the exact resulting PR #24 HTML, without replaying unchanged successful focused checks.
