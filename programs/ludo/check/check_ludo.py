"""Ludo Freeflow's program on a simulated ledger: tables, seats, independent rolls, VRF callbacks, moves, replays and
races. No network.

MagicBlock's VRF program only exists on the rollup, so a stand-in that accepts every call and logs what it was given
is loaded at its address (EternalBliss's multiplayer/stub). The VRF callback is then sent by the test itself, as the
VRF program's identity PDA: signature checks are switched off for those transactions only (a PDA has no key), and
switched back on for everything else. Tables are not delegated here; the program does not depend on it.
Fees exist on this simulator (not on the rollup), so players are funded here; nothing else differs.
usage: python3 -I check_ludo.py <libludo_freeflow.so> <magic stand-in .so> <operator keypair json>
"""
import json
import re
import struct
import sys
from pathlib import Path

from solders.clock import Clock
from solders.instruction import AccountMeta, Instruction
from solders.keypair import Keypair
from solders.litesvm import LiteSVM
from solders.message import Message
from solders.pubkey import Pubkey
from solders.signature import Signature
from solders.transaction import Transaction
from solders.transaction_metadata import FailedTransactionMetadata, TransactionMetadata

args = sys.argv[1:]
PROGRAM = Keypair().pubkey()
OPERATOR = Pubkey.from_string("HDbeKypeJHXMM5Xs2RHKj8XGQgJeP4jnQ1Jiugf8mXPn")
VRF = Pubkey.from_string("Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz")
QUEUE = Pubkey.from_string("5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc")
SLOT_HASHES = Pubkey.from_string("SysvarS1otHashes111111111111111111111111111")
MAGIC = Pubkey.from_string("Magic11111111111111111111111111111111111111")
CONTEXT = Pubkey.from_string("MagicContext1111111111111111111111111111111")
SYSTEM = Pubkey.default()
IDENTITY = Pubkey.find_program_address([b"identity"], PROGRAM)[0]
VRF_IDENTITY = Pubkey.find_program_address([b"identity", bytes(PROGRAM)], VRF)[0]
CALLBACK = b"LFVRFCB1"
TABLE_SIZE, T_SEATS, SEAT, T_RING, RING = 512, 40, 104, 456, 7
BASE, HOME = 255, 56

svm = LiteSVM()
svm.add_program_from_file(PROGRAM, Path(args[0]).resolve())
svm.add_program_from_file(VRF, Path(args[1]).resolve())
operator = Keypair.from_bytes(bytes(json.loads(Path(args[2]).read_text())))
assert operator.pubkey() == OPERATOR, "the operator key built into the program"
A, B, C, D, E, X = (Keypair() for _ in range(6))
SA, SB, SC, SD, SE, S2 = (Keypair() for _ in range(6))  # session keys the game page would make
SESSION = {A: SA, B: SB, C: SC, D: SD, E: SE}
for who in (operator, A, B, C, D, E, X, SA, SB, SC, SD, SE, S2):
    assert isinstance(svm.airdrop(who.pubkey(), 10_000_000_000), TransactionMetadata)
passed = 0
NOW = 1_800_000_000


def clock(t):
    global NOW
    NOW = t
    svm.set_clock(Clock(1000 + t, 0, 0, 0, t))


clock(NOW)


def ok(label, condition=True):
    global passed
    assert condition, label
    passed += 1
    print("  ok  ", label)


def m(key, signer=False, writable=False):
    return AccountMeta(key, signer, writable)


def table_key(i):
    return Pubkey.find_program_address([b"lf-table", bytes([i])], PROGRAM)[0]


def code(result):
    found = re.search(r"InstructionError\(\d+, Custom\((\d+)\)\)", str(result))
    return int(found[1]) if found else str(result)[:300]


