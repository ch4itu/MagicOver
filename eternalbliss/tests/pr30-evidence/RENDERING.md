# PR 30 rendering benchmark

The comparison isolates renderer commit `c3b8963` from the later gameplay changes. It uses the same 844 × 390 Chromium viewport, SwiftShader, fixed noon, scene order, twenty warm-up renders and ninety recorded frames per scene. The harness times frame intervals, main-thread draw time, render submission, terrain sync and actor sync. The renderer reports submitted geometry and call counts. Camera poses are fixed after scene entry. Water and ship animation continue normally.

| Scene | Frame median before → after | Frame p95 before → after | Frame p99 before → after | Main-thread draw p95 before → after |
| --- | ---: | ---: | ---: | ---: |
| Wayfarer’s Rest overhead | 16 → 20 ms | 938 → 793 ms | 1,576 → 1,205 ms | 16 → 21 ms |
| Wayfarer’s Rest first person | 16 → 17 ms | 2,774 → 1,885 ms | 3,067 → 2,836 ms | 9 → 13 ms |
| Lantern Quay afloat, first person | 21 → 17 ms | 1,437 → 1,342 ms | 1,578 → 3,507 ms | 42 → 6 ms |

Afloat, material clones fell from 450 to zero in the recorded sample. The cached materials keep the opacity and depth-write settings of the previous first-person boat. Opaque terrain now draws before water so the depth test can discard hidden water fragments. The water shader skips wave work at the fully faded horizon, where the result is the same sky color.

Terrain sync and actor sync reached at most 1 ms per recorded frame across these stationary scenes. Frame intervals frequently exceeded main-thread draw intervals, including 21 of 90 afloat frames over 100 ms both before and after. This is consistent with rendering or delivery outside the measured JavaScript span. The run does not isolate garbage collection, OS scheduling or software GPU completion time. First-person Rest still submits roughly 350,000 triangles. Terrain rebuilds during sustained travel, physical Android and a reliable frame budget need separate evidence.

- [Baseline samples and source hash](render-f3d1b298.json)
- [Optimized samples and source hash](render-7a5a19d7.json)
- [Before afloat screenshot](benchmark-quay-afloat-first-person-f3d1b298.png) and [after afloat screenshot](benchmark-quay-afloat-first-person-7a5a19d7.png)
