# EternalBliss autonomous development mandate

Base: `76664bfa954062cf15de7b63d21edb26d44e7847`. HTML SHA-256 expected: `4c08d431fdd3119e96125b4fd15c86d95f1bca6ae4094455e88ee3a9edf86c55`.

## Objective
Produce an integrated release candidate without requesting approval for routine development. Preserve the existing single-HTML, serverless, no-wallet-required game. Do not merge, deploy, modify blockchain programs, or make irreversible save changes without explicit approval.

## Ordered backlog
- [ ] Audit source and previous tests; independently verify baseline, console errors, movement, conversation, save/reload.
- [ ] Fix coastal geography (Shellwick, Pearl Banks, Lantern Quay, Amberport, other coastal sites): favor sensible coastline reshaping and enforce terrain-aware coastal tests. Relocate sites only with migration and evidence.
- [ ] Implement reversible, versioned save migration covering hero, horse, boats, terrain edits, interior positions, quest state. Preserve originals; no silent data loss.
- [ ] Complete distinct farming, coastal, and trading settlement identities while preserving IDs, paths, doors and schedules.
- [ ] Implement varied Hearthfield, Shellwick and Wayfarer's Rest quests using existing mechanics. Verify old active and completed quest compatibility.
- [ ] Improve the first 15 minutes: objectives, NPC readability, discovery, progression and exploration.
- [ ] Audit and improve mobile landscape/portrait controls, camera, minimap, travel, first person, combat and performance.
- [ ] Audit solo versus MagicBlock shared-world boundaries; implement only compatible client-side improvements, no protocol changes.
- [ ] Produce a single integrated release-candidate validation and draft PR with screenshots, exact SHA-256, pinned preview, tests, limitations and rollback.

## Working rules
1. Work on `automation/eternalbliss-codex` or a child feature branch, never main or Claude's preserved branch.
2. Keep commits focused. Use targeted tests and a short smoke check after each change; run the broad suite once at release-candidate stage. Keep a cumulative test ledger.
3. Limit automated repair attempts to three per isolated failure. On repeated failure, revert that isolated change, mark it BLOCKED, and continue independent tasks.
4. Do not weaken tests to hide failures. Record verified results separately from reused evidence. Physical Android and live two-client results must never be fabricated.
5. Keep existing stable IDs, internal NPC names, quests, saves, collision, movement and MagicBlock/Smaran behavior. Do not introduce a second engine or mandatory server.
6. Escalate only for irreversible migrations, on-chain or deployed protocol changes, unexpected credential/cost exposure, and final merge/deploy approval.

## Completion contract
Report implemented, deferred and blocked items, exact commits, SHA-256, preview, test evidence, screenshots, migration behavior, risks and whether a human approval gate remains.
