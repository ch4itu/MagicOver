"""The shared world's capacity on a simulated ledger: 32 signed players join and walk; the 33rd new character is refused
(30) while the 32 are there; after one of them leaves and frees an unused slot (28), the next player gets in. No network."""
import struct
import sys

import check_world as w
from solders.keypair import Keypair
from solders.transaction_metadata import TransactionMetadata

w.tick(1000)
w.tx(w.operator, [w.meta(w.operator.pubkey(), True, True), w.meta(w.WORLD, writable=True), w.meta(w.SYSTEM)],
     struct.pack("<BBQ", 16, 0, w.svm.minimum_balance_for_rent_exemption(w.SIZE)))
players = []
for i in range(33):
    key = Keypair()
    assert isinstance(w.svm.airdrop(key.pubkey(), 100_000_000), TransactionMetadata)
    players.append((key, f"cap-player-{i:02d}"))
for key, name in players[:32]:
    w.action(key, 18, name.encode(), shard=w.registered(name, key))
w.check("32 players have characters in the world", all(w.player(k) is not None for k, _ in players[:32]))
moved = 0
for second, (key, _) in enumerate(players[:32]):
    w.tick(1100 + second)
    before = w.u32(w.player(key), 84)
    w.action(key, 25)  # a heartbeat: every one of the 32 can still act
    moved += w.u32(w.player(key), 84) == before + 1
w.check("each of the 32 can act", moved == 32)
extra, extra_name = players[32]
shard = w.registered(extra_name, extra)
result = w.action(extra, 18, extra_name.encode(), shard=shard, ok=False)
w.check("the 33rd new character is refused while all 32 slots are taken", "Custom(30)" in str(result))
leaver, _ = players[0]
w.action(leaver, 24)
w.action(leaver, 28)
w.action(extra, 18, extra_name.encode(), shard=shard)
w.check("after one player leaves and frees an unused slot, the next player gets in",
        w.player(leaver) is None and w.player(extra) is not None)
