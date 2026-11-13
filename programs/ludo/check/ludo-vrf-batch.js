'use strict';
// A long four-player session on the rollup: four new keys (zero SOL) fill a table, the match starts by itself, and
// every seat rolls and moves at once for the given minutes (moving its first legal token, or passing). Collects every
// VRF die for a fairness check (chi-square, 5 degrees of freedom), callback latency, retries after unanswered rolls,
// refused actions, and the events the program wrote. Ends with all but one leaving (the last wins).
// usage: node ludo-vrf-batch.js [minutes]
const L = require('./ludo-client');
const { web3 } = L;
const sleep = ms => new Promise(r => setTimeout(r, ms));
const er = new web3.Connection(L.ER_URL, 'confirmed');
const read = async i => L.parseTable((await er.getAccountInfo(L.tablePda(i))).data);

function legalTokens(t, seat, d) {
  const starts = [0, 13, 26, 39], P = t.seats.map(s => s.pieces);
  const block = (q, r) => P[q].filter(v => v >= 0 && v < 52 && (starts[q] + v) % 52 === r).length;
  return [0, 1, 2, 3].filter(i => {
    const v = P[seat][i];
    if (v === -1) return d === 6;
    if (v >= 57 || v + d > 57) return false;
    for (let s = 1; s <= d; s++) { const n = v + s; if (n >= 52) break; const r = (starts[seat] + n) % 52; for (let q = 0; q < 4; q++) if (q !== seat && block(q, r) >= 2) return false; }
    return true;
  });
}

(async () => {
  const minutes = Number(process.argv[2] || 4);
  let idx = null;
  for (let i = 0; i < L.NTABLES && idx === null; i++) { const t = await read(i); if (t.phase === 'free' || (t.phase === 'finished' && Date.now() / 1000 - t.active > 25)) idx = i; }
  if (idx === null) throw new Error('no free table');
  const owners = [0, 1, 2, 3].map(() => web3.Keypair.generate()), sessions = [0, 1, 2, 3].map(() => web3.Keypair.generate());
  await L.send(er, [L.I.create(owners[0].publicKey, idx, sessions[0].publicKey, 'Batch0')], [owners[0]]);
  const match = (await read(idx)).match;
  for (let s = 1; s < 4; s++) await L.send(er, [L.I.join(owners[s].publicKey, idx, match, sessions[s].publicKey, 'Batch' + s)], [owners[s]]);
  let t = await read(idx);
  console.log('table', idx, L.tablePda(idx).toBase58(), 'match', match, 'phase', t.phase, '(started by the fourth join)');
  const dice = [], latency = [], refusals = {}, stats = { retries: 0, moves: 0, passes: 0 };
  const end = Date.now() + minutes * 60000;
  async function seatLoop(seat) {
    const me = sessions[seat];
    while (Date.now() < end) {
      t = await read(idx);
      if (t.phase !== 'playing') return;
      const s = t.seats[seat];
      try {
        if (s.wait) {
          const opts = legalTokens(t, seat, s.wait);
          const i = opts[0];
          await L.send(er, [i === undefined ? L.I.move(me.publicKey, idx, match, seat, s.seq, 255, 0) : L.I.move(me.publicKey, idx, match, seat, s.seq, i, s.pieces[i] === -1 ? 255 : s.pieces[i])], [me]);
          if (i === undefined) stats.passes++; else stats.moves++;
        } else if (!s.pending || Date.now() / 1000 - s.reqAt > 10) {
          if (s.pending) stats.retries++;
          const t0 = Date.now(), seq = s.seq;
          await L.send(er, [L.I.roll(me.publicKey, idx, match, seat, seq)], [me]);
          for (let k = 0; k < 60; k++) {
            await sleep(250);
            const q = (await read(idx)).seats[seat];
            if (q.pending === 0 && q.req === seq) { dice.push(q.last); latency.push(Date.now() - t0); break; }
          }
        } else await sleep(300);
      } catch (e) { refusals[e.code ?? 'other'] = (refusals[e.code ?? 'other'] || 0) + 1; await sleep(300); }
    }
  }
  await Promise.all([0, 1, 2, 3].map(seatLoop));
  t = await read(idx);
  const counts = [1, 2, 3, 4, 5, 6].map(f => dice.filter(d => d === f).length), n = dice.length, e = n / 6;
  const chi = counts.reduce((a, c) => a + (c - e) ** 2 / e, 0);
  latency.sort((a, b) => a - b);
  console.log(`dice ${n}: faces ${JSON.stringify(counts)}; chi-square ${chi.toFixed(2)} (5 d.f.; 11.07 is the 5% critical value)`);
  console.log(`roll-to-die latency: median ${latency[n >> 1]} ms, 95th ${latency[Math.floor(n * 0.95)]} ms, max ${latency.at(-1)} ms`);
  console.log(`moves ${stats.moves}, passes ${stats.passes}, re-rolls after an unanswered roll ${stats.retries}, refused actions ${JSON.stringify(refusals)} (7 = the board moved on, 15 = a block appeared: expected in a race)`);
  console.log(`state: ${t.phase}, winner ${t.winner}; captures ${JSON.stringify(t.seats.map(s => s.captures))}; tokens home ${JSON.stringify(t.seats.map(s => s.pieces.filter(v => v === 57).length))}; events written ${t.eventCount}`);
  if (t.phase === 'playing') for (let s = 1; s < 4; s++) await L.send(er, [L.I.leave(sessions[s].publicKey, idx, match, s)], [sessions[s]]).catch(() => {});
  t = await read(idx);
  console.log('after three leave:', t.phase, 'winner', t.winner);
})().catch(e => { console.error('ERR', e.message, e.code); process.exit(1); });
