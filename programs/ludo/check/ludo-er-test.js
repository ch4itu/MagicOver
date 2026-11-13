'use strict';
// Ludo Freeflow on the real rollup: two new keys with zero SOL create, join and start a match on a free table, roll
// through MagicBlock VRF at the same time, move, and try the refusals that matter (replay, roll while pending, fake
// callback, wrong seat). Prints the evidence: signatures, the VRF callback transactions and the dice.
// usage: node ludo-er-test.js [table] [rolls per player]
const L = require('./ludo-client');
const { web3 } = L;
const sleep = ms => new Promise(r => setTimeout(r, ms));
const er = new web3.Connection(L.ER_URL, 'confirmed');
const base = new web3.Connection(L.BASE_URL, 'confirmed');
const results = [];
const check = (name, ok, detail = '') => { results.push({ name, ok }); console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ' | ' + detail : ''}`); };
const read = async i => L.parseTable((await er.getAccountInfo(L.tablePda(i))).data);
async function refused(promise) { try { await promise; return null; } catch (e) { return e.code ?? e.message; } }

(async () => {
  let idx = process.argv[2] !== undefined ? Number(process.argv[2]) : null;
  const perPlayer = Number(process.argv[3] || 12);
  if (idx === null) for (let i = 0; i < L.NTABLES && idx === null; i++) { const t = await read(i); if (t.phase === 'free' || (t.phase === 'finished' && Date.now() / 1000 - t.active > 25)) idx = i; }
  if (idx === null) throw new Error('no free table');
  // A and B stand for players' Smaran keys; SA and SB are the session keys their pages would make
  const A = web3.Keypair.generate(), B = web3.Keypair.generate(), X = web3.Keypair.generate(), SA = web3.Keypair.generate(), SB = web3.Keypair.generate(), SA2 = web3.Keypair.generate();
  console.log('program', L.PROGRAM_ID.toBase58(), 'table', idx, L.tablePda(idx).toBase58());
  console.log('players', A.publicKey.toBase58(), B.publicKey.toBase58());
  const sigs = {};
  sigs.create = await L.send(er, [L.I.create(A.publicKey, idx, SA.publicKey, 'Asha')], [A]);
  let t = await read(idx);
  const match = t.match;
  sigs.join = await L.send(er, [L.I.join(B.publicKey, idx, match, SB.publicKey, 'Bala')], [B]);
  sigs.start = await L.send(er, [L.I.start(SB.publicKey, idx, match)], [SB]);
  t = await read(idx);
  check('Create and Join signed by the players\' keys (naming session keys), Start by a session key: on the rollup, with keys that hold no SOL', t.phase === 'playing' && t.seats[0].key === A.publicKey.toBase58() && t.seats[0].session === SA.publicKey.toBase58() && t.seats[1].key === B.publicKey.toBase58(),
    `match ${match}; create ${sigs.create.slice(0, 12)}…, join ${sigs.join.slice(0, 12)}…, start ${sigs.start.slice(0, 12)}…`);
  const clockGap = Math.abs(t.active - Date.now() / 1000);
  check('The rollup clock (unix seconds) is real time, so the 8 s retry and 15 min abandonment mean what they say', clockGap < 30, `${clockGap.toFixed(1)} s from this machine`);

  // fake callback and wrong-seat roll
  const fake = new web3.TransactionInstruction({ programId: L.PROGRAM_ID, keys: [{ pubkey: X.publicKey, isSigner: true, isWritable: false }, { pubkey: L.tablePda(idx), isSigner: false, isWritable: true }],
    data: Buffer.concat([Buffer.from('LFVRFCB1'), Buffer.alloc(32, 5), Buffer.from([idx]), Buffer.from(new Uint32Array([match]).buffer), Buffer.from([0]), Buffer.alloc(4)]) });
  check('A callback signed by anyone but the VRF program\'s identity is refused (13)', await refused(L.send(er, [fake], [X])) === 13);
  check('Rolling for the other player\'s seat is refused (6)', await refused(L.send(er, [L.I.roll(SA.publicKey, idx, match, 1, 0)], [SA])) === 6);
  check('Rebind signed by a session key is refused (24)', await refused(L.send(er, [L.I.rebind(SA.publicKey, idx, match, 0, SA2.publicKey)], [SA])) === 24);

  // rolls: both players at once, each waiting for its own callback
  const seen = { A: [], B: [] }, latency = [];
  async function play(who, seat, label) {
    for (let n = 0; n < perPlayer; n++) {
      let s = (await read(idx)).seats[seat];
      if (s.wait) {
        const d = s.wait, token = s.pieces.findIndex(v => v === -1 ? d === 6 : v + d <= 57);
        await L.send(er, [token < 0 ? L.I.move(who.publicKey, idx, match, seat, s.seq, 255, 0) : L.I.move(who.publicKey, idx, match, seat, s.seq, token, s.pieces[token] === -1 ? 255 : s.pieces[token])], [who]).catch(e => console.log(label, 'move refused', e.code));
        s = (await read(idx)).seats[seat];
      }
      const t0 = Date.now();
      const sig = await L.send(er, [L.I.roll(who.publicKey, idx, match, seat, s.seq)], [who]);
      if (n === 0) sigs['roll' + label] = sig;
      let after = null;
      for (let k = 0; k < 100; k++) { await sleep(250); const q = (await read(idx)).seats[seat]; if (q.pending === 0 && q.req === s.seq) { after = q; break; } }
      if (!after) { console.log(label, 'no callback within 25 s'); continue; }
      latency.push(Date.now() - t0);
      seen[label].push(after.last);
    }
  }
  const t1 = Date.now();
  await Promise.all([play(SA, 0, 'A'), play(SB, 1, 'B')]);
  const all = [...seen.A, ...seen.B];
  check(`Both players rolled at the same time; every roll got its own VRF callback`, all.length === 2 * perPlayer, `${all.length}/${2 * perPlayer} dice in ${((Date.now() - t1) / 1000).toFixed(0)} s; latency median ${latency.sort((a, b) => a - b)[latency.length >> 1]} ms, max ${latency.at(-1)} ms`);
  const hist = [1, 2, 3, 4, 5, 6].map(f => all.filter(d => d === f).length);
  check('Dice are 1-6', all.every(d => d >= 1 && d <= 6), `faces ${JSON.stringify(hist)} (A ${seen.A.join('')}, B ${seen.B.join('')})`);

  // the callback transactions themselves
  const sigsOnTable = await er.getSignaturesForAddress(L.tablePda(idx), { limit: 100 });
  let callbacks = 0, sample = null;
  for (const s of sigsOnTable.slice(0, 40)) {
    const tx = await er.getTransaction(s.signature, { commitment: 'confirmed', maxSupportedTransactionVersion: 0 }).catch(() => null);
    const logs = tx?.meta?.logMessages || [];
    if (logs.some(l => l.includes('ludo: rolled'))) {
      callbacks++;
      if (!sample) sample = { sig: s.signature, signers: tx.transaction.message.staticAccountKeys.slice(0, tx.transaction.message.header.numRequiredSignatures).map(x => x.toBase58()), logs: logs.filter(l => /invoke|ludo|success/.test(l)).slice(0, 6) };
    }
  }
  const vrfId = L.vrfIdentity().toBase58();
  check('The dice were applied by transactions that invoked the VRF program, which called back into Ludo as its identity', callbacks > 0 && sample.logs.some(l => l.includes('Vrf1RNUjX')) && sample.logs.some(l => l.includes(L.PROGRAM_ID.toBase58())),
    `${callbacks} callback transactions seen in the last 40; sample ${sample?.sig}; VRF identity ${vrfId}; logs ${JSON.stringify(sample?.logs)}`);

  // refusals on the rollup: replay, roll while a die waits / is pending
  t = await read(idx);
  const a = t.seats[0];
  if (a.wait) {
    const d = a.wait, token = a.pieces.findIndex(v => v === -1 ? d === 6 : v + d <= 57);
    const mv = token < 0 ? L.I.move(SA.publicKey, idx, match, 0, a.seq, 255, 0) : L.I.move(SA.publicKey, idx, match, 0, a.seq, token, a.pieces[token] === -1 ? 255 : a.pieces[token]);
    check('Rolling while a die waits to be played is refused (8)', await refused(L.send(er, [L.I.roll(SA.publicKey, idx, match, 0, a.seq)], [SA])) === 8);
    sigs.move = await L.send(er, [mv], [SA]);
    check('The same move sent again (retry, second device) is refused (7)', await refused(L.send(er, [mv], [SA])) === 7, `move ${sigs.move.slice(0, 12)}…`);
  }
  t = await read(idx);
  const a2 = t.seats[0];
  await L.send(er, [L.I.roll(SA.publicKey, idx, match, 0, a2.seq)], [SA]);
  check('A second roll while the first is outstanding is refused (8)', await refused(L.send(er, [L.I.roll(SA.publicKey, idx, match, 0, a2.seq + 1)], [SA])) === 8);
  // another device: Asha's Smaran key names a new session key; the old one stops at once
  await sleep(3000);
  sigs.rebind = await L.send(er, [L.I.rebind(A.publicKey, idx, match, 0, SA2.publicKey)], [A]);
  t = await read(idx);
  const a3 = t.seats[0];
  const oldRefused = await refused(L.send(er, [a3.wait ? L.I.move(SA.publicKey, idx, match, 0, a3.seq, 255, 0) : L.I.roll(SA.publicKey, idx, match, 0, a3.seq)], [SA]));
  check('After Rebind (another device), the old session key is refused (6)', t.seats[0].session === SA2.publicKey.toBase58() && oldRefused === 6, `rebind ${sigs.rebind.slice(0, 12)}…`);

  // zero SOL throughout
  const bal = await Promise.all([A, B, SA, SB, SA2].map(p => base.getBalance(p.publicKey)));
  const erBal = await Promise.all([A, B, SA, SB, SA2].map(p => er.getBalance(p.publicKey).catch(() => 0)));
  check('The players never held SOL (Solana and rollup balances 0)', bal.every(b => b === 0) && erBal.every(b => b === 0), JSON.stringify({ solana: bal, rollup: erBal }));

  // leave: the other player wins and the table frees itself 20 s later
  await sleep(4000);
  sigs.leave = await L.send(er, [L.I.leave(SB.publicKey, idx, match, 1)], [SB]);
  t = await read(idx);
  check('Bala leaves mid-match: Asha is the last colour in play and wins', t.phase === 'finished' && t.winner === 0, `leave ${sigs.leave.slice(0, 12)}…`);
  console.log('events', JSON.stringify(t.events.map(e => e.kind + ':' + e.seat)));
  console.log('signatures', JSON.stringify(sigs));
  console.log(`${results.filter(r => r.ok).length}/${results.length} passed`);
})().catch(e => { console.error('ERR', e.message, e.code, (e.logs || []).join('\n')); process.exit(1); });
