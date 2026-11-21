# Matched rendering measurements

The same Chromium/ANGLE Vulkan SwiftShader runner rendered both builds at fixed noon. The landscape profile was 844 × 390 and the portrait profile was 390 × 844. Each scene warmed 20 frames. The first-person Rest collected 100 intervals and the other scenes collected 60. Interval timings measure calls into the game’s draw loop. The submission timer measures elapsed JavaScript time in Three.js, and draw counts are from the main camera pass.

| Scene | Main-pass calls | Triangles | Median interval | Interval p95 | Interval p99 | CPU submission p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Rest overhead, landscape | 348 → 287 | 59,488 → 59,488 | 17 → 20 ms | 880 → 1,209 ms | 1,266 → 1,609 ms | 7 → 20 ms |
| Rest first person, landscape | 274 → 285 | 180,676 → 181,264 | 17 → 16 ms | 1,444 → 1,441 ms | 1,576 → 2,180 ms | 8 → 7 ms |
| Bell Orchard first person, portrait | 47 → 59 | 104,122 → 106,734 | 16 → 16 ms | 708 → 738 ms | 1,776 → 1,419 ms | 8 → 4 ms |
| Shellwick overhead, landscape | 403 → 347 | 89,524 → 89,780 | 21 → 19 ms | 909 → 996 ms | 1,057 → 1,481 ms | 8 → 10 ms |

Static regional building meshes now share prebuilt geometry grouped by existing material. The independent geometry check compared world-space triangles, size and shadow flags. It found 26 → 16 meshes for a Hearthmere cottage, 32 → 19 for a Lantern Keys shop, and 64 → 13 for a hall. The Rest overhead submitted 61 fewer draws with the same triangle count. The orchard scenes include the new belfry and cloth-marked props, which add visible geometry.

The trial that moved sky work later and skipped invisible parts of its shader recorded 1,579 ms p95 in the 100-frame Rest scene, compared with 1,444 ms on the baseline. CPU submission p95 went from 8 to 15 ms. That trial was removed. [Reverted trial data](performance/sky-trial-reverted.json)

The retained batching reduces submission work for buildings. The measured frame tail remains unsuitable for release on this SwiftShader runner. The overhead p95 worsened in the observed pair, and the first-person p99 at the Rest increased. Terrain and actor sync stayed near zero in the stationary samples. [Earlier exact-hash WebGL and shadow controls](../final-integration/PERFORMANCE.md) implicate the software rendering submission path, but leave hardware GPU and compositor timings unmeasured. The environment limits the container to two CPUs, so these results cannot establish physical Android performance.

## Before and after

- Rest overhead [before](performance/rest-overhead-before.png), [after](performance/rest-overhead-after.png), [before JSON](performance/rest-overhead-before.json), [after JSON](performance/rest-overhead-after.json)
- Rest first person [before](performance/rest-first-person-before.png), [after](performance/rest-first-person-after.png), [before JSON](performance/rest-first-person-before.json), [after JSON](performance/rest-first-person-after.json)
- Bell Orchard first person [before](performance/bell-orchard-first-person-before.png), [after](performance/bell-orchard-first-person-after.png), [before JSON](performance/bell-orchard-first-person-before.json), [after JSON](performance/bell-orchard-first-person-after.json)
- Shellwick overhead [before](performance/shellwick-overhead-before.png), [after](performance/shellwick-overhead-after.png), [before JSON](performance/shellwick-overhead-before.json), [after JSON](performance/shellwick-overhead-after.json)
- [Replay settings](REPLAY.md)
