# EternalBliss automation checkpoint

The working branch `automation/eternalbliss-release-candidate` was created from the remote automation head `2346b1f28d5baf6202685b3905bcf998dee97635`. It contains Claude’s baseline `76664bfa954062cf15de7b63d21edb26d44e7847` and the two mandate commits. Baseline HTML SHA-256 is `4c08d431fdd3119e96125b4fd15c86d95f1bca6ae4094455e88ee3a9edf86c55`. The working branch is a development checkpoint.

## Implementation

- `7953fcdf4d989cfcd37f972ff6b1dcbe9a020784` accepts valid satellite settlement IDs when reading v1 checkpoints. Existing major stations still work and invalid IDs fall back to Wayfarer’s Rest. The Atlas and transport board explain the separate station sailing scene and distinguish it from free sailing on terrain.
- `07a5693890807fc22f88ede0e711f7219fff9820` adds authored quest text for Wayfarer’s Rest, Hearthfield and Shellwick. The existing three mechanics, objectives, NPC identities, saved flags and payouts are retained. An overlapping site only counts as a road survey visit when the world identifies it as the actual target. Quest lookups build plans only for active quests.
- `e45702bcb7e97c047f3fd8674148ff844db082d2` protects unknown nested Farlands save versions before any player state is applied. Loading uses the existing blocked-save status and autosave guard. It also corrects the transport board note for ports with no carriage service.
- The last gameplay commit is `e45702bcb7e97c047f3fd8674148ff844db082d2`. Its HTML SHA-256 is `d8f7cd113d0681888ba9dead81673d20f49751b698318375ff93bff46672c35d`. All 32 inline scripts parse. The local save, solo Farlands and local quest scripts differ from the baseline. The other inline scripts, program and deployment files match the automation base.

The commit-pinned GitHub raw download matched the latest gameplay hash. The downloaded standalone HTML can be opened locally. The third-party rendering URL returned HTTP 403 from the test environment, so no hosted playable preview was verified.

## Terrain observations

The audit calls the procedural sampler used by gameplay terrain, including road and settlement overrides. Shellwick is 215.4 tiles from water, Pearl Banks 218.8, Lantern Quay 269.7 and Reedhaven 92.6. Amberport has no water within 450 tiles. Of 28 port, waterfront and priority sites sampled, 23 lack water inside their authored site radius. This radius is a screening threshold. Land access, usable hull clearance and connectivity to a wider water body need their own checks.

The fixed routes in `routeFor` start a self-contained sailing scene and arrive at `pointAt` when the player docks. The chart overlays are illustrative. Free sailing uses the actual tile map and a separate boat inventory or mooring path. Fixing the coast requires preserving both behaviors and testing their distinct entry points.

## Shared-world source audit

Solo Farlands has procedural terrain, sparse local edits, saved quests and discoveries, horses, boats and local resident schedules. The guest and owner-specific adventure saves store those locally. MagicBlock uses the Eternal Kingdom map and snapshot state. The client applies confirmed player positions and rooms, character progress and inventory, enemy HP and cooldowns, item availability, and opened terrain from world snapshots. Nearby player presentation is scoped by shared room and recent presence. Remote smoothing, overlap offsets and patrol poses are derived for presentation. Shared movement predicts pending input locally, then reconciles against the confirmed sequence. The code uses the shared clock for time-dependent state. This review is source evidence. Physical Android and concurrent shared-world sessions were outside the observed coverage.

## Evidence and limits

- [Automation evidence](tests/automation-evidence/) contains terrain reports, quest contracts and the browser ledger. Historical PR #24 results in the prior ledger refer to other HTML hashes.
- The v1 fixture covers the checkpoint ID, inside and outside hero coordinates, horse location, a terrain edit, discovery and quest flags. The browser fixture separately exercises old-version and unsupported-version records through the outer save/load path. There is no new save version in this checkpoint.
- The v1 browser round trip restored a Shellwick inn interior and exterior coordinates, its settlement checkpoint, a horse, taken and parked boat coordinates and heading, a terrain edit, the Atlas pin, active/completed quest flags and 450 gold. The unknown-version browser fixture kept the stored record byte-for-byte after accepted keyboard input and a flush attempt. The first v1 fixture paired a Shellwick station with a Wayfarer’s Rest position, where the location update correctly set the station to Wayfarer’s Rest. The corrected fixture uses a consistent Shellwick position.
- The quest contract checks targets, givers and payouts across 162 existing plans, plus the Shellwick/Pearl Banks overlap and active/completed flags. The Node run took 491 ms before and 66 ms after for 1,000 plan lookups with one active quest. This is a microbenchmark of that routine, with no claim about frame rate.
- Desktop Chromium passed baseline and latest-build startup, overhead/first person switching and 2D. The earlier integrated HTML `d5341100…` passed desktop FP combat, including Attack, Magic, Heal and enemy replies, and portrait FP/joystick controls. The latest HTML passed the desktop Farlands/Eternal Kingdom switch and local save/reload checks, including the stubbed active-shared local-save guard.
- Atlas and passage entry passed on desktop, portrait and landscape. The desktop and portrait captures use the prior HTML `d5341100…`, which has identical Atlas code. The landscape capture uses the latest HTML. The check opens and exits the passage scene. It does not sail the entire passage.
- A fixed-noon, crowded-well fixture used Space to report the Harvest Bell to Mira. It awarded 65 gold and 85 XP exactly once and retained the reward on reload. The fixture starts with the relic flag already set.
- In the first Atlas harness pass, landscape timed out while opening Actions. Phones in landscape intentionally start with the location panel minimized. The harness now opens that panel before opening Actions. The timeout is retained in the logs.

