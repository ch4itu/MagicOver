#!/usr/bin/env node
// Smaran save fund: the refill script. Anyone can run it, from any computer, as often as they like.
//
// It looks at every account the save fund may pay for. An account on a MagicBlock rollup whose save reserve is low
// (fewer than ten paid saves) is handed back to Solana. An account on Solana that is low is topped up by the save fund,
// and every account found on Solana is put back on its rollup with the same settings it had. Without --apply it only
// reports. Nothing here can move the fund's SOL anywhere but into these accounts: the save fund program decides that.
//
// Needs: Node 18 or later, `npm install` in this folder, and a Solana keypair file with a little SOL for fees. Each
// Solana transaction costs about 0.000005 SOL, and putting an account back on a rollup costs MagicBlock's delegation
// deposit, about 0.003 SOL. Transactions on the rollups are free.
//
//   node refill.js                         report only
//   node refill.js --apply --keypair FILE  hand back, refill and delegate where needed
//   node refill.js --create-fund --keypair FILE   create the save fund account (once, on a new cluster)
//   --only TEXT                            only accounts whose name contains TEXT
'use strict';
const fs = require('fs');
const nacl = require('tweetnacl');
const bs58m = require('bs58');
const bs58 = bs58m.default || bs58m;
const {
  Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction,
} = require('@solana/web3.js');

const BASE = process.env.SOLANA_RPC || 'https://api.devnet.solana.com';
const AS = 'https://devnet-as.magicblock.app';
const TEE = 'https://devnet-tee.magicblock.app';
const VALIDATOR = { [AS]: 'MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57', [TEE]: 'MTEWGuqxUpYZGFJQcp8tLN7x5v9BSeoFHYWQQ3n3xzo' };

const FUND_PROGRAM = new PublicKey('PRU4HhX5x3Kf5wpTCt7amrjTbwXZ88NxU3DwwnHrcuo');
const FUND = PublicKey.findProgramAddressSync([Buffer.from('smaran-save-fund')], FUND_PROGRAM)[0];
const DELEGATION = new PublicKey('DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh');
const MAGIC_PROGRAM = new PublicKey('Magic11111111111111111111111111111111111111');
const MAGIC_CONTEXT = new PublicKey('MagicContext1111111111111111111111111111111');
const REGISTRY = new PublicKey('CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P');
const PLAY = new PublicKey('HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B');
const STREAKS = new PublicKey('A98As1brXWi2DuJFChgat6bL9XbmaRRiF5vMFZswALoi');
const BOARD = new PublicKey('BcS2Gb1Gihfzqs1UkViNnbQzjGcCGyvXu81VhwsFrUzU');
const ETERNAL = new PublicKey('6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb');

const SAVE_FEE = 100000;
const LOW = 10 * SAVE_FEE;
const NO_TIMED_COMMITS = 0xffffffff;

const u32 = (v) => { const b = Buffer.alloc(4); b.writeUInt32LE(v >>> 0); return b; };
const seedBytes = (s) => (typeof s === 'string' ? Buffer.from(s) : Buffer.from(s));
const pda = (seeds, program) => PublicKey.findProgramAddressSync(seeds.map(seedBytes), program)[0];
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// Every account the save fund may pay for: where it lives, the fund's list entry, and how its own program hands it back
// to Solana and delegates it again (the arguments anyone may use).
function accounts() {
  const list = [];
  const add = (name, program, seeds, er, entry, a, b, handBack, delegate) =>
    list.push({ name, program, address: pda(seeds, program), er, entry, a, b, handBack: Buffer.from(handBack), delegate: Buffer.from(delegate) });
  const mas1 = new PublicKey(VALIDATOR[AS]).toBuffer();
  const tee = new PublicKey(VALIDATOR[TEE]).toBuffer();
  for (let i = 0; i < 4; i++) {
    add(`registry ${i}`, REGISTRY, ['sabha-reg', [i]], AS, 0, i, 0, [5], Buffer.concat([Buffer.from([3, i, 1]), mas1, u32(NO_TIMED_COMMITS)]));
  }
  for (let g = 1; g <= 5; g++) {
    for (let i = 0; i < 4 << g; i++) {
      add(`registry ${g}.${i}`, REGISTRY, ['sabha-reg', [g], [i]], AS, 1, g, i, [5], Buffer.concat([Buffer.from([3, i, 1]), mas1, u32(NO_TIMED_COMMITS)]));
    }
  }
  add('relic register', PLAY, ['play-reg'], TEE, 2, 0, 0, [3, 0, 0], Buffer.concat([Buffer.from([1, 0, 0, 1]), tee, u32(30000)]));
  for (let i = 0; i < 4; i++) {
    add(`achievements ${i}`, PLAY, ['play-ach', [i]], TEE, 3, i, 0, [3, 1, i], Buffer.concat([Buffer.from([1, 1, i, 1]), tee, u32(30000)]));
  }
  for (let i = 0; i < 4; i++) {
    add(`relics ${i}`, PLAY, ['play-rel', [i]], TEE, 4, i, 0, [3, 2, i], Buffer.concat([Buffer.from([1, 2, i, 1]), tee, u32(30000)]));
  }
  for (let i = 0; i < 8; i++) {
    add(`streaks ${i}`, STREAKS, ['smaran-streaks', [i]], AS, 5, i, 0, [8, i], Buffer.concat([Buffer.from([1, i]), u32(NO_TIMED_COMMITS)]));
  }
  for (let i = 0; i < 2; i++) {
    add(`leaderboard ${i}`, BOARD, ['mo-board', [i]], TEE, 6, i, 0, [5, i], Buffer.concat([Buffer.from([1, i, 0, 1]), tee, u32(NO_TIMED_COMMITS)]));
  }
  for (let i = 0; i < 8; i++) {
    add(`eternalbliss world ${i}`, ETERNAL, ['eb-land', [i]], AS, 7, i, 0, [30], [17, i]);
  }
  for (let i = 0; i < 8; i++) {
    add(`eternalbliss book ${i}`, ETERNAL, ['eb-book', [i]], AS, 8, i, 0, [30], [41, i]);
  }
  return list;
}

