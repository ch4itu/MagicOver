"""Test-only JSON-lines bridge to the checked-in sBPF binary. Nothing is sent to Devnet."""
import base64
import json
import struct
import sys
from pathlib import Path

program = Path(__file__).resolve().parents[2] / "program"
source = program / "check_world.py"
fixture = {"__file__": str(source), "__name__": "fixture"}
code = source.read_text().replace(
    'ROOT / "target/bpfel-unknown-none/release/libeternalbliss_world.so"',
    'ROOT / "libeternalbliss_world.so"',
)
exec(compile(code, str(source), "exec"), fixture)
operator, world, svm = (fixture[key] for key in ("operator", "WORLD", "svm"))
meta, tx, tick = (fixture[key] for key in ("meta", "tx", "tick"))
players = [fixture["ivy"], fixture["ash"]]
names = ["ivy-hero", "ash-hero"]
shards = [fixture["registered"](name, player) for name, player in zip(names, players)]
tick(1000)
size = fixture["SIZE"]
tx(operator, [meta(operator.pubkey(), True, True), meta(world, writable=True), meta(fixture["SYSTEM"])],
   struct.pack("<BBQ", 16, 0, svm.minimum_balance_for_rent_exemption(size)))
book = fixture["book_account"]()
slot = 1

for line in sys.stdin:
    try:
        request = json.loads(line)
        op = request["op"]
        index = request.get("index", 0)
        player = players[index]
        now = request.get("time", 1000)
        tick(now)
        if op == "join":
            fixture["action"](player, 18, names[index].encode(), shard=shards[index])
            slot += 1
        if op == "place":
            fixture["place_for_test"](player, request["x"], request["y"])
        if isinstance(op, int):
            payload = bytes(request.get("payload", []))
            sequence = request["sequence"]
            if op == 24:
                tx(player, [meta(player.pubkey(), True, True), meta(world, writable=True), book],
                   struct.pack("<BI", op, sequence))
            else:
                fixture["action"](player, op, payload, seq=sequence)
            slot += 1
        result = {"slot": slot, "time": now, "data": base64.b64encode(svm.get_account(world).data).decode(),
                  "names": names, "owners": [str(key.pubkey()) for key in players]}
        print(json.dumps(result), flush=True)
    except Exception as error:
        print(json.dumps({"error": str(error)}), flush=True)