## Failed transport check

The desktop vehicle suite passed the horse purchase, mount, first-person movement, dismount and reload assertions before stopping at the boat fixture. The original fixture required water beside walkable land. The first repair searched for a visible boat within boarding range and a connected water path to a legal landing. That reached the Use action but did not open Embark. The second repair aimed the actual rendered camera at the boat hull and failed the ray intersection check. The failed harness changes were removed, and the three logs are in the evidence directory. The result leaves real first-person boarding and free-sailing acceptance open. The next diagnostic should record the boat mesh bounds and transforms, the camera ray, higher-priority actor hits and the player's actual distance at Use before deciding whether to fix fixture setup or picking behavior.

## Remaining work in priority order

1. Design a connected coastline for Lantern Quay, Shellwick and Pearl Banks using actual terrain contours. Map walkable approaches, route intersections, docks and boat clearances before editing. Treat Amberport separately. The priority sites are far enough inland that a small local depression would create an isolated pond.
2. Pair any geometry change with a reversible save version. Preserve the exact old save and test old land and water coordinates, mounted and parked horses, moorings, occupied boats, edits, interior exits, pins, visited sites and quest flags. Check coastline differences against existing building footprints and generated actor or item positions. Maintain the prior terrain for recovery until the migration and rollback both pass.
3. Improve actual settlement layouts and resident activity, then test schedules, collision, interaction and trade. Extend authored quest mechanics and exploration rewards through the normal step, dialogue and reward interfaces. Verify the Harvest Bell chain and first 15 minutes from a fresh guest slot.
4. Verify portrait and landscape D-pad, touch movement, tap-to-move, minimap, Atlas, camera and first-person controls. Complete combat, transport and save/reload checks and measure rendered frame timings on available profiles. Record emulation and device evidence separately.
5. Prepare a portable final regression run once the integrated work is frozen. The older PR #24 driver includes private scratch paths and source expectations specific to that PR. Provide the draft PR with the final hash, matching visual evidence, remaining device and shared-world limits, and rollback instructions.

## Resume

```bash
git fetch origin automation/eternalbliss-codex automation/eternalbliss-release-candidate
git switch automation/eternalbliss-release-candidate
git show 76664bfa954062cf15de7b63d21edb26d44e7847:eternalbliss/index.html > /tmp/eb-baseline.html
sha256sum /tmp/eb-baseline.html eternalbliss/index.html
node eternalbliss/tests/automation-audit.cjs
node eternalbliss/tests/automation-audit.cjs eternalbliss/index.html --require-waterfront
node eternalbliss/tests/automation-quests.cjs /tmp/eb-baseline.html
node eternalbliss/tests/automation-save-version.cjs
EB_VEHICLE_CASE=desktop node eternalbliss/tests/release/vehicles.cjs
```

The strict waterfront command is expected to fail on this checkpoint. For the last two commands, use a guest profile with system Chromium, Playwright available through `NODE_PATH` and a writable `EB_EVIDENCE` directory. Run the scripts sequentially. If smaller screens start with the panel minimized, use the Show panel control to reveal Actions. Start with failed or missing profiles only. Set `EB_SOURCE=/tmp/eb-baseline.html EB_AUTOMATION_BASELINE=1` for baseline Atlas runs and `EB_AUTOMATION_PROFILE=landscape` to select only that profile. The checked-in helper intercepts the initial HTML at `http://127.0.0.1:8091/eternalbliss/`. Suites that open a raw page themselves require a static server at that address. Avoid the production origin and use fresh test slots.

For application rollback before a geometry migration, use the pinned baseline commit or revert the two implementation commits on a new child branch. The application still serializes the existing v1 save.
