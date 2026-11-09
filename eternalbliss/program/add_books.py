"""Create character books 0 to <last> on Devnet, each with a save reserve, and delegate them to MagicBlock's devnet-as
rollup. A book ["eb-book", b] keeps 80 characters' lasting records (instruction 40 creates it, 41 delegates it, 29
saves it like a world). Accounts that already exist are not created again, and delegated ones are left alone. Reuses
the helpers in deploy_devnet.py.
usage: python3 add_books.py <program id> <payer keypair json> <last book> [book reserve lamports]
"""
import base64
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import deploy_devnet as d  # noqa: E402
from solders.keypair import Keypair  # noqa: E402
from solders.pubkey import Pubkey  # noqa: E402

BOOK_SIZE = 9104
program = Pubkey.from_string(sys.argv[1])
payer = Keypair.from_bytes(bytes(json.loads(Path(sys.argv[2]).read_text())))
last = int(sys.argv[3])
reserve = int(sys.argv[4]) if len(sys.argv) > 4 else 50_000_000
assert 0 <= last < 256
assert d.rpc("getGenesisHash") == d.DEVNET, "not Devnet"
assert d.rpc("getIdentity", url=d.ER)["identity"] == d.VALIDATOR, "unexpected rollup"
info = d.read_account(program)
assert info and info["executable"], "program not deployed"

print("balance before", d.rpc("getBalance", [str(payer.pubkey())])["value"])
books = range(0, last + 1)
for number in books:
    pda = Pubkey.find_program_address([b"eb-book", bytes([number])], program)[0]
    init, delegation = d.setup_instructions(program, payer.pubkey(), pda, number, 40, 41, d.rent(BOOK_SIZE) + reserve)
    state = d.read_account(pda)
    if not state:
        d.send([init], [payer])
        state = d.read_account(pda)
        print("created book", number, pda, "lamports", state["lamports"])
    if state["owner"] == str(program):
        d.send([delegation], [payer])
        print("delegated book", number, pda)
    elif state["owner"] != str(d.DELEGATION):
        raise RuntimeError(f"unexpected owner at {pda}: {state['owner']}")

for number in books:
    pda = Pubkey.find_program_address([b"eb-book", bytes([number])], program)[0]
    for _ in range(60):
        account = d.read_account(pda, d.ER)
        if account and account["owner"] == str(program):
            data = base64.b64decode(account["data"][0])
            if len(data) == BOOK_SIZE and data[:8] == b"EBBOOK01" and data[60] == number:
                print("on the rollup: book", number, pda, "lamports", account["lamports"])
                break
        time.sleep(2)
    else:
        raise RuntimeError(f"the rollup has not picked up {pda}")
print("balance left", d.rpc("getBalance", [str(payer.pubkey())])["value"])
