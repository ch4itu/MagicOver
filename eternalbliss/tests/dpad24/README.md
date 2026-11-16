# Focused D-pad evidence

Read [the correction report](../../D-PAD-RELEASE-FIX.md). The before/after checks are complete; do not restart them on resume.

`matrix.json` selects the latest completed observation for each source/profile. `ledger.jsonl` retains every actual record, including baseline missing-control observations and the two compact mouse cases repeated after changing their input from synthetic touch to actual mouse. Failed/partial process logs remain alongside successful targeted runs. `artifacts.json` hashes the review files.

The archived `tablet-dpad-v*.cjs` files are the actual script versions referenced by the evidence, not additional scripts to execute. The current runner is `../tablet-dpad.cjs`. Its `checkpoint-compatibility.json` pins the current runner hash before allowing reviewed prior checkpoints; changing its assertions invalidates reuse unless compatibility is explicitly reviewed again. `checkpoint-no-replay.json` is a Node-only read of the completed corrected checkpoints, with zero browsers launched.

The working evidence directory is `/workspace/.magic-over-setup/dpad24`. Before/after source snapshots remain there; their exact bytes are recoverable from commits `2746efbb2576ec48f5c47573314dd713aacbc3ce` and `719343d76340a05efb36a7a5915d1ce9156a03d6`. Do not treat a missing working state file in a new environment as evidence that these tests never ran.

CSS capabilities come from Chromium itself. Hybrid available-pointer bits are emulated with Blink settings; `navigator.maxTouchPoints` is zero for that synthetic hybrid, so it is not a physical-device certificate. Six main profiles use continuous rendering; final added boundary/HUD checks pause drawing for efficiency. Fixture UI can use mouse input, while D-pad movement on touch/hybrid profiles uses CDP touch events with recipient/type assertions. Browser tests add one renderer-reference hook; production has none. Raw touch Atlas activation after joystick input remains explicitly unverified, with old/new traces retained. Land touch-to-walk rejection was independently observed on live R15.56 and both candidates.
