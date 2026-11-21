# EternalBliss automation checkpoint

The working branch `automation/eternalbliss-release-candidate` was created from the remote automation head `2346b1f28d5baf6202685b3905bcf998dee97635`. It contains Claude’s baseline `76664bfa954062cf15de7b63d21edb26d44e7847` and the two mandate commits. Baseline HTML SHA-256 is `4c08d431fdd3119e96125b4fd15c86d95f1bca6ae4094455e88ee3a9edf86c55`. The working branch is a development checkpoint.

## Coastal continuation

The latest gameplay commit is `75cb6f0c399a6dac1df96b3455a3afcbcd835737`. Its HTML SHA-256 is `f3d1b298955756bab42eaa340bab2f44cc3e01c1dcd4e03c8fb90b720933a5e8`. The older evidence below remains tied to the previous checkpoint.

- `7b96892` adds a south-facing bay in the Lantern Keys and a separate east-facing inlet at Amberport. Lantern Quay and Shellwick have moorings and jetties. Pearl Banks shares the Shellwick landing. The existing station and landmark coordinates are unchanged. The automated geography test checks existing road centerlines, dry settlement cores, the exact rendered-hull clearance calculation and a connected five-tile fairway to open sea.
- `49ffab4` keeps the station boats' model proportions consistent with terrain clearance and moves Use clear of the landscape minimap. `75cb6f0` also separates Use from the Look controls in short touch layouts and keeps the direct v1 restore caller compatible.
- `1848171` adds Shellwick's **Sound the Lantern Channel**. It requires the player to visit the Shellwick mooring and then Lantern Quay's mooring while afloat, and report back to the actual waykeeper. It tracks a separate set of flags and pays 24 gold and 25 XP. The old 162 quest plans retain their original givers, objectives and rewards. The two shore markers reuse existing 2D and 3D world props.
- The nested Farlands save format advances from v1 to v2. Loading v1 archives the complete original save at `<slot>/before-farlands-v2` before applying state or enabling autosave. Dry hero and horse tiles newly under water move to nearby dry ground. Existing edits take precedence. Saved interior coordinates, afloat positions, the horse aboard, boat records, pins, discoveries and quest flags carry over. A failed backup write blocks autosave through the existing save error path.
- To roll back, close other game tabs and copy the current v2 save somewhere safe. Restore the raw archived value to its original slot under the same browser origin and load gameplay commit `e45702bcb7e97c047f3fd8674148ff844db082d2`. This returns to the pre-migration state. Progress earned after migration stays in the separately retained v2 save. Browser fixtures passed migration and rollback through the prior HTML.

[Coastal evidence](tests/coastal-evidence/) includes maps sampled from the old and new terrain, rendered screenshots with the obstructed and corrected landscape HUD at the same jetty, and exact source hashes in the browser ledger. Chromium tested all three real jetty approaches and desktop/portrait/landscape first-person Use, a water step and disembark. The extra landscape retest also hit-tested both Look controls and changed the camera with a real tap. The Shellwick quest browser fixture accepted the board quest, passed 95 normal game movement checks from the Shellwick mooring to the Quay, restored the result after reload and awarded the quest payoff once through the waykeeper's dialogue. The fixture begins the sailing leg already aboard and positions the player for the report. Boarding is covered independently by the transport fixture.

Rendered frame timings are recorded for 30 consecutive intervals after boarding. Desktop and portrait on the integrated predecessor recorded medians of 28 and 27 ms and p95 intervals of 824 and 477 ms. The final landscape HUD retest recorded 24 ms median and 994 ms p95. These are browser software-renderer observations, with significant stalls. They do not establish a physical phone frame-rate budget.

Use `node eternalbliss/tests/coastal-geography.cjs` for the geography and relocation gates. To rerun the browser fixtures, export `NODE_PATH=/workspace/.magic-over-setup/browser/node_modules`, set `EB_EVIDENCE` to a new writable directory, and run sequentially.

```bash
git show 047b3c91d0c433c0615d4b319367be8566a96d5d:eternalbliss/index.html > /tmp/eb-before-coast.html
EB_ROLLBACK_SOURCE=/tmp/eb-before-coast.html node eternalbliss/tests/automation-save-version.cjs
node eternalbliss/tests/coastal-boarding.cjs
node eternalbliss/tests/coastal-quest.cjs
EB_CONTROL_CASE=landscape node eternalbliss/tests/controls.cjs
EB_DPAD_CASE=tap node eternalbliss/tests/tablet-dpad.cjs
EB_COMBAT_CASE=landscape:false node eternalbliss/tests/combat.cjs
```

The original Wayfarer's Rest generated-mooring test remains a separate regression from the new authored-jetty path. It previously failed in fixture selection and aim intersection. The new test positions a hero at a real Lantern Quay jetty, inspects the rendered hull and camera ray, then activates the actual Use button and confirms the boarding dialog. The older full-suite driver still needs portability work, including its private scratch paths. Remaining release gates also include broader resident schedules and early progression, natural first-15-minute gameplay, expanded coastal validation beyond these priority sites, physical device performance and the integrated regression campaign.

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