def send(signer, keys, data, error=None, watch=(), payer=None, unsigned=None):
    """One instruction signed by `signer` (fee `payer` if given). `unsigned`: a signer without a key (a PDA), sent
    with signature checks off. With `error`, it must fail with that code and change none of `watch`."""
    svm.expire_blockhash()
    before = [bytes(svm.get_account(k).data) if svm.get_account(k) else None for k in watch]
    ix = Instruction(PROGRAM, bytes(data), keys)
    if unsigned is not None:
        svm.with_sigverify(False)
        msg = Message.new_with_blockhash([ix], payer.pubkey(), svm.latest_blockhash())
        wire = Transaction.populate(msg, [Signature.default()] * msg.header.num_required_signatures)
    else:
        fee = payer or signer
        signers = [fee] if fee == signer else [fee, signer]
        wire = Transaction.new_signed_with_payer([ix], fee.pubkey(), signers, svm.latest_blockhash())
    result = svm.send_transaction(wire)
    if unsigned is not None:
        svm.with_sigverify(True)
    if error is None:
        assert isinstance(result, TransactionMetadata), code(result)
    else:
        assert isinstance(result, FailedTransactionMetadata), f"expected error {error}, it succeeded"
        assert code(result) == error, (code(result), error)
        assert [bytes(svm.get_account(k).data) if svm.get_account(k) else None for k in watch] == before, "a refused instruction changed state"
    return result


def logs(result):
    meta = result if isinstance(result, TransactionMetadata) else result.meta()
    return meta.logs() if callable(meta.logs) else meta.logs


def tbl(i=0):
    return bytes(svm.get_account(table_key(i)).data)


def seat(t, s):
    o = T_SEATS + s * SEAT
    return dict(key=Pubkey(t[o:o + 32]), session=Pubkey(t[o + 32:o + 64]), name=t[o + 65:o + 65 + t[o + 64]].decode(), pieces=list(t[o + 80:o + 84]),
                wait=t[o + 84], pending=t[o + 85], sixes=t[o + 86], bonus=t[o + 87], last=t[o + 88], captures=t[o + 89], flags=t[o + 90], home=t[o + 91],
                seq=struct.unpack_from("<I", t, o + 92)[0], req=struct.unpack_from("<I", t, o + 96)[0], req_at=struct.unpack_from("<I", t, o + 100)[0])


def head(t):
    return dict(phase=t[9], seated=t[10], winner=t[11], match=struct.unpack_from("<I", t, 12)[0], events=struct.unpack_from("<I", t, 32)[0])


def last_events(t, n):
    count = struct.unpack_from("<I", t, 32)[0]
    return [tuple(t[T_RING + (k % RING) * 8:T_RING + (k % RING) * 8 + 8]) for k in range(max(0, count - n), count)]


def poke(i, edits):
    """Test setup only: writes bytes straight into the table account (the simulator's set_account)."""
    acc = svm.get_account(table_key(i))
    data = bytearray(acc.data)
    for off, val in edits:
        data[off:off + len(val)] = bytes(val)
    from solders.account import Account
    svm.set_account(table_key(i), Account(acc.lamports, bytes(data), acc.owner, acc.executable, acc.rent_epoch))


u32 = lambda n: struct.pack("<I", n)
name = lambda s: bytes([len(s)]) + s.encode()
create = lambda who, i, nm, error=None: send(who, player(who, i), bytes([2, i]) + bytes(SESSION[who].pubkey()) + name(nm), error, watch=[table_key(i)] if error else ())
join = lambda who, i, match, nm, error=None: send(who, player(who, i), bytes([3, i]) + u32(match) + bytes(SESSION[who].pubkey()) + name(nm), error, watch=[table_key(i)] if error else ())
player = lambda who, i=0: [m(who.pubkey(), True, True), m(table_key(i), False, True)]
roll_keys = lambda who, i=0: [m(who.pubkey(), True, True), m(table_key(i), False, True), m(IDENTITY), m(VRF), m(QUEUE, False, True), m(SYSTEM), m(SLOT_HASHES)]


def roll(who, i, match, s, seq, error=None):
    return send(who, roll_keys(who, i), bytes([5, i]) + u32(match) + bytes([s]) + u32(seq), error, watch=[table_key(i)] if error else ())


def move(who, i, match, s, seq, token, frm, error=None):
    return send(who, player(who, i), bytes([6, i]) + u32(match) + bytes([s]) + u32(seq) + bytes([token, frm]), error, watch=[table_key(i)] if error else ())


def callback(i, match, s, seq, rnd, error=None, signer=None, size=None):
    keys = [m(signer.pubkey() if signer else VRF_IDENTITY, True, False), m(table_key(i), False, True)]
    data = CALLBACK + rnd + bytes([i]) + u32(match) + bytes([s]) + u32(seq)
    if size is not None:
        data = data[:size]
    if signer is not None:
        return send(signer, keys, data, error, watch=[table_key(i)] if error else ())
    return send(None, keys, data, error, watch=[table_key(i)] if error else (), payer=X, unsigned=True)


