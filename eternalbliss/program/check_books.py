"""Character books (option 3) on a simulated ledger, no network: a character's lasting record lives in a book; a seat
in a world is taken only while its player is there. Joining from a book, leaving (the seat goes back to the record
and is freed), coming back with progress kept, being seated in one world at a time, a full world giving the seat of a
player who has been away 45 s to a newcomer (written back to that player's own book first), seats from before books,
and the refusals. Optional argument: the MagicBlock delegation program, to delegate a book."""
import struct
import sys
from pathlib import Path

DELEGATION_FILE = sys.argv[1] if len(sys.argv) > 1 else None
sys.argv = sys.argv[:1]
import check_world as w  # noqa: E402
from solders.keypair import Keypair  # noqa: E402
from solders.pubkey import Pubkey  # noqa: E402
from solders.transaction_metadata import TransactionMetadata  # noqa: E402

meta = w.meta
BOOK_SIZE = 64 + 80 + 80 * 112
RECORDS = 144
w.svm.set_clock = w.svm.set_clock  # (clock set through w.tick)


def world_key(n):
    return Pubkey.find_program_address([b"eb-land", bytes([n])], w.PROGRAM)[0]


def book_key(b):
    return Pubkey.find_program_address([b"eb-book", bytes([b])], w.PROGRAM)[0]


def create_world(n):
    w.tx(w.operator, [meta(w.operator.pubkey(), True, True), meta(world_key(n), writable=True), meta(w.SYSTEM)],
         struct.pack("<BBQ", 16, n, w.svm.minimum_balance_for_rent_exemption(w.SIZE)))


def create_book(b, ok=True):
    return w.tx(w.operator, [meta(w.operator.pubkey(), True, True), meta(book_key(b), writable=True), meta(w.SYSTEM)],
                struct.pack("<BBQ", 40, b, w.svm.minimum_balance_for_rent_exemption(BOOK_SIZE)), ok)


def book(b):
    return w.svm.get_account(book_key(b)).data


def record(b, key):
    data = book(b)
    for r in range(80):
        at = RECORDS + r * 112
        if data[at:at + 32] == bytes(key.pubkey()):
            return r, data[64 + r], data[at:at + 112]
    return None


def seat(n, key):
    data = w.svm.get_account(world_key(n)).data
    for i in range(32):
        at = w.H + i * 112
        if data[at:at + 32] == bytes(key.pubkey()):
            return i, data[at:at + 112]
    return None


def code(result):
    import re
    found = re.search(r"Custom\((\d+)\)", str(result))
    return int(found[1]) if found else None


def send(key, n, op, args=b"", *, accounts=(), seq=None, first=0, ok=True):
    own = seat(n, key)
    nonce = (struct.unpack_from("<I", own[1], 84)[0] if own else 0) if seq is None else seq
    keys = [meta(key.pubkey(), True, True), meta(world_key(n), writable=True)] + list(accounts)
    data = struct.pack("<B", op) + (struct.pack("<BBBB", first, 0, 0, 0) if op == 18 else struct.pack("<I", nonce)) + args
    svm = w.svm
    svm.expire_blockhash()
    from solders.instruction import Instruction
    from solders.transaction import Transaction
    from solders.transaction_metadata import FailedTransactionMetadata
    tx = Transaction.new_signed_with_payer([Instruction(w.PROGRAM, data, keys)], key.pubkey(), [key], svm.latest_blockhash())
    result = svm.send_transaction(tx)
    assert isinstance(result, TransactionMetadata if ok else FailedTransactionMetadata), result
    return result


def join(key, name, n, home, other=None, *, first=0, ok=True):
    accounts = [meta(w.registered(name, key)), meta(book_key(home), writable=True)]
    if other is not None:
        accounts.append(meta(book_key(other), writable=True))
    return send(key, n, 18, name.encode(), accounts=accounts, first=first, ok=ok)


def leave(key, n, home, ok=True):
    return send(key, n, 24, accounts=[meta(book_key(home), writable=True)], ok=ok)


def u32(data, at):
    return struct.unpack_from("<I", data, at)[0]


def pos(data):
    return (struct.unpack_from("<H", data, 64)[0] & 511, struct.unpack_from("<H", data, 66)[0] & 511)


w.tick(1000)
for n in (0, 1):
    create_world(n)
