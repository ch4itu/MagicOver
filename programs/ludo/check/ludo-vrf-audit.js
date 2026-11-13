'use strict';
// Reads every die-applying transaction of a match from the rollup and checks it at the byte level: each was a call by
// MagicBlock's VRF program into Ludo, with the VRF program's identity as signer and 32 bytes of randomness. Then tests the
// raw randomness bytes for uniformity (chi-square, 255 d.f.) and reports the dice Ludo's mapping gives for them (the
// mapping itself is checked by the host tests and the simulator checks).
// usage: node ludo-vrf-audit.js <table> [more tables...]
const L = require('./ludo-client');
const { web3 } = L;
const bs58 = s => { const A = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'; let n = 0n; for (const c of s) n = n * 58n + BigInt(A.indexOf(c)); const out = []; while (n > 0n) { out.unshift(Number(n % 256n)); n /= 256n; } for (const c of s) { if (c === '1') out.unshift(0); else break; } return Uint8Array.from(out); };
const dieFrom = r => { for (const b of r) if (b < 252) return (b % 6) + 1; return (r[0] % 6) + 1; };
const er = new web3.Connection(L.ER_URL, 'confirmed');

(async () => {
  const bytes = new Array(256).fill(0), dice = [0, 0, 0, 0, 0, 0, 0];
  let callbacks = 0, notVrf = 0, wrongSigner = 0;
  const vrfId = L.vrfIdentity().toBase58(), program = L.PROGRAM_ID.toBase58();
  for (const idx of process.argv.slice(2).map(Number)) {
    const pda = L.tablePda(idx);
    let sigs = [], before;
    for (;;) { const page = await er.getSignaturesForAddress(pda, { limit: 1000, before }); if (!page.length) break; sigs.push(...page); before = page.at(-1).signature; if (page.length < 1000) break; }
    for (let i = 0; i < sigs.length; i += 25) {
      const txs = await er.getTransactions(sigs.slice(i, i + 25).map(s => s.signature), { commitment: 'confirmed', maxSupportedTransactionVersion: 0 });
      for (const tx of txs) {
        if (!tx || tx.meta?.err || !(tx.meta?.logMessages || []).some(l => l.includes('ludo: rolled'))) continue;
        callbacks++;
        const keys = tx.transaction.message.staticAccountKeys.map(k => k.toBase58());
        const top = tx.transaction.message.compiledInstructions.map(c => keys[c.programIdIndex]);
        if (!top.includes(L.VRF.toBase58())) notVrf++;
        const inner = (tx.meta.innerInstructions || []).flatMap(g => g.instructions).find(x => keys[x.programIdIndex] === program);
        if (!inner) { notVrf++; continue; }
        const data = bs58(inner.data);
        if (Buffer.from(data.subarray(0, 8)).toString() !== 'LFVRFCB1' || keys[inner.accounts[0]] !== vrfId) { wrongSigner++; continue; }
        const rnd = data.subarray(8, 40);
        for (const b of rnd) bytes[b]++;
        dice[dieFrom(rnd)]++;
      }
    }
    console.log(`table ${idx}: ${sigs.length} transactions read`);
  }
  const n = bytes.reduce((a, b) => a + b, 0), e = n / 256;
  const chiBytes = bytes.reduce((a, c) => a + (c - e) ** 2 / e, 0);
  const m = dice.slice(1).reduce((a, b) => a + b, 0), ed = m / 6;
  const chiDice = dice.slice(1).reduce((a, c) => a + (c - ed) ** 2 / ed, 0);
  console.log(`die-applying transactions: ${callbacks}; not a VRF-program call into Ludo: ${notVrf}; wrong discriminator or signer: ${wrongSigner}`);
  console.log(`randomness bytes: ${n}; chi-square ${chiBytes.toFixed(1)} over 255 d.f. (5% critical value 293.2, mean 255)`);
  console.log(`dice those bytes map to: ${JSON.stringify(dice.slice(1))}; chi-square ${chiDice.toFixed(2)} (5 d.f.; 5% critical 11.07)`);
})().catch(e => { console.error('ERR', e.message); process.exit(1); });