def die_bytes(d):
    return bytes([d - 1]) + bytes(31)  # first byte below 252 -> (d - 1) % 6 + 1 = d


print("tables")
lam = svm.minimum_balance_for_rent_exemption(TABLE_SIZE)
init = lambda who, i: send(who, [m(who.pubkey(), True, True), m(table_key(i), False, True), m(SYSTEM)], bytes([0, i]) + struct.pack("<Q", lam), None if who == operator else 22)
init(A, 0)
ok("InitTable from anyone but the operator: refused (22)")
for i in (0, 1, 2):
    init(operator, i)
t = tbl(0)
ok("InitTable: 512-byte table, magic LUDOFF01, free, no winner, every token at base", len(t) == 512 and t[:8] == b"LUDOFF01" and head(t)["phase"] == 0 and head(t)["winner"] == 255 and all(seat(t, s)["pieces"] == [BASE] * 4 for s in range(4)))
send(operator, [m(operator.pubkey(), True, True), m(table_key(0), False, True), m(SYSTEM)], bytes([0, 8]) + struct.pack("<Q", lam), 12)
ok("InitTable for index 8 (eight tables, 0-7): refused (12)")
send(operator, [m(operator.pubkey(), True, True), m(table_key(0), False, True), m(SYSTEM)], bytes([0, 3]) + struct.pack("<Q", lam), 9)
ok("InitTable for table 3 with table 0's address: refused (9)")
send(A, [m(A.pubkey(), True, True), m(table_key(0), False, True), m(MAGIC), m(CONTEXT, False, True)], bytes([8, 0]), 22)
ok("CommitUndelegate from anyone but the operator: refused (22)")

print("lobby")
create(A, 0, "A" * 16, 4)
ok("Create with a 16-byte name: refused (4)")
create(A, 0, "Ann")
t = tbl()
ok("Create: lobby, match 1, red seat is Ann's Smaran key with her session key", head(t)["phase"] == 1 and head(t)["match"] == 1 and seat(t, 0)["key"] == A.pubkey() and seat(t, 0)["session"] == SA.pubkey() and seat(t, 0)["name"] == "Ann" and head(t)["seated"] == 1)
create(B, 0, "Bo", 21)
ok("Create on a table with a live lobby: refused (21)")
join(A, 0, 1, "Ann", 18)
ok("Join by a key that already has a seat (same player, second device): refused (18)")
join(B, 0, 2, "Bo", 14)
ok("Join naming another match: refused (14)")
send(C, player(C), bytes([4, 0]) + u32(1), 6, watch=[table_key(0)])
ok("Start by someone without a seat: refused (6)")
send(A, player(A), bytes([4, 0]) + u32(1), 20, watch=[table_key(0)])
ok("Start with one player: refused (20)")
join(B, 0, 1, "Bo")
send(SB, player(SB), bytes([4, 0]) + u32(1))
t = tbl()
ok("Join, then Start signed by the second player's session key: blue seat is Bo's, playing", seat(t, 1)["key"] == B.pubkey() and head(t)["phase"] == 2 and head(t)["seated"] == 3)
join(C, 0, 1, "Cy", 5)
ok("Join after the start: refused (5)")

print("rolls (independent per seat)")
roll(A, 0, 1, 1, 0, 6)
ok("Roll for someone else's seat: refused (6)")
roll(A, 0, 1, 0, 1, 7)
ok("Roll with an action number ahead of the seat's: refused (7)")
roll(A, 0, 2, 0, 0, 14)
ok("Roll naming another match: refused (14)")
roll(X, 0, 1, 0, 0, 6)
ok("Roll signed by a key that is neither the seat's Smaran key nor its session key: refused (6)")
r = roll(SA, 0, 1, 0, 0)
calls = [line for line in logs(r) if "0xda" in line]
s = seat(tbl(), 0)
ok("Roll signed by red's session key asks the VRF program (stand-in saw one RequestRandomness, data starts 10) and marks the seat pending", len(calls) == 1 and ", 0xa, " in calls[0] and s["pending"] == 1 and s["seq"] == 1 and s["req"] == 0)
roll(A, 0, 1, 0, 1, 8)
ok("A second roll while the first is outstanding: refused (8)")
roll(B, 0, 1, 1, 0)
ok("Another seat rolls while red's roll is outstanding (pending rolls never block other seats)", seat(tbl(), 1)["pending"] == 1)

