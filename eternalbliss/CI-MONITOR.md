# EternalBliss CI monitor (no OpenAI API billing)

Workflow: `.github/workflows/eternalbliss-monitor.yml` on `automation/eternalbliss-release-candidate`.

## What runs
- On pushes to the two automation branches and on pull requests touching EternalBliss: Node-based geography and quest contracts, inline-script syntax validation, SHA-256 report, Chromium/Playwright save migration, coastal boarding and desktop vehicle checks.
- Artifacts retained for 14 days. Failure-only comments on Issue #29 for push runs, when repository Actions permissions permit. Every run has a GitHub Actions summary.
- No Codex execution, OpenAI API calls, merges, deployment or blockchain changes.

## Scope and limitations
- This is a smoke/regression monitor, not the complete 17-check release matrix or a performance benchmark.
- Browser tests use GitHub-hosted Linux Chromium, not physical Android or live MagicBlock multiplayer.
- The broad `--require-waterfront` assertion is intentionally not used because some other coastal settlements remain unfinished; existing audit still checks expected baseline contracts.
- First run must be inspected for runner-specific failures (browser installation, network assets or fixed fixture expectations). Do not report CI green until an actual run passes.
- GitHub Actions minutes and artifact storage are subject to repository plan quotas; no additional OpenAI API cost does not mean unlimited free compute.

## Scheduled monitoring
GitHub `schedule` workflows execute only from the default branch. To avoid changing `main` or merging anything without approval, this workflow currently uses push, pull_request and manual workflow_dispatch triggers. Adding a daily schedule later requires explicitly authorizing a workflow on the default branch.

## Review
Open the repository Actions tab, select **EternalBliss regression monitor**, and inspect run summaries and artifacts. PR #30 should remain draft until all release gates are approved.
