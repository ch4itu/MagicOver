# PR #24 touch-tablet D-pad correction

This focused follow-up supersedes the D-pad capability coverage in the earlier [release report](RELEASE-READINESS.md). That suite incorrectly expected its large touch profile to hide the D-pad and omitted tablet landscape coverage. The independent audit correctly identified a release blocker.

**The CSS blocker is corrected and focused D-pad checks pass.** Validation completed on 16 November 2026. PR #24 remains a draft; physical-device review, the separate touch-click limitation below, and release approval remain open.

Previous candidate: commit `2746efbb2576ec48f5c47573314dd713aacbc3ce`, HTML SHA-256 `b0c9cd4243fe595ebcc002d66a939390e8ffd88ba8445452bdb7e6b1c70c26dd`.

Corrected HTML SHA-256: `18ecff7c1bfd4f12502b8216e2720b8b3c32e2f62d0fde033b719c33906bdf0c`.

CSS correction commit: `719343d76340a05efb36a7a5915d1ce9156a03d6`, feature branch `fix/eternalbliss-touch-tablet-dpad`.

## Exact application change

Replace the two size-based override blocks with:

```css
/* PR #2: keep desktop controls hidden by default; retain touch movement at every size, including hybrids. */
@media (max-width: 768px), (any-pointer: coarse) {
    #mobile-controls, body.eb-capture-clean #mobile-controls { display:grid!important; }
}
```

The former unconditional `min-width: 769px` / `min-height: 521px` hide rule is removed. Existing default hiding still applies to ordinary large mouse/keyboard desktops. Existing compact-width behavior is retained. Any available coarse pointer enables touch movement, even when the primary pointer is fine and supports hover. Battle visibility and modal movement blocking retain their existing rules.

The entire HTML differs only by this CSS replacement: three added lines, six removed lines. All 29 inline script blocks and every other byte are unchanged. No application JavaScript, gameplay, graphics, architecture, save, Smaran, MagicBlock, networking or protocol change is included.

## Focused evidence

Each source was checked at **18 profile/resolution cases × four modes = 72 mode checks**: overhead normal, overhead Clean View, first person normal and first person Clean View. The previous candidate missed the required D-pad in **24 checks**; the correction misses it in **zero**. On the correction, all **60 visible-control checks include actual accepted movement**, not visibility alone; the remaining 12 wide mouse-only checks preserve hiding and working keyboard input. There are also six HUD-state checks per source. See [the machine matrix](tests/dpad24/matrix.json), [raw ledger](tests/dpad24/ledger.jsonl) and [artifact hashes](tests/dpad24/artifacts.json).

| Device / CSS viewport | Actual pointer capabilities | Previous candidate | Corrected candidate |
| --- | --- | --- | --- |
| Desktop 1280 × 800 | Primary fine, hover, no available coarse | Hidden; keyboard works | Hidden; keyboard works |
| Tablet portrait 820 × 1180 | Coarse only, no hover | Hidden in all four modes | Visible; actual touch movement in all four modes |
| Tablet landscape 1180 × 820 | Coarse only, no hover | Hidden in all four modes | Visible; actual touch movement in all four modes |
| Phone portrait 390 × 844 | Coarse only, no hover | Visible; touch movement works | Preserved in all four modes |
| Phone landscape 844 × 390 | Coarse only, no hover | Visible; touch movement works | Preserved in all four modes |
| Hybrid 1280 × 800 | Primary fine/hover; both fine and coarse available | Hidden in all four modes | Visible; actual touch movement in all four modes |

All four requested boundary combinations were checked on mouse-only, touch-only and hybrid profiles, in all four modes. Actual `innerWidth`/`innerHeight` matched the requested CSS dimensions. “Shown” below includes actual touch or mouse-drag movement.

| CSS viewport | Touch-only before → after | Hybrid before → after | Mouse-only before → after |
| --- | --- | --- | --- |
| 768 × 520 | Shown → shown | Shown → shown | Shown → shown |
| 768 × 521 | Shown → shown | Shown → shown | Shown → shown |
| 769 × 520 | Shown → shown | Hidden → shown | Hidden → hidden |
| 769 × 521 | Hidden → shown | Hidden → shown | Hidden → hidden |

HUD fixtures check Actions disclosure where offered (the intermediate tablet layout exposes Atlas directly), Atlas modal keyboard blocking, location-card collapse/restore, battle visibility/input blocking, and D-pad restoration with accepted movement after battle. Battle checks include actual touch input at the former joystick position and unchanged player coordinates. Mode/HUD setup uses mouse fixtures where needed; the separate touch Atlas limitation is explicit below.

One **full-DPR 3D/FP/2D startup smoke** passed on the exact corrected HTML, with no page exceptions. [Smoke log](tests/dpad24/corrected-startup-smoke.log). The [source audit](tests/dpad24/scope-audit-final.json) proves that the entire application differs only by the declared CSS replacement. Unaffected historical gameplay suites remain tied to their original hashes; the broad quest/combat/save/vehicle/world matrix was not repeated.

