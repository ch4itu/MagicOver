#!/usr/bin/env python3
"""Builds smaran.js: the sign-up code Magic Over runs, as one file any web page can load.

It copies four script blocks from ../index.html unchanged (Solana web3.js, the ChainKey crypto, TweetNaCl and the
registry client) and adds api.js. Run it after changing either file:  python3 smaran/build.py
"""
import pathlib
import re

HERE = pathlib.Path(__file__).resolve().parent
page = (HERE.parent / 'index.html').read_text(encoding='utf-8')
blocks = re.findall(r'<script(?:\s[^>]*)?>(.*?)</script>', page, flags=re.S)

# each block is found by its first line; it must be there exactly once
WANT = ['/* solana web3.js 1.98.4 */', '/* chainkey */', '/* tweetnacl */', '/* sabha-sol */']
parts = []
for head in WANT:
    found = [b for b in blocks if b.lstrip().startswith(head)]
    if len(found) != 1:
        raise SystemExit(f'expected one script block starting with {head!r} in index.html, found {len(found)}')
    parts.append(found[0].strip())
parts.append((HERE / 'api.js').read_text(encoding='utf-8').strip())

banner = ('/*! Smaran 1.0.0: sign up for a Solana app with a player name and a passphrase.\n'
          ' * Built by smaran/build.py from Magic Over (https://github.com/ch4itu/MagicOver). Project code: MIT.\n'
          ' * Bundled: Solana web3.js 1.98.4, hash-wasm, noble-ciphers and TweetNaCl, under their own licenses\n'
          ' * (see THIRD_PARTY_NOTICES.md in the repository). Load this file once; it defines the globals\n'
          ' * solanaWeb3, SabhaChainKey, nacl, SabhaSol and Smaran. */\n')
out = banner + '\n;\n'.join(parts) + '\n'
(HERE / 'smaran.js').write_text(out, encoding='utf-8')
print(f'smaran.js: {len(out.encode()):,} bytes from {len(WANT)} index.html blocks and api.js')
