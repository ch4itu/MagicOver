'use strict';
// Differential check: the on-chain rules (programs/ludo, through its host tests) against the Freeflow page's own
// JavaScript. Replays every action of a trace written by `LUDO_TRACE=<file> cargo test random_games` through the
// page's roll(), move() and legal() in a sandbox with the page's DOM and canvas stubbed out, and compares the whole
// game state after every action. The page's die() is replaced by the trace's die; its bots are kept from moving on
// their own (the trace says which token every colour moves). No network.
// usage: node ludo-diff.js <ludo-freeflow/index.html> <trace.jsonl>
const fs = require('fs');
const vm = require('vm');
const [page, tracePath] = process.argv.slice(2);
const html = fs.readFileSync(page, 'utf8');
const game = html.match(/<\/main><script>([\s\S]*?)<\/script>/)[1];

const stub = () => new Proxy(function () {}, {
  get: (_, k) => (k === Symbol.toPrimitive ? () => 0 : k === 'offsetWidth' ? 0 : stub()),
  set: () => true,
  apply: () => stub(),
});
const ctx = vm.createContext({
  document: stub(), matchMedia: () => ({ matches: true }), requestAnimationFrame: () => 0,
  performance, crypto: require('crypto').webcrypto, Uint32Array, Math, console,
});
vm.runInContext(game, ctx);
vm.runInContext(`var __nextDie = 0, __inRoll = false; const __origMove = move;
  die = () => __nextDie;
  move = (p, i) => { if (!__inRoll) __origMove(p, i); };
  var __roll = (p, d) => { __nextDie = d; __inRoll = true; try { roll(p); } finally { __inRoll = false; } };
  var __state = () => ({ pieces: pieces.map(a => a.slice()), waiting: waiting.map(w => w === null ? 0 : w), sixes: sixes.slice(),
    bonus: bonus.slice(), last: lastRolls.map(v => v === null ? 0 : v), captures: captureCounts.slice(), over,
    winner: over ? pieces.findIndex(a => a.every(x => x === 57)) : 255 });`, ctx);

const lines = fs.readFileSync(tracePath, 'utf8').trim().split('\n');
let games = 0, actions = 0, mismatches = 0;
const counts = { roll: 0, move: 0, pass: 0 };
for (const line of lines) {
  const r = JSON.parse(line);
  if ('game' in r) { vm.runInContext('reset()', ctx); games++; continue; }
  actions++; counts[r.a]++;
  if (r.a === 'roll') vm.runInContext(`__roll(${r.seat}, ${r.die})`, ctx);
  else if (r.a === 'move') vm.runInContext(`__origMove(${r.seat}, ${r.token})`, ctx);
  else vm.runInContext(`waiting[${r.seat}] = null`, ctx);
  const js = vm.runInContext('__state()', ctx);
  const want = r.state;
  const diff = Object.keys(want).filter(k => JSON.stringify(js[k]) !== JSON.stringify(want[k]));
  if (diff.length) {
    mismatches++;
    if (mismatches <= 5) console.log('MISMATCH game', games - 1, 'action', JSON.stringify(r).slice(0, 80), diff.map(k => `${k}: page ${JSON.stringify(js[k])} program ${JSON.stringify(want[k])}`).join('; '));
  }
}
console.log(`${games} games, ${actions} actions (${counts.roll} rolls, ${counts.move} moves, ${counts.pass} passes): ${mismatches ? mismatches + ' MISMATCHES' : 'page and program agree on every state'}`);
process.exit(mismatches ? 1 : 0);