The append-only repository ledger retains its prior **203 rows** and adds **58 actual records** from this work, including baseline failure observations and changed-input retries. These are evidence rows, not a count of independent passed playthroughs. Per-record source and test-version hashes are retained. Early fixture failures (incorrect Actions-layout expectation, an unsettled 2D target, software-renderer scroll timeout) and the touch-click attempts remain in raw logs. Completed checks were reused when only fixture/recording paths changed. Checkpoint compatibility is pinned to the exact final runner; future assertion edits automatically invalidate that compatibility. The [checkpoint-only scan](tests/dpad24/checkpoint-no-replay.json) retained all 26 corrected profile/HUD/tap checkpoints with **zero browser launches**.

Chromium pointer capabilities are checked using actual CSS media queries, not a mocked `matchMedia`. Touch-only profiles use primary/any coarse with no fine pointer; desktop uses primary/any fine and no coarse pointer. Hybrid emulation uses Chromium Blink pointer settings so primary fine and both available fine/coarse queries are true. Hybrid touch gestures are real CDP-generated DOM touch events; this is not physical hybrid-device certification.

The initial full-resolution baseline completed several checkpoints. Those results are retained; the remaining controls checks use device scale factor 0.5 for efficient software rendering, with CSS viewport sizes and pointer capabilities unchanged. The original harness, interrupted-run log and per-record script hashes preserve that distinction. Compact desktop boundary checks changed from synthetic touch to actual mouse drag and are tested with the revised harness. No physical-phone performance claim follows from these checks.

The six main device profiles run with continuous rendering. Added boundary and HUD-state checks pause drawing while keeping native DOM layout, pointer events and accepted/blocked movement assertions active. Mode and HUD fixtures can be arranged with native mouse input; this does not certify touch activation of every HUD button. D-pad gestures on touch-only and hybrid profiles use actual CDP touch events, with pointer type/recipient and changed coordinates/move count checked.

**Separate touch-click limitation, left unchanged:** after joystick input, raw touch taps reached Atlas as `pointerdown`/`pointerup` but did not synthesize a `click`. The Atlas guard was clear. This reproduced on the previous candidate's phone profile as well as the corrected tablet profile; an alternative synthetic tap command also failed to arrange the fixture reliably. Mouse input opened the dialog and allowed the HUD-state checks. The logs retain these failed attempts. This patch does not claim touch Atlas activation after joystick input is verified, and does not change application JavaScript to address it. Physical-device review is still needed to distinguish the automation path from device behavior. This is separate from the deliberate land tap-to-walk exclusion investigated below.

Representative traces: [previous phone](tests/dpad24/before-phone-touch-hud-diagnostic.log), [corrected tablet](tests/dpad24/after-tablet-hud-recipients.log), [mouse-staged tablet HUD](tests/dpad24/after-final-tablet-portrait.log).

## Tap-to-walk finding

**Touch land tap-to-walk is pre-existing behavior, not a regression introduced by this CSS patch.** On live R15.56 (`77b0b43`), the previous candidate and the correction, an actual touch `click` on an unobstructed legal land tile leaves the player stationary; a mouse click on the same tile moves the player. This was observed in both overhead 3D and 2D. The unchanged 3D handler explicitly requires `event.pointerType !== 'touch'`; the 2D handler explicitly rejects `event.pointerType === 'touch'`. First-person canvas taps use the existing interaction target rather than land walking. No tap-to-walk behavior was changed.

[Live R15.56 probe](tests/dpad24/live-r1556-tap.log) · [Previous candidate](tests/dpad24/before-v4-remaining.log) · [Corrected candidate](tests/dpad24/after-final-tap.log). The actual touch click was recorded in these land probes, distinguishing intentional exclusion from the Atlas click-synthesis discrepancy.

## Control screenshots

| State | Previous candidate | Corrected candidate |
| --- | --- | --- |
| Tablet landscape, normal | [Image](tests/dpad24/before-tablet-landscape-normal.png) | [Image](tests/dpad24/after-tablet-landscape-normal.png) |
| Tablet landscape, FP Clean View | [Image](tests/dpad24/before-tablet-landscape-fp-clean.png) | [Image](tests/dpad24/after-tablet-landscape-fp-clean.png) |

These use the same CSS viewport and legal fixture lane, at DPR 0.5. The native climate clock and actual accepted movement affect lighting, location text and framing; these are control-presence captures, not an art comparison. The screenshots were manually inspected. No graphics or asset code changed.

## Release boundaries

PR #24 remains a draft pending approval. No merge, deployment or blockchain write is authorized or performed. Free play and the non-intrusive-advertising-only policy remain unchanged; monetization remains unimplemented. Physical Android, human visual approval and real two-client shared-world coverage remain open from the earlier report.
