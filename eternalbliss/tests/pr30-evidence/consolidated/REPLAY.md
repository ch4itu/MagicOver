# Replay and rollback

Run from the repository root with Chromium and Playwright available. The exact gameplay candidate is `b04490efd68c73c0e0a2a6c390506de2f6f33143` and the browser matrix pins the HTML digest. Run one browser suite at a time. Set `CHROMIUM_EXECUTABLE` if Chromium is installed outside the default path.

```bash
git show 12ecd8d07f0ef3d0b0627d01a8fc46edf6530a50:eternalbliss/index.html >/tmp/eb-consolidated-base.html
git show 047b3c91d0c433c0615d4b319367be8566a96d5d:eternalbliss/index.html >/tmp/eb-consolidated-v1.html
git show 76664bfa954062cf15de7b63d21edb26d44e7847:eternalbliss/index.html >/tmp/eb-consolidated-quests.html
export NODE_PATH=/workspace/.magic-over-setup/browser/node_modules
export EB_EVIDENCE=/tmp/eb-consolidated-replay
export EB_SCOPE_BASELINE=/tmp/eb-consolidated-base.html
export EB_ROLLBACK_SOURCE=/tmp/eb-consolidated-v1.html
export EB_QUEST_BASELINE=/tmp/eb-consolidated-quests.html
export EB_RC_CANDIDATE=9cebdf7920b20d96e71b9604bf29173e6ad443098af796de2f9ed1590132d589
node eternalbliss/tests/release/pr30-matrix.cjs
```

For a rendering comparison, run `tests/render-benchmark.cjs` with `EB_SOURCE` set to the saved baseline HTML and `EB_RC_CANDIDATE` set to its digest (`efd0588467394bbb5bd96271b7913ff0f5d7a02db7caa178a52c5f1f44b4f0bd`). Run again with the checkout HTML and candidate digest. Use a different evidence directory for each scene and build. The controls are `EB_RENDER_SCENE` (`rest-first-person`, `rest-overhead`, `bell-orchard-first-person`, or `shellwick-overhead`), `EB_RENDER_PROFILE=portrait` for 390 × 844, and `EB_RENDER_SAMPLES`. The first-person Rest uses 100 samples. The other scenes use 60. All runs warm 20 frames at fixed noon.

To roll back this integration, export or copy the current browser save and load `12ecd8d07f0ef3d0b0627d01a8fc46edf6530a50` on the same origin. The earlier HTML reads terrain v2 saves and has no UI for the new orchard quest. Preserve a copy if you want to retain that quest’s progress for a later return to the new build. To roll back the coastline migration itself, use the separately archived v1 save and pre-coast HTML. [Earlier rollback instructions](../final-integration/REPLAY.md)