create_book(0)
create_book(1)
w.check("books 0 and 1 exist, empty, numbered", book(0)[:8] == b"EBBOOK01" and book(1)[60] == 1 and len(book(0)) == BOOK_SIZE
        and not any(book(0)[64:]))
create_book(0, ok=False)
w.check("a book cannot be created twice", True)

players = [(Keypair(), f"book-player-{i:02d}") for i in range(34)]
for key, _ in players:
    assert isinstance(w.svm.airdrop(key.pubkey(), 100_000_000), TransactionMetadata)
(ivy, ivy_name), (ash, ash_name) = players[0], players[1]

# 1. join from a book: a new record and a seat
join(ivy, ivy_name, 0, 0)
r, place, rec = record(0, ivy)
i, s = seat(0, ivy)
w.check("a new player gets a record in its book, marked as seated in world 0, and a seat", place == 1 and
        rec[32:32 + len(ivy_name)] == ivy_name.encode() and s[:32] == bytes(ivy.pubkey()))
home = (struct.unpack_from("<H", s, 68)[0] >> 9) | ((struct.unpack_from("<H", s, 70)[0] >> 9) << 7)
w.check("the seat remembers its home book, and the safe tile keeps its coordinates",
        home == 1 and (struct.unpack_from("<H", s, 68)[0] & 511, struct.unpack_from("<H", s, 70)[0] & 511) == (75, 75))

# progress in the world: walk and earn
w.tick(1010)
w.place_for_test(ivy, 80, 80)
send(ivy, 0, 19, b"\x01\x01")
i, s = seat(0, ivy)
gold_before = u32(s, 76)
w.check("playing works as before: two steps", pos(s) == (82, 80))

# 2. leave with the book: the seat goes back to the record and is freed
leave(ivy, 0, 0)
r, place, rec = record(0, ivy)
w.check("leaving writes the character back to its record and frees the seat",
        seat(0, ivy) is None and place == 0 and pos(rec) == (82, 80) and u32(rec, 76) == gold_before and u32(rec, 88) == 0)
rec_home = (struct.unpack_from("<H", rec, 68)[0] >> 9)
w.check("the stored record keeps plain coordinates", rec_home == 0 and (struct.unpack_from("<H", rec, 68)[0] & 511) == 82)

# 3. coming back: same character, same progress
w.tick(1100)
join(ivy, ivy_name, 0, 0)
i, s = seat(0, ivy)
w.check("coming back seats the same character with its progress", pos(s) == (82, 80) and u32(s, 76) == gold_before and
        record(0, ivy)[1] == 1 and u32(s, 88) == 1100)

# 4. one world at a time
w.check("a character seated in world 0 cannot join world 1 (40)", code(join(ivy, ivy_name, 1, 0, ok=False)) == 40 and seat(1, ivy) is None)
leave(ivy, 0, 0)
join(ivy, ivy_name, 1, 0)
w.check("after leaving world 0 it can join world 1, with the same progress", pos(seat(1, ivy)[1]) == (82, 80) and record(0, ivy)[1] == 2)
leave(ivy, 1, 0)

# 5. a full world: fill world 0, then a newcomer takes the seat of a player away for 45 s
w.tick(2000)
for key, name in players[2:34]:
    join(key, name, 0, 0 if int(name[-2:]) % 2 == 0 else 1)
w.check("32 players fill world 0, their records split across two books", all(seat(0, k) for k, _ in players[2:34]))
newcomer, newcomer_name = ash, ash_name
w.check("while everyone is present, a full world refuses a newcomer (30)",
        code(join(newcomer, newcomer_name, 0, 0, first=5, ok=False)) == 30 and seat(0, newcomer) is None)
# player 03 (book 1) goes quiet; everyone else stays present
away, away_name = players[3]
away_seat = seat(0, away)[0]
w.tick(2050)
for key, name in players[2:34]:
    if key is not away:
        send(key, 0, 25)
w.check("the seat of a player away 45 s goes back only to that player's own book: book 0 is refused (42)",
        code(join(newcomer, newcomer_name, 0, 0, other=None, first=away_seat, ok=False)) == 42 and seat(0, newcomer) is None)
before = seat(0, away)[1]
join(newcomer, newcomer_name, 0, 0, other=1, first=away_seat)
r, place, rec = record(1, away)
w.check("with its book, the away player's seat is written back to its record and given to the newcomer",
        seat(0, away) is None and place == 0 and rec[:88] == before[:68] + rec[68:72] + before[72:88] and seat(0, newcomer)[0] == away_seat)
