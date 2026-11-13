'use strict';
// Operator setup on Solana devnet: creates Ludo Freeflow tables (the operator pays rent) and delegates each to
// MagicBlock's devnet rollup, then waits until the rollup serves it. Safe to run again: done steps are skipped.
// usage: node ludo-setup.js <operator keypair json> [from] [to]
const fs = require('fs');
const L = require('./ludo-client');
const { web3 } = L;

(async () => {
  const op = web3.Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(process.argv[2], 'utf8'))));
  const from = Number(process.argv[3] || 0), to = Number(process.argv[4] || L.NTABLES);
  const base = new web3.Connection(L.BASE_URL, 'confirmed');
  const er = new web3.Connection(L.ER_URL, 'confirmed');
  const rent = await base.getMinimumBalanceForRentExemption(L.TABLE_SIZE);
  const cu = [web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 200000 }), web3.ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 20000 })];
  console.log('program', L.PROGRAM_ID.toBase58(), 'operator', op.publicKey.toBase58(), 'balance', (await base.getBalance(op.publicKey)) / 1e9, 'table rent', rent / 1e9);
  for (let i = from; i < to; i++) {
    const pda = L.tablePda(i);
    let info = await base.getAccountInfo(pda);
    if (!info) {
      const sig = await L.send(base, [...cu, L.I.initTable(op.publicKey, i, rent)], [op], { skipPreflight: false });
      console.log(`table ${i} ${pda.toBase58()} created ${sig}`);
      info = await base.getAccountInfo(pda);
    }
    if (info.owner.equals(L.PROGRAM_ID)) {
      const sig = await L.send(base, [...cu, L.I.delegate(op.publicKey, i)], [op], { skipPreflight: false });
      console.log(`table ${i} delegated ${sig}`);
    } else console.log(`table ${i} ${pda.toBase58()} already delegated (owner ${info.owner.toBase58().slice(0, 8)})`);
    let seen = null;
    for (let t = 0; t < 20 && !seen; t++) { seen = await er.getAccountInfo(pda).catch(() => null); if (!seen) await new Promise(r => setTimeout(r, 1500)); }
    console.log(`table ${i} on the rollup: ${seen ? L.parseTable(seen.data).phase + ', owner ' + seen.owner.toBase58().slice(0, 8) : 'not yet'}`);
  }
  console.log('operator balance', (await base.getBalance(op.publicKey)) / 1e9);
})().catch(e => { console.error('ERR', e.message, (e.logs || []).join('\n')); process.exit(1); });
