# Shared first-person peer movement

Game commit `d2404c5868c2bf4e12ee813bac2d5760cecd6792`, HTML SHA-256 `5d5dc2f1b638d2643a1ec5a4a890a48861c31923d76f12d16622e48cbebaf9b3`. The comparison is the exact v14 HTML at PR head `a2fee97c962c566a0071409f841d32378167658b`, SHA-256 `9cbb061a692f9a8e9ae520d56204988ce9e672ba38a4f696d855bfc57f8cf190`.

## What changed

In shared, outdoor first person on SwiftShader, the old renderer submitted one large batch for each terrain material across a 128 × 128 tile window at spawn. The whole window intersected the camera volume regardless of heading. The candidate divides that same rectangle into 24-tile batches and uses the existing bounds for eye-camera and shadow-camera culling. It retains the same tile window, geometry, materials, distance and resolution. The accelerated render path, transport, signing, authoritative movement, program and save format are byte-for-byte unchanged outside this renderer edit.

At four camera headings at the shared spawn, the geometry probe counted 433,926 terrain triangles before culling on both builds. The old path submitted all 433,926 for each heading. The partitioned path submitted 173,442–233,014. At (75,75), (80,75), (1,1) and (299,299), the sorted surface triangles and their attributes, instance transforms, instance colours and shadow flags matched between the unpartitioned and partitioned paths. These are geometry checks on a fixed summer fixture. The full browser scene measurements below include actors and sky.

## Paired public movement trial

The same two independently registered Smaran accounts played v14 and then the candidate in isolated Chromium contexts on the same runner and Devnet world. The public endpoint was `https://devnet-as.magicblock.app`. Normal browser fetch and WebSocket delivery were used. The harness instruments the checked-out HTML in memory and timestamps the ordinary movement entry point, signature, request start and response headers, world-account callback, reconciled peer state and renderer. Test hooks are absent from the published game.

| Median in ms | 2D v14 | 2D candidate | FP v14 | FP candidate |
| --- | ---: | ---: | ---: | ---: |
| Input to signed movement | 53 | 57 | 738 | 477 |
| Submission to browser HTTP response headers | 201 | 224 | 713 | 455 |
| Submission to peer world-account callback | 206 | 237 | 1,061 | 684 |
| Account callback to reconciled state | 0.2 | 0.3 | 0.2 | 0.3 |
| Reconciled state to avatar placement | 236 | 237 | 727 | 491 |
| Submission to avatar placement | 455 | 460 | 1,786 | 1,166 |
| Frame interval, median / p95 | 31 / 47 | 34 / 44 | 723 / 753 | 465 / 492 |

Four isolated one-tile movements per build per mode, two per direction of observation. Placement counts the first DOM position write or completed WebGL submission whose avatar coordinate is within 0.48 tile of the confirmed destination on the movement axis. It measures avatar placement in the scene. The separate facing-camera test checks the peer mesh draw. Frame intervals pool both clients: 280/271 for 2D and 64/66 for FP (before/after). They measure browser work, with no display scanout timestamps. The medians of stages need not sum. Full-scene median triangles per FP submission fell from 435,934 to 216,668; median draw calls rose from 124.5 to 183.

The browser handled genuine WebSocket notifications immediately before reconciled world snapshots in both modes. Once the callback ran, median decode and reconciliation took under one millisecond. The sizeable delays before the callback and before the next submitted frame shrank when rendering work fell. Browser HTTP timings also include renderer scheduling. The earlier 6.9–7.9 second legacy HTTP/SwiftShader observations did not recur on this runner, and these measurements cannot retrospectively apportion those individual seconds.

[Paired raw timing summary](paired-timings.json) · [Independent finalized move receipts](movement-receipts.json) · [Public trial and integrated checks](https://github.com/ch4itu/MagicOver/actions/runs/45005148367)

## Lifecycle and release verification

The candidate recovered a peer movement after an induced network cut, retained the session on a failed leave, released the seat on retry, and restored the same public identity after a reload and sign-in. Both players walked to the same inn with signed cardinal moves, disappeared from each other's outdoor view on separate entry, saw each other indoors, exited and left. A subsequent independent finalized read of all 32 world seats found neither test owner. All 51 distinct candidate signatures for which the browser recorded a successful RPC acknowledgement in this paired run subsequently showed finalized success. The eight profiled candidate movement receipts showed zero fees and zero signer balances. This set excludes rejected requests and carries no mainnet cost estimate.

The exact candidate passed all 18 release-matrix jobs on their first run and all four Clean View profiles. Coverage includes the natural Harvest Bell route, mobile input, boats, mounts, combat, save migration and rollback. The game program was unchanged. The earlier program-only test results are historical. Physical Android and accelerated GPU timings remain open.

[Exact-hash release matrix](release-matrix.json)

## Facing-camera trial

A new pair of Smaran identities used the same facing positions on each build, three tiles apart. Each first-person observation additionally required a remote model mesh to reach the main eye camera's render callback in that frame. The candidate observer also rendered two further signed movements while its WebSocket was disabled. During that interval it recorded four HTTP world reads, two state ingestions and zero socket notifications.

| Public trial, ms | v14 | Candidate |
| --- | ---: | ---: |
| Median submission to 2D avatar placement | 508 | 477 |
| Median submission to visible FP avatar placement | 1,766 | 1,059 |
| Median FP submission to account callback | 1,038 | 643 |
| Median FP callback to reconciled state | 0.3 | 0.3 |
| Median FP reconciliation to drawn avatar | 721 | 417 |
| 2D frame interval p95 (227 / 241 intervals) | 56 | 54 |
| FP submission interval p95 (63 / 76 intervals) | 727 | 444 |

Four measured moves per mode per build. A separate read-only RPC client started polling from the browser request event. For the four FP transactions it first observed finalized status after 552–583 ms on v14 and 193–638 ms on the candidate. These include polling delay and RPC transit. Finality observation and the browser's confirmed world updates can arrive in either order. No physical display or device timing was measured. Every profiled movement has an independently fetched transaction receipt in the trace archive.

[Visible peer screenshot](visible-peer.webp) · [Facing-camera timings](facing-timings.json) · [Facing-camera trial](https://github.com/ch4itu/MagicOver/actions/runs/45005959502) · [Raw events, receipts, geometry probes and regression logs](performance-traces.zip)

## Publication and rollback

Published Sites version 15 to the [existing release preview](https://eternalbliss-rc-abc17827.speedy-stork-2900.chatgpt.site/), using preview-source commit `c986f386b5a2997a3869984f12d49406709de0a6`. An independent HTTPS GET of the preview root returned bytes identical to the game at `d2404c5`. A separate browser opened the hosted game and displayed its Solo adventure and Shared world entry points. Version 14 is the same-origin rollback. Production GitHub Pages and the deployed game program were outside this change.

For replay, the public trial scripts and runner definitions are pinned in the two linked runs. They register disposable Devnet players only when explicitly opted in. Both trials used Chromium 141.0.7390.37 and the software renderer with two 900 × 560 viewports. Browser contexts use an instrumented copy of the selected source; the release hash refers to the uninstrumented, deployable HTML. The original mixed HTTP/SwiftShader trial used a different environment.
