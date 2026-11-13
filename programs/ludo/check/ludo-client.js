'use strict';
// Node client for the Ludo Freeflow program (tests and operator setup): addresses, instructions, table parsing.
// Needs @solana/web3.js. The page carries its own browser copy of the same layout.
const web3 = require('@solana/web3.js');

const PROGRAM_ID = new web3.PublicKey(process.env.LUDO_PROGRAM || '2i2REZWzDeAqKuJBEypH9fBtVZB6jmZbBRX7YH4fiETp');
const DELEGATION = new web3.PublicKey('DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh');
const MAGIC = new web3.PublicKey('Magic11111111111111111111111111111111111111');
const MAGIC_CONTEXT = new web3.PublicKey('MagicContext1111111111111111111111111111111');
const VRF = new web3.PublicKey('Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz');
const QUEUE = new web3.PublicKey('5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc');
const SLOT_HASHES = new web3.PublicKey('SysvarS1otHashes111111111111111111111111111');
const BASE_URL = 'https://api.devnet.solana.com';
const ER_URL = 'https://devnet-as.magicblock.app';
const TABLE_SIZE = 512, NTABLES = 8, T_SEATS = 40, SEAT = 104, T_RING = 456, RING = 7;
const PHASES = ['free', 'lobby', 'playing', 'finished'];
const EVENTS = { 1: 'join', 2: 'start', 3: 'roll', 4: 'move', 5: 'capture', 6: 'home', 7: 'forfeit', 8: 'nomove', 9: 'win', 10: 'leave', 11: 'pass' };
const ERRORS = { 24: 'smaran key only', 1: 'accounts', 2: 'signer', 3: 'owner', 4: 'data', 5: 'phase', 6: 'seat', 7: 'seq', 8: 'busy', 9: 'pda', 10: 'cpi', 11: 'tag', 12: 'range', 13: 'identity', 14: 'match', 15: 'illegal', 16: 'stale', 17: 'full', 18: 'seated', 19: 'no die', 20: 'players', 21: 'table busy', 22: 'operator', 23: 'request' };

const tablePda = i => web3.PublicKey.findProgramAddressSync([Buffer.from('lf-table'), Buffer.from([i])], PROGRAM_ID)[0];
const identityPda = () => web3.PublicKey.findProgramAddressSync([Buffer.from('identity')], PROGRAM_ID)[0];
const vrfIdentity = () => web3.PublicKey.findProgramAddressSync([Buffer.from('identity'), PROGRAM_ID.toBuffer()], VRF)[0];
const u32 = n => { const b = Buffer.alloc(4); b.writeUInt32LE(n >>> 0); return b; };
const u64 = n => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; };
const k = (pubkey, isSigner, isWritable) => ({ pubkey, isSigner, isWritable });
const ix = (keys, data) => new web3.TransactionInstruction({ programId: PROGRAM_ID, keys, data: Buffer.from(data) });
const nameBytes = s => { const b = Buffer.from(String(s), 'utf8').subarray(0, 15); return Buffer.concat([Buffer.from([b.length]), b]); };

const I = {
  initTable: (op, i, lamports) => ix([k(op, true, true), k(tablePda(i), false, true), k(web3.SystemProgram.programId, false, false)], Buffer.concat([Buffer.from([0, i]), u64(lamports)])),
  delegate: (op, i, commitMs = 0xffffffff) => {
    const pda = tablePda(i);
    const buffer = web3.PublicKey.findProgramAddressSync([Buffer.from('buffer'), pda.toBuffer()], PROGRAM_ID)[0];
    const record = web3.PublicKey.findProgramAddressSync([Buffer.from('delegation'), pda.toBuffer()], DELEGATION)[0];
    const metadata = web3.PublicKey.findProgramAddressSync([Buffer.from('delegation-metadata'), pda.toBuffer()], DELEGATION)[0];
    return ix([k(op, true, true), k(pda, false, true), k(PROGRAM_ID, false, false), k(buffer, false, true), k(record, false, true), k(metadata, false, true), k(DELEGATION, false, false), k(web3.SystemProgram.programId, false, false)], Buffer.concat([Buffer.from([1, i]), u32(commitMs)]));
  },
  create: (p, i, session, name) => ix([k(p, true, true), k(tablePda(i), false, true)], Buffer.concat([Buffer.from([2, i]), session.toBuffer(), nameBytes(name)])),
  join: (p, i, match, session, name) => ix([k(p, true, true), k(tablePda(i), false, true)], Buffer.concat([Buffer.from([3, i]), u32(match), session.toBuffer(), nameBytes(name)])),
  rebind: (p, i, match, seat, session) => ix([k(p, true, true), k(tablePda(i), false, true)], Buffer.concat([Buffer.from([9, i]), u32(match), Buffer.from([seat]), session.toBuffer()])),
  start: (p, i, match) => ix([k(p, true, true), k(tablePda(i), false, true)], Buffer.concat([Buffer.from([4, i]), u32(match)])),
  roll: (p, i, match, seat, seq) => ix([k(p, true, true), k(tablePda(i), false, true), k(identityPda(), false, false), k(VRF, false, false), k(QUEUE, false, true), k(web3.SystemProgram.programId, false, false), k(SLOT_HASHES, false, false)], Buffer.concat([Buffer.from([5, i]), u32(match), Buffer.from([seat]), u32(seq)])),
  move: (p, i, match, seat, seq, token, from) => ix([k(p, true, true), k(tablePda(i), false, true)], Buffer.concat([Buffer.from([6, i]), u32(match), Buffer.from([seat]), u32(seq), Buffer.from([token, from])])),
  leave: (p, i, match, seat) => ix([k(p, true, true), k(tablePda(i), false, true)], Buffer.concat([Buffer.from([7, i]), u32(match), Buffer.from([seat])])),
  commitUndelegate: (op, i) => ix([k(op, true, true), k(tablePda(i), false, true), k(MAGIC, false, false), k(MAGIC_CONTEXT, false, true)], [8, i]),
};

