# Replay and rollback

From the repository root, use Chromium with Playwright and run the browser suites sequentially.

```bash
git show c75f2d9c0ceca1d4aa27ea86d66e1d1169dfa42d:eternalbliss/index.html >/tmp/closure-base.html
git show 12ecd8d07f0ef3d0b0627d01a8fc46edf6530a50:eternalbliss/index.html >/tmp/closure-prebatch.html
git show 76664bfa954062cf15de7b63d21edb26d44e7847:eternalbliss/index.html >/tmp/closure-quest.html
git show 047b3c91d0c433c0615d4b319367be8566a96d5d:eternalbliss/index.html >/tmp/closure-v1.html
export NODE_PATH=/workspace/.magic-over-setup/browser/node_modules
export EB_RC_CANDIDATE=abc17827a03e60d5594581b0c5b7273bf38aea87d36f3f2aa6f1344145e0d350
EB_LIVE_JOURNEY=1 EB_EVIDENCE=/tmp/closure-journey node eternalbliss/tests/natural-harvest.cjs
EB_EVIDENCE=/tmp/closure-matrix EB_SCOPE_BASELINE=/tmp/closure-base.html \
  EB_ARCHITECTURE_BASELINE=/tmp/closure-prebatch.html EB_QUEST_BASELINE=/tmp/closure-quest.html \
  EB_ROLLBACK_SOURCE=/tmp/closure-v1.html node eternalbliss/tests/release/pr30-matrix.cjs
EB_EVIDENCE=/tmp/closure-render EB_RENDER_SCENE=rest-first-person EB_RENDER_SAMPLES=120 \
  node eternalbliss/tests/render-benchmark.cjs
```

For the rendering reference, repeat the benchmark with `EB_SOURCE=/tmp/closure-base.html`, `EB_RC_CANDIDATE=9cebdf7920b20d96e71b9604bf29173e6ad443098af796de2f9ed1590132d589` and a new evidence directory. Set `EB_RENDER_FENCE_TRIAL=1` on that baseline to repeat the instrumented queue experiment.

For application rollback, export or copy the browser save and load `c75f2d9c0ceca1d4aa27ea86d66e1d1169dfa42d` on the same origin. Both builds read terrain-v2 saves. Use a separate copy for the new sounding progress because the earlier UI does not expose that quest. [Separate archived-v1 rollback](../consolidated/REPLAY.md)
