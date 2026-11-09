"""The shared world's capacity on a simulated ledger: 32 signed players join and walk; the 33rd new character is refused
(30) while the 32 are there; after one of them leaves and frees an unused slot (28), the next player gets in. Then a
second world takes the next new character. No network. Pass the delegation program file to also delegate world 1."""
import struct
import sys

DELEGATION_FILE = sys.argv[1] if len(sys.argv) > 1 else None
sys.argv = sys.argv[:1]  # check_world reads its own argument at import; this one is for world 1 below
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

# More worlds. World 1 is its own account at [b"eb-land", 1], created like world 0 and numbered in its header. A world
# number of 64 or more is refused. With world 0 full, the next new character joins world 1 instead; a character exists
# only in its own world, and a world account cannot stand in for another world's address.
from solders.pubkey import Pubkey

def world_key(number):
    return Pubkey.find_program_address([b"eb-land", bytes([number])], w.PROGRAM)[0]

def create(number, ok=True):
    keys = [w.meta(w.operator.pubkey(), True, True), w.meta(world_key(number), writable=True), w.meta(w.SYSTEM)]
    return w.tx(w.operator, keys, struct.pack("<BBQ", 16, number, w.svm.minimum_balance_for_rent_exemption(w.SIZE)), ok)

def act_in(number, key, op, args=b"", *, shard=None, seq=0, ok=True):
    keys = [w.meta(key.pubkey(), True, True), w.meta(world_key(number), writable=True)]
    if shard is not None:
        keys.append(w.meta(shard))
    return w.tx(key, keys, struct.pack("<BI", op, seq) + args, ok)

def slot_in(number, key):
    data = w.svm.get_account(world_key(number)).data
    for i in range(w.constant("PLAYERS")):
        at = w.H + i * w.P_SIZE
        if data[at:at + 32] == bytes(key.pubkey()):
            return data[at:at + w.P_SIZE]
    return None

create(1)
one = w.svm.get_account(world_key(1)).data
w.check("world 1 is created at its own address and keeps its number in its header", one[:8] == b"EBWRLD01" and one[60] == 1)
w.check("a world number of 64 or more is refused", "Custom" in str(create(64, ok=False)) and w.svm.get_account(world_key(64)) is None)
w.check("world 1 cannot be created twice", "Custom" in str(create(1, ok=False)) or "already in use" in str(create(1, ok=False)))
newcomer = Keypair()
assert isinstance(w.svm.airdrop(newcomer.pubkey(), 100_000_000), TransactionMetadata)
newcomer_shard = w.registered("cap-newcomer", newcomer)
w.check("with world 0 full, a new character is still refused there",
        "Custom(30)" in str(w.action(newcomer, 18, b"cap-newcomer", shard=newcomer_shard, ok=False)))
act_in(1, newcomer, 18, b"cap-newcomer", shard=newcomer_shard)
w.check("the new character joins world 1", slot_in(1, newcomer) is not None and w.player(newcomer) is None)
w.tick(2000)
seq = w.u32(slot_in(1, newcomer), 84)
act_in(1, newcomer, 25, seq=seq)
w.check("the world 1 character can act in world 1", w.u32(slot_in(1, newcomer), 84) == seq + 1)
w.check("it has no character in world 0, so it cannot act there (35)",
        "Custom(35)" in str(act_in(0, newcomer, 25, seq=seq + 1, ok=False)))
w.check("world 0's 32 characters are untouched", sum(w.player(k) is not None for k, _ in players[1:]) == 32)
if DELEGATION_FILE:
    from pathlib import Path
    w.svm.add_program_from_file(w.DELEGATION, Path(DELEGATION_FILE).resolve())
    key = world_key(1)
    buffer = Pubkey.find_program_address([b"buffer", bytes(key)], w.PROGRAM)[0]
    record = Pubkey.find_program_address([b"delegation", bytes(key)], w.DELEGATION)[0]
    metadata = Pubkey.find_program_address([b"delegation-metadata", bytes(key)], w.DELEGATION)[0]
    w.tx(w.operator, [w.meta(w.operator.pubkey(), True, True), w.meta(key, writable=True), w.meta(w.PROGRAM),
                      w.meta(buffer, writable=True), w.meta(record, writable=True), w.meta(metadata, writable=True),
                      w.meta(w.DELEGATION), w.meta(w.SYSTEM)], bytes([17, 1]))
    w.check("the MagicBlock delegation program accepts world 1", w.svm.get_account(key).owner == w.DELEGATION)