print("VRF callbacks")
callback(0, 1, 0, 0, die_bytes(6), 13, signer=X)
ok("Callback signed by an ordinary key: refused (13)")
callback(0, 1, 0, 0, die_bytes(6), 4, size=49)
ok("Callback with a short payload: refused (4)")
callback(0, 1, 0, 5, die_bytes(6), 23)
ok("Callback for a different action number than the outstanding roll: refused (23)")
callback(0, 9, 0, 0, die_bytes(6), 14)
ok("Callback for another match: refused (14)")
callback(0, 1, 2, 0, die_bytes(6), 6)
ok("Callback for an empty seat: refused (6)")
callback(0, 1, 0, 0, die_bytes(6))
s = seat(tbl(), 0)
ok("Callback from the VRF identity for the outstanding roll: red's die is 6, waiting to be played", s["wait"] == 6 and s["last"] == 6 and s["pending"] == 0 and last_events(tbl(), 1)[0][:3] == (3, 0, 6))
callback(0, 1, 0, 0, die_bytes(3), 23)
ok("The same callback again (duplicate or replay): refused (23), the die stays 6")
callback(0, 1, 1, 0, die_bytes(4))
ok("Blue's callback: a 4 with every token at base has no move, so nothing waits", seat(tbl(), 1)["wait"] == 0 and seat(tbl(), 1)["last"] == 4 and last_events(tbl(), 1)[0][:3] == (8, 1, 4))

print("moves")
move(A, 0, 1, 0, 1, 0, 7, 16)
ok("Move naming a square the token is not on: refused (16)")
move(B, 0, 1, 1, 1, 0, BASE, 19)
ok("Move with no die waiting: refused (19)")
move(A, 0, 1, 0, 1, 4, BASE, 12)
ok("Move of token 5: refused (12)")
move(A, 0, 1, 0, 1, 255, 0, 15)
ok("Pass while a legal move exists: refused (15)")
move(A, 0, 1, 0, 1, 0, BASE)
s = seat(tbl(), 0)
ok("Move: a six enters red's token 1 on its start square; bonus shown; action number 2", s["pieces"] == [0, BASE, BASE, BASE] and s["wait"] == 0 and s["bonus"] == 1 and s["seq"] == 2)
move(A, 0, 1, 0, 1, 0, BASE, 7)
ok("The same move sent again (retry after a timeout, or a second device): refused (7)")

print("session keys")
rb = lambda who, i, match, s_, new, error=None: send(who, player(who, i), bytes([9, i]) + u32(match) + bytes([s_]) + bytes(new.pubkey()), error, watch=[table_key(i)] if error else ())
rb(SA, 0, 1, 0, S2, 24)
ok("Rebind signed by the session key itself: refused (24, only the Smaran key may)")
rb(B, 0, 1, 0, S2, 6)
ok("Rebind of red's seat signed by blue's Smaran key: refused (6)")
rb(A, 0, 1, 0, S2)
ok("Rebind signed by red's Smaran key: the seat now names the new session key (another device)", seat(tbl(), 0)["session"] == S2.pubkey())
roll(SA, 0, 1, 0, 2, 6)
ok("The replaced session key can no longer play (6)")
r = roll(S2, 0, 1, 0, 2)
callback(0, 1, 0, 2, die_bytes(5))
ok("The new session key plays at once (roll, callback applied: a 5 with one token out)", seat(tbl(), 0)["wait"] == 5)
move(S2, 0, 1, 0, 3, 0, 0)
ok("…and moves", seat(tbl(), 0)["pieces"][0] == 5 and seat(tbl(), 0)["seq"] == 4)

print("captures, blocks and races")
o0, o1 = T_SEATS, T_SEATS + SEAT
poke(0, [(o0 + 80, [3, BASE, BASE, BASE]), (o0 + 84, [2]), (o1 + 80, [(5 + 52 - 13) % 52, BASE, BASE, BASE])])
move(A, 0, 1, 0, 4, 0, 3)
t = tbl()
ok("Red lands on blue's single token off the safe squares: blue's token goes back to base, red has 1 capture and a bonus",
   seat(t, 1)["pieces"][0] == BASE and seat(t, 0)["captures"] == 1 and seat(t, 0)["bonus"] == 1 and (5, 0, 0, 5, 1, 0, 0, 0) in last_events(t, 3))
