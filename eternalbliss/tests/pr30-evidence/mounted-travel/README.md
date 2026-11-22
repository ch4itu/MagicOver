# Solo Farlands mounted travel

Gameplay commit `2ab8006b1b5457d24c765d0fef8487bb1f999ce1`. Release HTML SHA-256 `053948281fc9795dd9cd1ef955e13ca89f515d74452d7723416560b31c1891b4`. Comparison build and same-origin rollback are preview v15, HTML `5d5dc2f1b638d2643a1ec5a4a890a48861c31923d76f12d16622e48cbebaf9b3`.

The 2D horse previously sampled the player tile every 80 ms while a held mounted step was eligible every 65 ms. The horse and rider had independent CSS transitions. The mounted sprite now shares the rider's interpolated position. Dismounting returns the horse to its world tile. In 3D the old horse gait discarded advances of 0.8 tiles or more. The horse now advances its stride by the distance the rider actually travels along accepted steps, including on slow frames.

A second hitch occurred when entering uncached Farlands terrain. A single tile request could synchronously generate 1,024 terrain cells. The bounded cache now computes a cell on its first read. Four alternating comparisons produced identical terrain digests over 48,166 sampled results each and exercised saved tile edits. The cold run's worst strip query took 28.7 ms on v15 and 1.9 ms on the candidate. Results of the three warm comparisons are included in the trace archive.

On SwiftShader the candidate draws the outdoor solo mounted 3D scene at 65% of its normal linear pixel resolution. It restores the previous ratio on dismount. The HUD and hardware-accelerated render resolution keep their existing settings. This makes the software-rendered scene softer while riding.

## Browser trial

Each run buys and mounts the horse with the game's controls, places the player at (4967, 4604), then holds right for at least 48 traversable tiles, crossing terrain-patch and chunk boundaries. Desktop runs use a held arrow key in 2D and overhead 3D. The emulated portrait first-person run uses a held touch on the circular D-pad. The fixture fixes the in-game date. The two matched rounds reverse build order. Input timings count accepted steps. Frame callbacks and WebGL submissions are measured on the browser side. Display presentation and physical-device performance were outside this trial.

| Measured on the active ride | Round 1 v15 → candidate | Round 2 v15 → candidate |
| --- | --- | --- |
| Classic horizontal saddle error, p95 | 24.08 → 0 px | 26.08 → 0 px |
| Overhead frames with frozen stride while advancing | 6 → 0 | 4 → 0 |
| First-person frames with frozen stride while advancing | 4 → 0 | 10 → 0 |
| Overhead WebGL submission gap, p95 | 297 → 250 ms | 308 → 240 ms |
| First-person WebGL submission gap, p95 | 261 → 234 ms | 273 → 250 ms |
| Overhead accepted-step gap, p95 | 74 → 212 ms | 273 → 71 ms |
| First-person accepted-step gap, p95 | 287 → 256 ms | 284 → 265 ms |

All 12 rides reached the end of the lane and dismounted with horse ownership intact. The candidate tracked the saddle within one pixel on both axes in every sampled 2D frame. Each sampled 3D rider and mount shared the same x coordinate. No tested frame reported missing terrain under the rider. The candidate restored the normal SwiftShader scene resolution after all four 3D dismounts.

Timing varies on the software renderer. Each 3D run contained 29–48 measured submission intervals. Upper-tail submission gaps improved in both matched rounds, and the overhead input results varied in opposite directions. The candidate's worst individual first-person interval reached 426 ms. These measurements support a fix to saddle drift and gait interruption and a limited improvement in submission timing. Physical-device smoothness remains unverified.

[Matched measurements](matched-travel.json) · [Diagnostic browser runs](https://github.com/ch4itu/MagicOver/actions/runs/45008423504) · [Matched browser runs and release checks](https://github.com/ch4itu/MagicOver/actions/runs/45008927200) · [Raw samples and test logs](mounted-traces.zip)

## Regression and publication

All 18 integrated checks passed on the candidate hash, including saves and rollback, geography and authored routes, mobile controls, boats, combat and the shared-world boundary. The controlled six-step long-frame check and four Clean View indoor-exit profiles also passed. The public two-player Devnet measurements are recorded separately for v15. This solo comparison performed no new public transactions.

[Release matrix](release-matrix.json) · [Candidate screenshots in all three views](mounted-views.webp)

Published preview version 16 at the [existing EternalBliss preview](https://eternalbliss-rc-abc17827.speedy-stork-2900.chatgpt.site/). The preview source commit is `f341ff510c89b6769e4c5ab1b63c5e1238624d81`. A separate HTTPS fetch returned bytes identical to the tested release HTML, and a browser displayed the hosted Solo adventure and Shared world entry points. Version 15 is the same-origin rollback.
