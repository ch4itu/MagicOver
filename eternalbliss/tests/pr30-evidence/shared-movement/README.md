# Shared World Phase 2 acceptance

Gameplay commit `1c5e07573b697d4c3948218ba080aa3965645f47`. HTML SHA-256 `e4f81a1263d70aa469291e3c01df3dccca35bd1fd18d1e6c1930b113b1128db5`.

First-person mode uses existing camera-relative input, look controls and sight-target selection. Shared diagonal steering alternates cardinal axes at the walking cadence. A blocked primary axis can yield to the other axis. The deployed movement instruction still validates every cardinal tile, room tile, obstacle and rate limit. Shared movement has no mounted state in its account or instruction set. Solo horses and true diagonal tiles are unchanged.

Remote 3D rendering follows the existing confirmed-position interpolation without a second smoothing pass. Gait advances from distance drawn. The underlying account, protocol, payload sizes and shared authority are unchanged. Seats, rooms, positions, encounter health, loot and stock remain shared. Farlands quests, relationships and solo saves remain local.

The two-context browser test substitutes sign-in and transport with a bridge to a local LiteSVM instance running the checked-in compiled program, with distinct signers and browser storage. It uses real keyboard input for camera-relative movement and diagonal steering. It also checks peer positions, rendered pose, room membership, transport-cut recovery, failed leave, successful leave and rejoin. It does not exercise public sign-in or public network transactions.

Frame intervals use 32 samples per view after a warm-up, with two software-rendered Chromium contexts and a fixed shared account state. The 2D pass disables WebGL in both clients. Local movement delivery measures from bridge request to peer ingestion separately. These intervals do not measure hardware or public-network performance.

Public MagicBlock HTTP reads and two WebSocket subscribe acknowledgments are recorded separately. The workspace had no authorized registered Smaran test accounts, and public signup would create persistent shared registry records. No public transactions were submitted. The compiled-program checks use the checked-in binary, with the optional delegation CPI case skipped because the official delegation binary was unavailable.
