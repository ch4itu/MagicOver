"""Create shared worlds 1 to <last> on Devnet, each with a save reserve and its four duel tables (world w uses tables
4w to 4w+3), and delegate them to MagicBlock's devnet-as rollup. Accounts that already exist are not created again,
and delegated ones are left alone. Reuses the helpers in deploy_devnet.py.
usage: python3 add_worlds.py <program id> <payer keypair json> <last world> [world reserve lamports]
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

program = Pubkey.from_string(sys.argv[1])
payer = Keypair.from_bytes(bytes(json.loads(Path(sys.argv[2]).read_text())))
last = int(sys.argv[3])
reserve = int(sys.argv[4]) if len(sys.argv) > 4 else 100_000_000
assert 1 <= last < 64
assert d.rpc("getGenesisHash") == d.DEVNET, "not Devnet"
assert d.rpc("getIdentity", url=d.ER)["identity"] == d.VALIDATOR, "unexpected rollup"
info = d.read_account(program)
assert info and info["executable"], "program not deployed"

wanted = []
for world in range(1, last + 1):
    wanted.append((b"eb-land", world, 16, 17, d.WORLD_SIZE))
    wanted += [(b"eb-duel", table, 0, 1, 112) for table in range(4 * world, 4 * world + 4)]
print("balance before", d.rpc("getBalance", [str(payer.pubkey())])["value"])
for seed, index, initialize, delegate, space in wanted:
    pda = Pubkey.find_program_address([seed, bytes([index])], program)[0]
    amount = d.rent(space) + (reserve if initialize == 16 else 0)
    init, delegation = d.setup_instructions(program, payer.pubkey(), pda, index, initialize, delegate, amount)
    state = d.read_account(pda)
    if not state:
        d.send([init], [payer])
        state = d.read_account(pda)
        print("created", seed.decode(), index, pda, "lamports", state["lamports"])
    if state["owner"] == str(program):
        d.send([delegation], [payer])
        print("delegated", seed.decode(), index, pda)
    elif state["owner"] != str(d.DELEGATION):
        raise RuntimeError(f"unexpected owner at {pda}: {state['owner']}")

for seed, index, initialize, delegate, space in wanted:
    pda = Pubkey.find_program_address([seed, bytes([index])], program)[0]
    for _ in range(60):
        account = d.read_account(pda, d.ER)
        if account and account["owner"] == str(program):
            data = base64.b64decode(account["data"][0])
            if initialize == 16:
                ok = len(data) == space and data[:8] == b"EBWRLD01" and data[60] == index
            else:
                ok = len(data) == space and data[:8] == b"EBDUEL01" and data[8] == index
            if ok:
                print("on the rollup", seed.decode(), index, pda, "lamports", account["lamports"])
                break
        time.sleep(2)
    else:
        raise RuntimeError(f"the rollup has not picked up {pda}")
print("balance left", d.rpc("getBalance", [str(payer.pubkey())])["value"])
