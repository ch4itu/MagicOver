# Focused R15.56 browser evidence

These are development-only Playwright scripts. Production remains one HTML file, with no added runtime dependency. Use an isolated guest profile; these checks do not need credentials or send chain transactions.

Start a static server from the repository root on port 8091. With Playwright available in Node's module resolution (this environment uses `NODE_PATH=/workspace/.magic-over-setup/browser/node_modules`), run one script at a time:

- `node eternalbliss/tests/smoke.cjs`: 3D startup, first person, look, return to overhead and 2D.
- `node eternalbliss/tests/mira.cjs`: fixed summer noon/evening final-state fixtures, two legal approaches, with/without a generic resident in a distance tie; actual Space, exact one-time rewards, save/reload. This is not a natural quest playthrough.
- `node eternalbliss/tests/controls.cjs`: five viewport/input profiles, normal/Clean View visibility, FP keyboard movement and pointer joystick where visible. Touch profiles are emulation, not physical phones.
- `node eternalbliss/tests/combat.cjs`: actual Attack/Magic/Heal buttons and enemy replies, desktop/portrait/landscape, mounted/unmounted, fixed RNG; eye view and overhead control case.
- `node eternalbliss/tests/framing.cjs`: opponent clear of the HUD and return to original aim; fixed-age guard, attack, magic, heal and enemy poses in actual WebGL. Pose captures do not measure real-time animation or performance.
- `node eternalbliss/tests/outcomes.cjs`: identical fixed-RNG overhead/FP outcomes on desktop and phone emulation, including victory, defeat and fleeing.
- `node eternalbliss/tests/edges.cjs`: wolf/dragon/crawler presentation, clear staging corridor by a structure, stubbed shared-world scope rejection. No live multiplayer evidence.

`browser.cjs` exposes the private renderer only in the served test response; the production HTML has no test hook. `EB_SOURCE` selects an alternate HTML candidate for the helper-based scripts. `EB_EVIDENCE` changes the output directory (default `/workspace/.magic-over-setup/eternalbliss-evidence`). Results append source SHA-256, script SHA-256, UTC timestamp and browser/profile to `ledger.jsonl`; committed summaries are in `evidence.jsonl`. Read the logs as well as the exit code. Do not rerun passed checks on unchanged inputs simply because a new session starts.

The historical 25/25 and 22/22 matrices mentioned in the handoff were not included in this checkout and have not been independently repeated. These focused checks do not establish physical Android performance, a natural full quest run, live multiplayer stability or production readiness. Run the release matrix once on the exact candidate proposed for deployment. No deployment is performed by these tests.

Rollback remains R15.55 `6321565231a81204e4c92c801ef3b9eca3175648`; live R15.56 baseline is `77b0b430995c9b37b9369bab1399ae5ec706ef80`.

`inn.cjs` captures only the named Wayfarer's Rest inn, overhead and FP, summer noon and 22:00, and checks its footprint/entrance. Fixed clocks are installed before app startup in fresh profiles. Use `EB_CAPTURE_ONLY=1 EB_LABEL=inn-before-final EB_SOURCE=<baseline HTML>` (and a separate candidate run) when refreshing pictures without repeating completed entrance checks. The initial capture attempts with a mid-session clock change were superseded; use the final comparison sheets below.

`corridor.cjs`: new large-foe/inn clearance regression, actual attack/reply and open-field guard comparison on desktop and phone emulation. See [CORRIDOR.md](CORRIDOR.md) for the reproduction, candidate hash and limits. This scenario supplements rather than repeats the completed matrices.
