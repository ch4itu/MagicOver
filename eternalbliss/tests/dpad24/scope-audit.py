import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
from datetime import datetime, timezone

out = Path(os.environ['EB_EVIDENCE'])
repo = Path(os.environ.get('EB_REPO', '/workspace/MagicOver'))
old = subprocess.check_output(['git', 'show', '2746efbb2576ec48f5c47573314dd713aacbc3ce:eternalbliss/index.html'], cwd=repo)
new = (repo / 'eternalbliss/index.html').read_bytes()
sha = lambda b: hashlib.sha256(b).hexdigest()
assert sha(old) == 'b0c9cd4243fe595ebcc002d66a939390e8ffd88ba8445452bdb7e6b1c70c26dd'
assert sha(new) == '18ecff7c1bfd4f12502b8216e2720b8b3c32e2f62d0fde033b719c33906bdf0c'
proof = json.loads((out / 'scope-audit.json').read_text())
assert old.count(proof['old'].encode()) == 1
assert old.replace(proof['old'].encode(), proof['new'].encode()) == new
blocks = lambda h: re.findall(rb'<script\b[^>]*>[\s\S]*?</script>', h)
assert blocks(old) == blocks(new)
assert len(blocks(old)) == 29
files = subprocess.check_output(['git', 'diff', '--name-only', '2746efbb2576ec48f5c47573314dd713aacbc3ce'], cwd=repo, text=True).splitlines()
assert all(f.startswith('eternalbliss/') for f in files)
row = dict(time=datetime.now(timezone.utc).isoformat(), sha=sha(new), script=sha(Path(__file__).read_bytes()), test='D-pad correction exact source scope', browser='Python byte comparison; not browser or chain testing', result=dict(result='PASS only declared CSS replacement; every other byte and all 29 scripts unchanged', before=sha(old), after=sha(new), scriptBlocks=29, old=proof['old'], new=proof['new']))
(out / 'scope-audit-final.json').write_text(json.dumps(row, indent=2) + '\n')
with (out / 'ledger.jsonl').open('a') as f:
    f.write(json.dumps(row) + '\n')
print('PASS exact CSS-only scope, frozen HTML, all 29 inline scripts unchanged')