function arg(name) {
  const i = process.argv.indexOf(name);
  return i < 0 ? null : process.argv[i + 1] || null;
}
const has = (name) => process.argv.includes(name);

async function teeUrl(signer) {
  const res = await fetch(`${TEE}/auth/challenge?pubkey=${signer.publicKey.toBase58()}`);
  const { challenge } = await res.json();
  if (!challenge) throw new Error('the private rollup gave no sign-in challenge');
  const signature = bs58.encode(nacl.sign.detached(Buffer.from(challenge, 'utf8'), signer.secretKey));
  const login = await fetch(`${TEE}/auth/login`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ pubkey: signer.publicKey.toBase58(), challenge, signature }) });
  const json = await login.json();
  if (!json.token) throw new Error('private rollup sign-in failed: ' + (json.error || login.status));
  return `${TEE}?token=${encodeURIComponent(json.token)}`;
}

async function checkIdentity(conn, want, label) {
  const res = await fetch(conn.rpcEndpoint, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'getIdentity' }) });
  const id = (await res.json()).result?.identity;
  if (id !== want) throw new Error(`${label} is not validator ${want} (it says ${id})`);
}

async function send(conn, signer, ixs, label) {
  const tx = new Transaction().add(...ixs);
  tx.feePayer = signer.publicKey;
  const { blockhash } = await conn.getLatestBlockhash('confirmed');
  tx.recentBlockhash = blockhash;
  tx.sign(signer);
  const sig = await conn.sendRawTransaction(tx.serialize(), { skipPreflight: false, preflightCommitment: 'confirmed' });
  for (let i = 0; i < 60; i++) {
    const st = (await conn.getSignatureStatuses([sig])).value[0];
    if (st?.err) throw new Error(`${label} failed: ${JSON.stringify(st.err)} (${sig})`);
    if (st && (st.confirmationStatus === 'confirmed' || st.confirmationStatus === 'finalized')) return sig;
    await sleep(1000);
  }
  throw new Error(`${label}: not confirmed in 60 s (${sig})`);
}

function refillIx(acct) {
  return new TransactionInstruction({
    programId: FUND_PROGRAM,
    keys: [{ pubkey: FUND, isSigner: false, isWritable: true }, { pubkey: acct.address, isSigner: false, isWritable: true }],
    data: Buffer.from([1, acct.entry, acct.a, acct.b]),
  });
}

function delegateIx(acct, payer) {
  return new TransactionInstruction({
    programId: acct.program,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: acct.address, isSigner: false, isWritable: true },
      { pubkey: acct.program, isSigner: false, isWritable: false },
      { pubkey: pda(['buffer', acct.address.toBuffer()], acct.program), isSigner: false, isWritable: true },
      { pubkey: pda(['delegation', acct.address.toBuffer()], DELEGATION), isSigner: false, isWritable: true },
      { pubkey: pda(['delegation-metadata', acct.address.toBuffer()], DELEGATION), isSigner: false, isWritable: true },
      { pubkey: DELEGATION, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: acct.delegate,
  });
}

function handBackIx(acct, caller) {
  return new TransactionInstruction({
    programId: acct.program,
    keys: [
      { pubkey: caller, isSigner: true, isWritable: true },
      { pubkey: acct.address, isSigner: false, isWritable: true },
      { pubkey: MAGIC_PROGRAM, isSigner: false, isWritable: false },
      { pubkey: MAGIC_CONTEXT, isSigner: false, isWritable: true },
    ],
    data: acct.handBack,
  });
}