poke(0, [(o0 + 80, [7, BASE, BASE, BASE]), (o0 + 84, [5]), (o1 + 80, [(10 + 52 - 13) % 52, (10 + 52 - 13) % 52, BASE, BASE])])
move(A, 0, 1, 0, 5, 0, 7, 15)
ok("Passing blue's block of two: refused (15)")
poke(0, [(o0 + 84, [3])])
move(A, 0, 1, 0, 5, 0, 7, 15)
ok("Landing on blue's block of two: refused (15)")
poke(0, [(o0 + 80, [7, 30, BASE, BASE]), (o0 + 84, [5])])
move(A, 0, 1, 0, 5, 255, 0, 15)
ok("Pass while token 2 could still move: refused (15)")
poke(0, [(o0 + 80, [7, BASE, BASE, BASE]), (o0 + 84, [5])])
move(A, 0, 1, 0, 5, 255, 0)
ok("Pass when a block made the waiting die unplayable: accepted, nothing waits", seat(tbl(), 0)["wait"] == 0 and seat(tbl(), 0)["seq"] == 6)
# race: blue's move is sent for a token that red has just captured
poke(0, [(o0 + 80, [9, BASE, BASE, BASE]), (o0 + 84, [2]), (o1 + 80, [(11 + 52 - 13) % 52, BASE, BASE, BASE]), (o1 + 84, [6])])
blue_from = (11 + 52 - 13) % 52
move(A, 0, 1, 0, 6, 0, 9)
ok("Race, part 1: red captures blue's token", seat(tbl(), 1)["pieces"][0] == BASE)
move(B, 0, 1, 1, 1, 0, blue_from, 16)
ok("Race, part 2: blue's move of that token (sent before it saw the capture) is refused (16), not turned into an entry with its 6")
ok("…and blue's 6 is still waiting for another choice", seat(tbl(), 1)["wait"] == 6)

print("timeouts and retries")
poke(0, [(o0 + 84, [0])])
roll(A, 0, 1, 0, 7)
clock(NOW + 3)
roll(A, 0, 1, 0, 8, 8)
ok("Roll again 3 s after an unanswered roll: refused (8)")
clock(NOW + 6)
roll(A, 0, 1, 0, 8)
ok("Roll again 9 s after an unanswered roll: accepted as a new request (action 8)", seat(tbl(), 0)["req"] == 8 and seat(tbl(), 0)["seq"] == 9)
callback(0, 1, 0, 7, die_bytes(2), 23)
ok("The first request's late callback: refused (23)")
callback(0, 1, 0, 8, die_bytes(1))
ok("The replacement's callback: applied", seat(tbl(), 0)["last"] == 1 and seat(tbl(), 0)["pending"] == 0)

print("three sixes")
poke(0, [(o0 + 80, [20, BASE, BASE, BASE]), (o0 + 84, [0]), (o0 + 86, [0])])
seq = seat(tbl(), 0)["seq"]
for k in range(3):
    roll(A, 0, 1, 0, seq)
    callback(0, 1, 0, seq, die_bytes(6))
    seq += 1
    if k < 2:
        move(A, 0, 1, 0, seq, 0, seat(tbl(), 0)["pieces"][0])
        seq += 1
s = seat(tbl(), 0)
ok("A third six in a row is forfeited: no die waits, the count restarts", s["wait"] == 0 and s["sixes"] == 0 and s["last"] == 6 and s["pieces"][0] == 32 and last_events(tbl(), 1)[0][:3] == (7, 0, 6))

print("pairs on safe squares")
poke(0, [(o0 + 80, [6, BASE, BASE, BASE]), (o0 + 84, [4]), (o0 + 86, [0]), (o1 + 80, [(8 + 52 - 13) % 52, (8 + 52 - 13) % 52, BASE, BASE])])
move(A, 0, 1, 0, seq, 0, 6)
seq += 1
ok("Passing blue's pair on a star square (every colour shares those squares): accepted", seat(tbl(), 0)["pieces"][0] == 10)