present = players[4][0]
late, late_name = Keypair(), "book-late"
assert isinstance(w.svm.airdrop(late.pubkey(), 100_000_000), TransactionMetadata)
w.check("a present player's seat cannot be taken (30)",
        code(join(late, late_name, 0, 0, other=0, first=seat(0, present)[0], ok=False)) == 30 and seat(0, present) is not None)

# 6. the away player comes back: its record holds its progress
w.tick(2100)
leave(newcomer, 0, 0)
join(away, away_name, 0, 1)
w.check("the away player comes back with its record", seat(0, away) is not None and record(1, away)[1] == 1)

# 7. a seat from before books (no home book) is written into a free record when it is freed
legacy, legacy_name = players[33]
leave(legacy, 0, 1)
w.check("without a book, a join cannot make a new character (43), even for a player with a record",
        code(send(legacy, 0, 18, legacy_name.encode(), accounts=[meta(w.registered(legacy_name, legacy))], ok=False)) == 43
        and seat(0, legacy) is None)
# a seat from before books: made the old way, then written into a book when it is freed
join(legacy, legacy_name, 0, 1)
data = bytearray(w.svm.get_account(world_key(0)).data)
i = seat(0, legacy)[0]
at = w.H + i * 112
struct.pack_into("<HH", data, at + 68, 80, 81)  # no home book in the upper bits, as before books
from solders.account import Account
w.svm.set_account(world_key(0), Account(w.svm.get_account(world_key(0)).lamports, bytes(data), w.PROGRAM))
book1 = bytearray(book(1))
r = record(1, legacy)[0]
book1[144 + r * 112:144 + (r + 1) * 112] = bytes(112)
book1[64 + r] = 0
w.svm.set_account(book_key(1), Account(w.svm.get_account(book_key(1)).lamports, bytes(book1), w.PROGRAM))
send(legacy, 0, 25)
w.check("a join without a book resumes an existing seat (the old page)",
        code(send(legacy, 0, 18, legacy_name.encode(), accounts=[meta(w.registered(legacy_name, legacy))])) is None)
w.tick(2200)
for key, name in players[2:34]:
    if key is not legacy and seat(0, key):
        send(key, 0, 25)
legacy_seat = seat(0, legacy)
w.tick(2300)
for key, name in players[2:34]:
    if key is not legacy and seat(0, key):
        send(key, 0, 25)
join(newcomer, newcomer_name, 0, 0, other=None, first=legacy_seat[0])
r, place, rec = record(0, legacy)
w.check("a seat with no home book is written into a free record of the book passed, its progress kept",
        seat(0, legacy) is None and place == 0 and rec[:68] == legacy_seat[1][:68] and rec[72:88] == legacy_seat[1][72:88])

# 8. release: a fresh character can remove its seat and its record
fresh, fresh_name = Keypair(), "book-fresh"
assert isinstance(w.svm.airdrop(fresh.pubkey(), 100_000_000), TransactionMetadata)
leave(newcomer, 0, 0)
join(fresh, fresh_name, 0, 0)
send(fresh, 0, 24)
send(fresh, 0, 28, accounts=[meta(book_key(0), writable=True)])
w.check("a fresh character that has left can remove its seat and its record", seat(0, fresh) is None and record(0, fresh) is None)

if DELEGATION_FILE:
    w.svm.add_program_from_file(w.DELEGATION, Path(DELEGATION_FILE).resolve())
    key = book_key(1)
    buffer = Pubkey.find_program_address([b"buffer", bytes(key)], w.PROGRAM)[0]
    rec_key = Pubkey.find_program_address([b"delegation", bytes(key)], w.DELEGATION)[0]
    meta_key = Pubkey.find_program_address([b"delegation-metadata", bytes(key)], w.DELEGATION)[0]
    w.tx(w.operator, [meta(w.operator.pubkey(), True, True), meta(key, writable=True), meta(w.PROGRAM),
                      meta(buffer, writable=True), meta(rec_key, writable=True), meta(meta_key, writable=True),
                      meta(w.DELEGATION), meta(w.SYSTEM)], bytes([41, 1]))
    w.check("the MagicBlock delegation program accepts a book", w.svm.get_account(key).owner == w.DELEGATION)
print("all book checks passed")
