'use strict';
// Waits until the rollup runs the current Ludo build: simulates a Create (session-key format) on a free table.
const L = require('./ludo-client');
const { web3 } = L;
(async () => {
  const er = new web3.Connection(L.ER_URL, 'confirmed');
  const A = web3.Keypair.generate(), S = web3.Keypair.generate();
  for (let n = 0; n < Number(process.argv[2] || 20); n++) {
    let idx = null;
    for (let i = 0; i < L.NTABLES && idx === null; i++) { const t = L.parseTable((await er.getAccountInfo(L.tablePda(i))).data); if (t.phase === 'free') idx = i; }
    const tx = new web3.Transaction().add(L.I.create(A.publicKey, idx, S.publicKey, 'Probe'));
    tx.feePayer = A.publicKey; tx.recentBlockhash = (await er.getLatestBlockhash()).blockhash; tx.sign(A);
    const r = await er.simulateTransaction(tx);
    const ok = !r.value.err;
    console.log(new Date().toISOString().slice(11, 19), 'table', idx, ok ? 'NEW BUILD (create simulates fine)' : 'old build? ' + JSON.stringify(r.value.err), (r.value.logs || []).filter(l => l.includes('consumed')).join(' '));
    if (ok) return;
    await new Promise(r => setTimeout(r, 15000));
  }
})();