print("finish, win and leave")
poke(0, [(o0 + 80, [HOME, HOME, HOME, HOME - 1]), (o0 + 84, [1]), (o0 + 91, [3])])
move(A, 0, 1, 0, seq, 3, HOME - 1)
t = tbl()
ok("Red's fourth token home: red wins, the match is finished", head(t)["phase"] == 3 and head(t)["winner"] == 0 and (9, 0, 0, 0, 0, 0, 0, 0) in last_events(t, 3))
roll(B, 0, 1, 1, seat(t, 1)["seq"], 5)
ok("Roll after the match ended: refused (5)")
callback(0, 1, 1, 0, die_bytes(3), 5)
ok("A callback arriving after the match ended: refused (5)")
create(C, 0, "Cy", 21)
ok("New match on a just-finished table (results stay visible 20 s): refused (21)")
clock(NOW + 21)
create(C, 0, "Cy")
t = tbl()
ok("21 s later: a new match (2) with Cy in red; the old match's seats are gone", head(t)["match"] == 2 and seat(t, 0)["key"] == C.pubkey() and seat(t, 1)["flags"] == 0 and all(seat(t, s)["pieces"] == [BASE] * 4 for s in range(4)))
for who, nm in ((D, "Di"), (E, "Ed"), (A, "Ann")):
    join(who, 0, 2, nm)
t = tbl()
ok("Four seated: the match starts by itself", head(t)["phase"] == 2 and head(t)["seated"] == 15)
join(B, 0, 2, "Bo", 5)
ok("A fifth player: refused (5)")
move(A, 0, 1, 3, seq + 1, 0, BASE, 14)
ok("An action naming the old match: refused (14)")
send(D, player(D), bytes([7, 0]) + u32(2) + bytes([1]))
send(E, player(E), bytes([7, 0]) + u32(2) + bytes([2]))
t = tbl()
ok("Two leave mid-match: their colours are out, the match goes on", head(t)["phase"] == 2 and seat(t, 1)["flags"] == 3 and seat(t, 2)["flags"] == 3)
roll(D, 0, 2, 1, 0, 6)
ok("A player who left cannot roll (6)")
send(A, player(A), bytes([7, 0]) + u32(2) + bytes([3]))
t = tbl()
ok("Down to one colour: it wins", head(t)["phase"] == 3 and head(t)["winner"] == 0)

print("abandoned lobby, signer and address checks")
create(A, 1, "Ann")
clock(NOW + 600)
create(B, 1, "Bo", 21)
ok("Create over a lobby idle for 10 minutes: refused (21)")
clock(NOW + 900)
create(B, 1, "Bo")
ok("Create over a lobby idle for 15 minutes: accepted (abandoned)", seat(tbl(1), 0)["key"] == B.pubkey() and head(tbl(1))["match"] == 2)
send(X, [m(B.pubkey(), False, True), m(table_key(1), False, True)], bytes([7, 1]) + u32(2) + bytes([0]), 2, watch=[table_key(1)])
ok("An action whose player did not sign: refused (2)")
send(B, [m(B.pubkey(), True, True), m(table_key(2), False, True)], bytes([7, 1]) + u32(2) + bytes([0]), 9, watch=[table_key(1), table_key(2)])
ok("An action naming table 1 with table 2's account: refused (9)")
send(B, [m(B.pubkey(), True, True), m(table_key(1), False, True), m(IDENTITY), m(VRF), m(X.pubkey(), False, True), m(SYSTEM), m(SLOT_HASHES)],
     bytes([5, 1]) + u32(2) + bytes([0]) + u32(0), 5, watch=[table_key(1)])
ok("Roll in a lobby: refused (5)")
join(B, 1, 2, "Bo", 18)
join(A, 1, 2, "Ann")
send(A, player(A, 1), bytes([4, 1]) + u32(2))
send(B, [m(B.pubkey(), True, True), m(table_key(1), False, True), m(IDENTITY), m(VRF), m(X.pubkey(), False, True), m(SYSTEM), m(SLOT_HASHES)],
     bytes([5, 1]) + u32(2) + bytes([0]) + u32(0), 1, watch=[table_key(1)])
ok("Roll naming a different VRF queue: refused (1)")
print(f"{passed} checks passed")
