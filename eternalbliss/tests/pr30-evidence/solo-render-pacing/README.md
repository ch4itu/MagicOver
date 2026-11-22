# Solo Farlands overhead software-renderer pacing

Gameplay commit `4785b782741ccedb281297deca92ec0b8544bd51`, HTML SHA-256 `9cbb061a692f9a8e9ae520d56204988ce9e672ba38a4f696d855bfc57f8cf190`. The single HTML change extends the existing one-frame-in-flight WebGL fence for SwiftShader to solo Farlands overhead view. Visible terrain, materials, shadows, camera, input cadence and accepted tiles use the existing code. The shared-world path and first-person pacing match version 13.

The instrumented profile held movement across the same open corridor, using the repository's existing scene and held-input fixture. The baseline reproduced long gaps in both overhead and first-person frames. Removing WebGL submission temporarily brought first-person frame callback p95 to 21 ms and accepted input gap p95 to 124 ms. Also omitting minimap drawing changed those to 20 ms and 125 ms. Actor synchronization CPU was about 1 ms p95. This isolates the largest measured stalls to software frame delivery in this environment. Individual streamed terrain patches also had occasional CPU spikes.

| Overhead comparison | Previous submission interval p95 | Paced submission interval p95 |
| --- | ---: | ---: |
| Run 1 | 3,845 ms | 909 ms |
| Run 2 | 2,189 ms | 712 ms |

Counts were 14 and 16 intervals before, 17 and 18 after. Timing measures time between WebGL submissions from Chromium. It does not measure GPU presentation or provide a statistically stable hardware estimate. Accepted movement timing remained variable under SwiftShader. The change reduces the worst observed overhead stalls in this setup. First-person and hardware-accelerated movement should be judged separately.

[Measured profiles](profile-summary.json) and the four paired sample files record the exact HTML hashes, retained terrain and per-frame data.

On the exact paced HTML, all 18 release checks and four Clean View profiles passed. A controlled long frame preserved all six accepted steps, and the first rendered advance stayed within the configured 0.25-second catch-up bound. Held diagonal touch in landscape and mounted first-person travel in emulated portrait passed with no missing terrain. The independent public MagicBlock trial used the preceding gameplay hash; the pacing change leaves shared mode and its protocol unchanged.