async function main() {
  const apply = has('--apply');
  const keyfile = arg('--keypair');
  const signer = keyfile ? Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(keyfile, 'utf8')))) : Keypair.generate();
  if ((apply || has('--create-fund')) && !keyfile) throw new Error('--apply and --create-fund need --keypair FILE');
  const base = new Connection(BASE, 'confirmed');

  if (has('--create-fund')) {
    const ix = new TransactionInstruction({
      programId: FUND_PROGRAM,
      keys: [{ pubkey: signer.publicKey, isSigner: true, isWritable: true }, { pubkey: FUND, isSigner: false, isWritable: true }, { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }],
      data: Buffer.from([0]),
    });
    console.log('save fund created:', FUND.toBase58(), await send(base, signer, [ix], 'create fund'));
    return;
  }

  const ers = { [AS]: new Connection(AS, 'confirmed'), [TEE]: new Connection(await teeUrl(signer), 'confirmed') };
  await checkIdentity(ers[AS], VALIDATOR[AS], 'the public rollup');
  await checkIdentity(ers[TEE], VALIDATOR[TEE], 'the private rollup');

  const fundInfo = await base.getAccountInfo(FUND);
  const fundRent = await base.getMinimumBalanceForRentExemption(64);
  if (!fundInfo || !fundInfo.owner.equals(FUND_PROGRAM)) throw new Error('the save fund does not exist yet: run with --create-fund');
  const d = fundInfo.data;
  const today = Math.floor(Date.now() / 86400000);
  const paidToday = Number(d.readBigUInt64LE(16)) === today ? Number(d.readBigUInt64LE(24)) : 0;
  console.log(`save fund ${FUND.toBase58()}: ${((fundInfo.lamports - fundRent) / 1e9).toFixed(6)} SOL to give, ${(paidToday / 1e9).toFixed(4)} of 0.1 SOL paid today, ${d.readUInt32LE(40)} refills in all`);

  const rents = {};
  const only = arg('--only');
  const list = accounts().filter((a) => !only || a.name.includes(only));
  const infos = [];
  for (let i = 0; i < list.length; i += 90) infos.push(...(await base.getMultipleAccountsInfo(list.slice(i, i + 90).map((a) => a.address))));
  for (let k = 0; k < list.length; k++) {
    const acct = list[k];
    let info = infos[k];
    if (!info) continue; // not created
    const len = info.data.length;
    const rent = rents[len] ?? (rents[len] = await base.getMinimumBalanceForRentExemption(len));
    const left = (lamports) => Math.floor((lamports - rent) / SAVE_FEE);
    try {
      if (info.owner.equals(DELEGATION)) {
        const rec = await base.getAccountInfo(pda(['delegation', acct.address.toBuffer()], DELEGATION));
        const validator = rec ? new PublicKey(rec.data.subarray(8, 40)).toBase58() : null;
        if (validator !== VALIDATOR[acct.er]) { console.log(`${acct.name}: delegated to ${validator}, not the expected rollup; skipped`); continue; }
        const onEr = await ers[acct.er].getAccountInfo(acct.address);
        if (!onEr) { console.log(`${acct.name}: delegated, but the rollup has no copy yet; skipped`); continue; }
        const low = onEr.lamports < rent + LOW;
        console.log(`${acct.name}: on the rollup, ${left(onEr.lamports)} paid saves left${low ? ' (low)' : ''}`);
        if (!low || !apply) continue;
        await send(ers[acct.er], signer, [handBackIx(acct, signer.publicKey)], `${acct.name} hand back`);
        process.stdout.write(`${acct.name}: handed back, waiting for it on Solana`);
        for (let i = 0; i < 90; i++) {
          await sleep(2000);
          info = await base.getAccountInfo(acct.address);
          if (info && info.owner.equals(acct.program)) break;
          process.stdout.write('.');
        }
        process.stdout.write('\n');
        if (!info || !info.owner.equals(acct.program)) { console.log(`${acct.name}: not back on Solana yet; run again later`); continue; }
      }
      if (info.owner.equals(acct.program)) {
        const low = info.lamports < rent + LOW;
        console.log(`${acct.name}: on Solana, ${left(info.lamports)} paid saves left${low ? ' (low)' : ''}`);
        if (!apply) continue;
        const ixs = low ? [refillIx(acct), delegateIx(acct, signer.publicKey)] : [delegateIx(acct, signer.publicKey)];
        const sig = await send(base, signer, ixs, `${acct.name} ${low ? 'refill and delegate' : 'delegate'}`);
        const after = await base.getAccountInfo(acct.address);
        console.log(`${acct.name}: ${low ? 'refilled and ' : ''}back on the rollup, ${left(after.lamports)} paid saves (${sig})`);
      }
    } catch (e) {
      console.log(`${acct.name}: ${e.message}`);
    }
  }
}

main().catch((e) => { console.error(e.message || e); process.exit(1); });
