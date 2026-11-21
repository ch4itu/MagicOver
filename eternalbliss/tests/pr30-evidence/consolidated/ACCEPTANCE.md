# Consolidated candidate

Gameplay commit `b04490efd68c73c0e0a2a6c390506de2f6f33143` and HTML SHA-256 `9cebdf7920b20d96e71b9604bf29173e6ad443098af796de2f9ed1590132d589`.

**18 checks passed against this exact HTML.** The integrated suite passed all 17 of its original jobs. A focused static architecture check then added an eighteenth result to the same pinned matrix. The run covered coastal access, generated moorings, save migration and archived rollback, old quest contracts, resident work and dialogue, fresh Millwright’s Tally and Harvest Bell progress, Shellwick sailing, 2D and 3D startup, portrait and landscape touch and combat, horse and boat interactions, Atlas, and the stubbed shared-world save guard. The shared-world protocol scripts matched the scope baseline.

## Opening and continuation

A new desktop Chromium guest started with 100 gold, zero XP and no Harvest Bell flags. Using the visible game controls, the guest found Mira, pinned Millbend, recovered the key, entered the archive, defeated the Warden in first person and collected Mira’s reward once. The guest bought and rode a horse, accepted The First Picking from Nessa, read the two tally boards at Bell Orchard, and returned. The wrong answer paid nothing. The right answer paid 22 gold and the quest stayed complete after reload. The same visit also completed the existing three-landmark survey. The route took **5 min 11 s** at fixed noon, with no injected gameplay state. A separate physical 15–30 minute playtest with an advancing clock is still outstanding.

- [Mira introduction](gameplay/natural-mira-start-9cebdf79.png), [Archive Warden result](gameplay/natural-warden-defeated-9cebdf79.png), [first ride](gameplay/natural-first-ride-9cebdf79.png)
- [North tally](gameplay/natural-orchard-tally-1-9cebdf79.png), [south tally](gameplay/natural-orchard-tally-2-9cebdf79.png), [Nessa’s reward](gameplay/natural-nessa-reward-9cebdf79.png)
- [Fresh-route events](gameplay/natural-harvest-events.jsonl)
- [Portrait touch Atlas](gameplay/touch-atlas-portrait-9cebdf79.png), [landscape touch Atlas](gameplay/touch-atlas-landscape-9cebdf79.png)
- [Portrait Lantern Quay](gameplay/lantern-first-person-portrait-9cebdf79.png), [landscape Lantern Quay](gameplay/lantern-first-person-landscape-9cebdf79.png)

## Verification bounds

Both phone layouts used browser-native Chromium touch events. The compact landscape panel’s Meet Mira action and the portrait objective both opened the real resident list. Physical Android, hardware GPU frame pacing, and real concurrent MagicBlock sessions remain unverified. Keep PR 30 draft until the release blockers are resolved.

- [Exact-hash matrix](matrix/pr30-matrix.json), [individual logs](matrix/), [ledger](../ledger.jsonl)
- [Matched performance comparison and screenshots](PERFORMANCE.md)
- [Replay and rollback](REPLAY.md)