function parseTable(data) {
  const d = Buffer.from(data);
  const seats = [0, 1, 2, 3].map(s => {
    const o = T_SEATS + s * SEAT;
    return {
      key: new web3.PublicKey(d.subarray(o, o + 32)).toBase58(), session: new web3.PublicKey(d.subarray(o + 32, o + 64)).toBase58(),
      name: d.subarray(o + 65, o + 65 + d[o + 64]).toString('utf8'),
      pieces: [...d.subarray(o + 80, o + 84)].map(v => (v === 255 ? -1 : v)), wait: d[o + 84], pending: d[o + 85], sixes: d[o + 86], bonus: d[o + 87],
      last: d[o + 88], captures: d[o + 89], seated: !!(d[o + 90] & 1), left: !!(d[o + 90] & 2), home: d[o + 91],
      seq: d.readUInt32LE(o + 92), req: d.readUInt32LE(o + 96), reqAt: d.readUInt32LE(o + 100),
    };
  });
  const count = d.readUInt32LE(32);
  const events = [];
  for (let n = Math.max(0, count - RING); n < count; n++) {
    const e = d.subarray(T_RING + (n % RING) * 8, T_RING + (n % RING) * 8 + 8);
    events.push({ n, kind: EVENTS[e[0]] || e[0], seat: e[1], a: e[2], b: e[3], c: e[4], d: e[5], e: e[6], f: e[7] });
  }
  return { magic: d.subarray(0, 8).toString(), idx: d[8], phase: PHASES[d[9]], seatedMask: d[10], winner: d[11] === 255 ? null : d[11], match: d.readUInt32LE(12),
    created: Number(d.readBigInt64LE(16)), active: Number(d.readBigInt64LE(24)), eventCount: count, seats, events };
}

function errorCode(e) {
  const m = /custom program error: 0x([0-9a-f]+)/i.exec(String(e && (e.message || e))) || /"Custom":(\d+)/.exec(JSON.stringify(e && e.err || {}));
  return m ? (m[0].startsWith('custom') ? parseInt(m[1], 16) : Number(m[1])) : null;
}

// One transaction, fee payer = first signer; confirms at 'confirmed'. Errors carry .code (program error) and .logs.
async function send(conn, ixs, signers, opts = {}) {
  const tx = new web3.Transaction().add(...ixs);
  tx.feePayer = signers[0].publicKey;
  const bh = await conn.getLatestBlockhash();
  tx.recentBlockhash = bh.blockhash;
  tx.sign(...signers);
  let sig;
  try { sig = await conn.sendRawTransaction(tx.serialize(), { skipPreflight: opts.skipPreflight !== false }); }
  catch (e) { const err = new Error(String(e.message || e).split('\n')[0]); err.code = errorCode(e); err.logs = e.logs; throw err; }
  const r = await conn.confirmTransaction({ signature: sig, ...bh }, 'confirmed');
  if (r.value.err) {
    const t = await conn.getTransaction(sig, { commitment: 'confirmed', maxSupportedTransactionVersion: 0 }).catch(() => null);
    const err = new Error('tx failed ' + JSON.stringify(r.value.err)); err.sig = sig; err.code = errorCode({ err: r.value.err }); err.logs = t?.meta?.logMessages; throw err;
  }
  return sig;
}

module.exports = { web3, PROGRAM_ID, DELEGATION, MAGIC, MAGIC_CONTEXT, VRF, QUEUE, SLOT_HASHES, BASE_URL, ER_URL, TABLE_SIZE, NTABLES, PHASES, EVENTS, ERRORS, tablePda, identityPda, vrfIdentity, I, parseTable, send, errorCode };
