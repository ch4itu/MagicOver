# Reproduce the PR 30 final integration

Use the release candidate branch, Chromium at `/usr/lib/chromium/chromium`, and Playwright. Commands run from the repository root. Use a new evidence directory for each HTML hash and run the browser suites sequentially. The browser harness intercepts the loopback HTML request and blocks external writes when a candidate hash is set.

```bash
git show b07121cc4995d3107f1660ba53aaa90870b5cf67:eternalbliss/index.html >/tmp/eb-final-before.html
git show 047b3c91d0c433c0615d4b319367be8566a96d5d:eternalbliss/index.html >/tmp/eb-final-v1.html
git show 76664bfa954062cf15de7b63d21edb26d44e7847:eternalbliss/index.html >/tmp/eb-final-quest.html
export NODE_PATH=/workspace/.magic-over-setup/browser/node_modules
export EB_SCOPE_BASELINE=/tmp/eb-final-before.html
export EB_ROLLBACK_SOURCE=/tmp/eb-final-v1.html
export EB_QUEST_BASELINE=/tmp/eb-final-quest.html
export EB_RC_CANDIDATE=efd0588467394bbb5bd96271b7913ff0f5d7a02db7caa178a52c5f1f44b4f0bd
EB_EVIDENCE=/tmp/eb-final-matrix node eternalbliss/tests/release/pr30-matrix.cjs

EB_RC_CANDIDATE=c1b17de53b6b0683856fd8e57655f5d63df9f96ba8ca5e84035f76301e02c0bc \
  EB_EVIDENCE=/tmp/eb-final-perf-before EB_SOURCE=/tmp/eb-final-before.html \
  EB_RENDER_SAMPLES=150 node eternalbliss/tests/render-benchmark.cjs
EB_EVIDENCE=/tmp/eb-final-perf-after EB_RENDER_SAMPLES=150 node eternalbliss/tests/render-benchmark.cjs
EB_EVIDENCE=/tmp/eb-final-perf-after EB_RENDER_SCENE=rest-first-person EB_RENDER_SAMPLES=150 \
  EB_RENDER_SKIP_SUBMIT=1 node eternalbliss/tests/render-benchmark.cjs
EB_EVIDENCE=/tmp/eb-final-perf-after EB_RENDER_SCENE=rest-first-person EB_RENDER_SAMPLES=150 \
  EB_RENDER_REUSE_SHADOW=1 node eternalbliss/tests/render-benchmark.cjs
```

The matrix fingerprints each check with the HTML, test code, helper and its external inputs. To retry an isolated failure on the same build, reuse its evidence directory and set `EB_RC_ONLY` to the job name. Omit `EB_ROUTE_RESUME` for fresh guest onboarding acceptance. When restoring a route for diagnosis, preserve the separate fresh-run record.

For a rollback of this integration, export or copy the current save, then use the standalone HTML from `b07121cc4995d3107f1660ba53aaa90870b5cf67` on the same origin. It reads terrain v2 saves. Rolling back the earlier coastline migration requires the preserved pre-migration slot and the pre-coast HTML from `047b3c91d0c433c0615d4b319367be8566a96d5d`, as exercised by the matrix. Keep a separate copy of the current v2 save before restoring that archived v1 record. See the [checkpoint](../../../AUTOMATION-CHECKPOINT.md).
