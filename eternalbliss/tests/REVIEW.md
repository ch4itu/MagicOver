# R15.56 development review — not deployed

Reviewed the local AGENTS/handoff, [D-pad PR #2](https://github.com/ch4itu/MagicOver/pull/2), [FP combat Issue #3](https://github.com/ch4itu/MagicOver/issues/3), and [revenue-policy PR #4](https://github.com/ch4itu/MagicOver/pull/4). Free play and non-intrusive-advertising-only policy remain constraints; this change adds no monetization.

The Mira fix is on `fix/eternalbliss-mira-interaction-r1556` (`bc7b558`): a pending Harvest Bell return wins an equal-distance NPC tie. Explicit clicks, nearer NPCs and shared-world selection are unchanged. Final-state fixtures use actual Space, exact +65 gold/+85 XP, once-only completion and save/reload. They do not establish the natural quest route.

The combat/control branch builds on Mira. Solo Farlands FP keeps the eye camera during a fight, uses a camera-local copy of the existing sword arm, stages the foe ahead, and preserves look preference. Mounted combat keeps the view clear while ordinary mounted rendering is preserved. Damage/rewards/saves and the shared-world protocols are not changed. The D-pad CSS incorporates PR #2 without merging it.

## New evidence

The append-only [ledger](evidence.jsonl) pins HTML and test hashes; raw logs and screenshots are retained under `/workspace/.magic-over-setup/eternalbliss-evidence/`.

- HTML `2405d6fb937cb580107bb7fbf8434fed6bb32ca83c06020e9faa3f1c9fc8d127`: actual Attack/Magic/Heal and enemy replies on desktop, portrait and landscape, mounted/unmounted. Expected fixed-RNG outcomes passed. Initial mounted landscape chest projection overlapped the HUD; it was corrected, not accepted as visually clear.
- HTML `920e581fc414fc0efeb3943fe7a1751c08f3d89709f5474bc875af08b1746779`: six corrected framing combinations, fixed-age rendered poses, original aim restored; five control profiles normal/Clean View; desktop and phone overhead/FP outcomes equal for attacks, magic, potions, victory XP/gold, defeat and fleeing; wolf/dragon/crawler, a structure-adjacent staging case and stubbed shared scope checks passed.
- HTML `09dfa95be03e56d4c32603e0a53e1705aa39d7665199736f8315974b3932ae55`: restricts horse hiding to FP combat. Focused mount rendering/ownership check and 3D/FP/2D smoke passed. Previously passed combat checks are reused because the combat branch of that condition is unchanged.
- Mira's original eight initial-clock fixtures passed. A later fixed-clock run completed seven cases; its last Space was swallowed by UI focus. The harness now explicitly returns focus to the world, and only that unfinished case was rerun and passed. Logs preserve the distinction.

Browser evidence is real headless Chromium with emulated phone viewports, not a physical phone. The original first smoke and Mira run briefly overlapped; no performance conclusion is drawn. Subsequent suites ran serially. Fixed-age pose pictures are render checks, not real-time animation measurements.

## Remaining release gates

These are draft development changes. The historical 25/25 and 22/22 suites are not present in this checkout; no historical pass is presented as a new result. A natural full quest playthrough, physical Android performance and live two-client shared-world behavior remain unverified. The stubbed scope check is not multiplayer evidence. Run one full release matrix on the exact candidate proposed for deployment, after visual review. Do not merge, deploy or write live chain data without approval.

Live baseline: `77b0b430995c9b37b9369bab1399ae5ec706ef80`; rollback: `6321565231a81204e4c92c801ef3b9eca3175648`. Both remain intact.
