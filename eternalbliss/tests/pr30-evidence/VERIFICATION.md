# PR 30 verification

The integrated matrix started on gameplay commit `6afa95ca2a709945f2c8c7d5a250bf3e3b0b3ab3` with HTML SHA-256 `a9c2af01a5bfeb48f45430ce440eafecc696c518884bd2dca56e88b1e75d327e`. Fourteen jobs passed on that HTML, including geography, save migration and rollback, the old quest contract, the new fresh guest tally, both mobile combat profiles, all three generated boat profiles, authored coastal transport, Shellwick’s quest, startup and the shared-active save guard. The shared-world verification combines source isolation and a stubbed active-world save guard. It does not involve live chain sessions.

Two jobs remained open on that hash. Chromium sometimes delivered a valid touch pointer release after a swipe without a compatibility click. The natural Harvest run reached the archive exit within its original four-minute budget, then the runner timed out. Its first resume used the overhead camera inside the archive, where the exit fixture required first person. The fixture now re-enters first person as needed.

The final HTML is gameplay commit `1d163690212dcc2057b5105d89a94cef08b782a2` with SHA-256 `c1b17de53b6b0683856fd8e57655f5d63df9f96ba8ca5e84035f76301e02c0bc`. Only the navigation and 3D UI scripts differ from the integrated predecessor. Focused tests verified stationary pointer release in portrait and landscape, a rejected swipe, the D-pad and Atlas, landscape Look and first-person Use with Lantern Quay sailing and docking, and 3D, first-person and 2D startup.

The fresh Harvest Bell route passed across both hashes. The final run loaded the real guest save recorded at bell recovery, exited in first person, delivered the bell to Mira, checked that speaking again did not duplicate the reward and reloaded with the reward intact. The first leg walked from the default spawn, obtained the key and defeated the Warden using normal input. The recorded clock was fixed at noon. A continuous first-fifteen-minute play session and broader work on schedules and density remain open.

The save rollback retry used the correct pre-coast v1 HTML. It passed the v1 round trip, unknown-version preservation and rollback. Restoring the pre-coast snapshot remains separate from reverting this continuation to an earlier v2 build.

The matrix and focused retests cover desktop Chromium and emulated phone profiles. SwiftShader still produces long frame intervals. The complete final-hash matrix and physical-device acceptance are open.

- [Predecessor matrix record](predecessor-matrix/pr30-matrix.json)
- [Final touch, startup and Harvest resume logs](final-touch/)
- [Cumulative test ledger](ledger.jsonl)
- [Rendering comparison](RENDERING.md)
