# Frame delivery and terrain culling

The comparison uses Chromium and ANGLE Vulkan SwiftShader at 844 × 390, with the same noon, fixed poses, 20 warm-up frames and 150 measured frames in each scene. Frame interval is elapsed wall time between the game’s render callbacks. Submission is elapsed main-thread time in Three.js. The counters measure the main camera pass. Three.js resets them after its shadow pass.

| Scene | Median interval | p95 interval | p99 interval | Intervals over 100 ms | Main-pass calls | Main-pass triangles |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wayfarer’s Rest overhead | 17 → 17 ms | 827 → 919 ms | 1,200 → 1,303 ms | 32 → 36 | 347 → 348 | 59,464 → 59,488 |
| Wayfarer’s Rest first person | 16 → 20 ms | 1,910 → 1,570 ms | 3,677 → 2,244 ms | 39 → 35 | 227 → 274 | 349,958 → 180,676 |
| Lantern Quay afloat first person | 18 → 17 ms | 1,339 → 1,073 ms | 2,035 → 1,528 ms | 40 → 35 | 379 → 407 | 194,454 → 115,494 |

Solo first person now groups ground and opaque terrain instances into spatial patches. The view and shadow cameras can reject the patches outside their separate frusta. Visible geometry, materials, shadow settings and the shoreline blend remain the same. The Rest and Quay first-person main passes submitted 48% and 41% fewer triangles, with a corresponding increase in draw calls. At the Rest, CPU submission p95 rose from 7 to 25 ms. First-person interval p90 also rose at both locations (1,014 → 1,371 ms at the Rest and 900 → 943 ms at the Quay). The overhead control has no first-person culling and still varied from run to run.

Warmed terrain and actor sync each took at most 4 ms during these stationary samples. This experiment excludes cold terrain rebuilds and sustained travel. On this container the software GPU process was observed using about 178% CPU while the cgroup allowed two CPU equivalents. The earlier no-WebGL-submission control on the predecessor measured 16 ms frame p95. Together these observations implicate the rendering path under software load. They leave GPU completion, compositor scheduling, driver shader work and other host scheduling costs unresolved. A render callback also does not measure physical display presentation.

The longest frame intervals still exceed one second in both first-person scenes. Chromium emulation and the matched screenshots provide a comparison for this runner. Physical Android frame pacing and the severity of the stalls on hardware GPUs remain open.

- [Before JSON](performance/render-c1b17de5.json) and [candidate JSON](performance/render-efd05884.json)
- [Overhead before](performance/benchmark-rest-overhead-c1b17de5.png) and [after](performance/benchmark-rest-overhead-efd05884.png)
- [Rest first person before](performance/benchmark-rest-first-person-c1b17de5.png) and [after](performance/benchmark-rest-first-person-efd05884.png)
- [Quay afloat before](performance/benchmark-quay-afloat-first-person-c1b17de5.png) and [after](performance/benchmark-quay-afloat-first-person-efd05884.png)
- [Historical no-submission control](../render-a9c2af01-no-submit.json) and [replay instructions](REPLAY.md)

The separate coastal interaction check collected 30 recent intervals per profile on the same candidate. Median/p95 were 17/1,176 ms at 1280 × 800, 17/193 ms at 390 × 844, and 21/469 ms at 844 × 390. These short approach samples accompany the boarding and Look checks. They cover other viewport shapes and do not replace the warmed comparison.

## Controls on the final HTML

Each control warmed 20 frames and sampled 150 at the Rest in first person. Keeping the game draw loop while withholding WebGL submissions produced 21 ms frame p95 and zero intervals over 100 ms. Keeping normal WebGL submission while reusing the warmed shadow map produced 1,315 ms frame p95 and 30 intervals over 100 ms, compared with 1,570 ms and 35 in the ordinary run. In that replay CPU submission p95 fell from 25 to 5 ms, while frame p99 rose from 2,244 to 2,644 ms. The warmed shadow replay retained 22 Three.js shader programs across all 150 samples. These controls show that replacing the dynamic shadow pass alone leaves severe stalls. They do not timestamp driver compilation, GPU completion or compositor presentation. Dynamic shadows remain enabled in the game.

- [WebGL submission withheld](performance/render-efd05884-no-submit.json)
- [Warmed shadow map reused](performance/render-efd05884-reuse-shadow.json)
