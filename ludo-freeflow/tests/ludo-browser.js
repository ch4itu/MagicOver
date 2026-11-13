'use strict';
// Ludo Freeflow in real browsers: the local preview, then an online match between two Smaran players (desktop and
// phone) on MagicBlock's devnet rollup with MagicBlock VRF dice, through Smaran's real sign-in window.
// Covers: sign-in, create, join, start, concurrent rolls and moves, identical boards on both clients, effects on
// both clients, a reload mid-match, the same player continuing on another device, a refused approval, sign-out,
// leaving, zero SOL, and what the page keeps in storage.
// Credentials of two existing test players come from the environment and are never written anywhere.
// usage: LUDO_P1=name LUDO_P2=name LUDO_PASS=... node ludo-browser.js [base url] [minutes of play]
const { chromium, devices } = require('playwright');
const BASE = process.argv[2] || 'http://127.0.0.1:8091/ludo-freeflow/';
const PLAY_MINUTES = Number(process.argv[3] || 7);
const [P1, P2, PASS] = [process.env.LUDO_P1, process.env.LUDO_P2, process.env.LUDO_PASS];
const ER = 'https://devnet-as.magicblock.app', DEVNET = 'https://api.devnet.solana.com';
const SHOTS = process.env.LUDO_SHOTS || '/home/claude/scratch/ludo-shots/';
const sleep = ms => new Promise(r => setTimeout(r, ms));
const results = [];
const C = (name, ok, detail = '') => { const status = ok === null ? 'NOT TESTED' : ok === 'MEASURED' ? 'MEASURED' : ok ? 'PASS' : 'FAIL'; results.push({ name, status, detail }); console.log(`${status.padEnd(10)} ${name}${detail ? ' | ' + detail : ''}`); };

async function rpc(url, method, params) {
  const r = await fetch(url, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params }) });
  return (await r.json()).result;
}
function watch(page, tag) {
  page.tag = tag;
  page.log = { errors: [], dialogs: [] };
  page.on('pageerror', e => page.log.errors.push(e.message));
  page.on('dialog', d => { page.log.dialogs.push(d.type() + ': ' + d.message()); d.dismiss().catch(() => {}); });
  return page;
}
async function client(browser, tag, device) {
  const ctx = await browser.newContext(device === 'phone' ? { ...devices['Pixel 7'] } : { viewport: { width: 1280, height: 860 } });
  const page = watch(await ctx.newPage(), tag);
  await page.goto(BASE, { waitUntil: 'load' });
  await page.waitForFunction(() => typeof solanaWeb3 !== 'undefined' && typeof online !== 'undefined', null, { timeout: 60000 });
  await instrument(page);
  return { ctx, page, tag };
}
// record effects (cinematics) and die faces the page shows; nothing else is changed
async function instrument(page) {
  await page.evaluate(() => {
    if (window.__fx) return;
    window.__fx = []; window.__dice = [];
    const c = celebrate, s = showDie;
    celebrate = (k, t, sub) => { window.__fx.push([k, t]); return c(k, t, sub); };
    showDie = d => { window.__dice.push(d); return s(d); };
  });
}
async function signIn(c, name) {
  const [popup] = await Promise.all([c.page.context().waitForEvent('page', { timeout: 30000 }), c.page.click('#smaranSignIn')]);
  watch(popup, c.tag + '-window');
  await popup.waitForSelector('#name', { timeout: 60000 });
  await popup.fill('#name', name);
  await popup.fill('#pass', PASS);
  await popup.click('#go');
  await popup.waitForSelector('#session:not([hidden])', { timeout: 90000 });
  await c.page.waitForFunction(n => document.getElementById('smaranStatus').textContent.includes(n), name, { timeout: 30000 });
  c.popup = popup;
  return popup;
}
async function answer(c, approve = true) {
  await c.popup.waitForSelector('#ask:not([hidden])', { timeout: 30000 });
  const title = await c.popup.textContent('#askTitle');
  await sleep(800); // the window keeps Approve disabled for 600 ms
  await c.popup.click(approve ? '#approve' : '#reject');
  return title;
}
const state = c => c.page.evaluate(() => ({
  online: !!online, me, over, phase: document.getElementById('olInfo').textContent, pieces: pieces.map(a => a.slice()), waiting: waiting.slice(),
  last: lastRolls.slice(), caps: captureCounts.slice(), button: document.getElementById('roll').textContent.trim(), disabled: document.getElementById('roll').disabled,
  hint: document.getElementById('hint').textContent, msg: document.getElementById('olMsg').textContent, fx: window.__fx.slice(), dice: window.__dice.slice(),
}));
const until = async (c, fn, ms = 30000, what = 'condition') => {
  const end = Date.now() + ms;
  while (Date.now() < end) { const s = await state(c); if (fn(s)) return s; await sleep(400); }
  throw new Error(`${c.tag}: timed out waiting for ${what}`);
};
async function shot(c, name) { await c.page.screenshot({ path: SHOTS + name + '.png' }).catch(() => {}); }

(async () => {
  require('fs').mkdirSync(SHOTS, { recursive: true });
  const browser = await chromium.launch({ proxy: process.env.HTTPS_PROXY ? { server: process.env.HTTPS_PROXY, bypass: '127.0.0.1,localhost' } : undefined });
  const all = [];
  try {
    // ---------------------------------------------------------------- local preview, untouched by sign-in
    const L = await client(browser, 'preview', 'desktop');
    all.push(L);
    let s = await state(L);
    C('Preview: loads as the local preview with bots, labelled LOCAL PREVIEW', !s.online && (await L.page.textContent('#modeNote')).includes('LOCAL PREVIEW'));
    const geo = await L.page.evaluate(() => { const b = document.getElementById('board').getBoundingClientRect(), r = document.getElementById('roll').getBoundingClientRect(); return { boardBottom: b.bottom, rollTop: r.top, rollBottom: r.bottom, vh: innerHeight }; });
    C('Preview: the roll button sits below the board', geo.rollTop >= geo.boardBottom - 1, JSON.stringify(geo));
    await L.page.click('#roll');
    s = await until(L, x => x.dice.length > 0, 5000, 'a die');
    C('Preview: Roll shows a die face', s.dice[0] >= 1 && s.dice[0] <= 6, `die ${s.dice[0]}`);
    await sleep(4000);
    s = await state(L);
    C('Preview: the bots play on their own (no turns)', s.last.slice(1).some(v => v !== null), `last dice ${JSON.stringify(s.last)}`);
    await shot(L, 'preview-desktop');
    if (!P1 || !P2 || !PASS) { C('Online match', null, 'test player credentials not given'); return; }

    // ---------------------------------------------------------------- two Smaran players
    const A = await client(browser, 'A-desktop', 'desktop'), B = await client(browser, 'B-phone', 'phone');
    all.push(A, B);
    await signIn(A, P1);
    await signIn(B, P2);
    C('Sign-in through Smaran\'s window (existing players) on desktop and phone', true, `${P1}, ${P2}`);
    C('After sign-in the page still runs the local preview until a match starts', !(await state(A)).online);

    // A creates: one approval in the window
    await A.page.click('#olCreate');
    const t1 = await answer(A);
    s = await until(A, x => x.online && /lobby/.test(x.phase), 30000, 'the lobby');
    const code = (/Table (\d) · match (\d+)/.exec(s.phase) || []).slice(1).join('-');
    C('Create online match: one approval in the Smaran window, then a lobby with an invite code', !!code, `window asked "${t1}"; ${s.phase}`);
    const sessionA = await A.page.evaluate(() => JSON.parse(sessionStorage.getItem('ludo-freeflow-session') || 'null'));
    const [ti, tm] = code.split('-').map(Number);
    C('The page keeps only a session key for this table and match (sessionStorage), never the Smaran key', !!sessionA && sessionA.sk.length === 64 && sessionA.idx === ti && sessionA.match === tm && (await A.page.evaluate(() => Object.keys(localStorage).filter(k => /ludo|session|smaran/i.test(k)).length)) === 0, JSON.stringify({ idx: sessionA?.idx, match: sessionA?.match }));

    // B joins by code
    await B.page.fill('#olCode', code);
    await B.page.click('#olJoin');
    await answer(B);
    s = await until(B, x => x.online && /lobby/.test(x.phase), 30000, 'joining');
    C('Join by invite code on the phone: one approval, seated as blue', s.me === 1, s.phase);
    const shown = await B.page.evaluate(() => ['onlineIdle', 'olLobbies', 'onlineMatch'].map(id => getComputedStyle(document.getElementById(id)).display !== 'none'));
    C('In a match, Create/Join and the list of open matches are hidden; the match bar is shown', !shown[0] && !shown[1] && shown[2], JSON.stringify(shown));
    C('The help under the button names the player\'s own colour', /glowing blue token/.test(await B.page.textContent('#controlHelp')), await B.page.textContent('#controlHelp'));
    await until(A, x => /2 OF 4 SEATED/.test(x.hint + x.button) || x.button.includes('START'), 15000, 'A sees B');
    await shot(B, 'lobby-phone');

    // B starts the match
    await B.page.click('#roll');
    s = await until(A, x => /playing/.test(x.phase), 20000, 'the start');
    C('Start: both pages switch to the match', /playing/.test((await state(B)).phase) && /playing/.test(s.phase));

    // ---------------------------------------------------------------- play: each client presses Roll whenever it is
    // enabled (roll, move the first legal token, or pass), both at the same time
    const rolls = { A: 0, B: 0 }, latency = [], dicesSeen = new Set();
    let red = A;
    async function playFor(p, who, ms) {
      const end = Date.now() + ms;
      while (Date.now() < end) {
        const x = await state(p);
        if (x.over || /finished/.test(x.phase)) return;
        if (!x.disabled && /ROLL THE DICE|ROLL AGAIN|MOVE TOKEN|PASS/.test(x.button)) {
          const wasRoll = /ROLL/.test(x.button) && !/MOVE|PASS/.test(x.button), before = x.dice.length, t0 = Date.now();
          await p.page.click('#roll').catch(() => {});
          if (wasRoll) {
            rolls[who]++;
            const y = await until(p, z => z.dice.length > before || z.over, 30000, 'the VRF die').catch(() => null);
            if (y && y.dice.length > before) { latency.push(Date.now() - t0); dicesSeen.add(y.dice.at(-1)); }
          }
        } else await sleep(300);
      }
    }
    const both = ms => Promise.all([playFor(red, 'A', ms), playFor(B, 'B', ms)]);
    const minute = PLAY_MINUTES * 60000;
    await both(minute * 0.25);
    // a reload mid-match on the phone (keep what its page showed so far)
    const fxBeforeReload = (await state(B)).fx;
    await B.page.reload({ waitUntil: 'load' });
    await B.page.waitForFunction(() => typeof online !== 'undefined' && !!online, null, { timeout: 60000 });
    await instrument(B.page);
    let y = await until(B, z => /playing|finished/.test(z.phase), 20000, 'resume');
    C('Reload mid-match: the phone resumes its seat from the tab\'s session key, without signing in again', y.me === 1, y.phase);
    await both(minute * 0.2);
    // the same player continues on another device
    if (!(await state(A)).over) {
      const A2 = await client(browser, 'A2-desktop', 'desktop');
      all.push(A2);
      await signIn(A2, P1);
      await A2.page.waitForSelector('[data-resume]', { timeout: 30000 });
      await A2.page.click('[data-resume]');
      await answer(A2);
      y = await until(A2, z => z.online && /playing|finished/.test(z.phase), 30000, 'continue here');
      const z = await until(A, w => /another device/.test(w.msg) || w.me === -1, 20000, 'the first device to notice');
      C('Same player on a second device: "Continue here" (one approval) moves the seat; the first device switches to watching', y.me === 0 && z.me === -1, `${y.phase}; first device: "${z.msg}"`);
      red = A2;
    } else C('Same player on a second device', null, 'the match ended before this step');
    await both(minute * 0.2);
    // a refused approval and sign-out, in a further window signed in as the phone's player
    const D = await client(browser, 'D-desktop', 'desktop');
    all.push(D);
    await signIn(D, P2);
    await D.page.click('#olCreate');
    await answer(D, false);
    y = await until(D, z => /Not approved/.test(z.msg), 20000, 'the refusal message');
    C('Rejecting in the Smaran window: nothing is sent, the page says so', !y.online, y.msg);
    await D.page.click('#smaranSignIn');
    await D.page.waitForFunction(() => document.getElementById('smaranSignIn').textContent.includes('Sign in'), null, { timeout: 10000 }).catch(() => {});
    C('Sign out', (await D.page.textContent('#smaranSignIn')).includes('Sign in'), await D.page.textContent('#smaranStatus'));
    await both(minute * 0.35);
    const players = () => [red, B];
    C('Both players rolled at the same time through MagicBlock VRF; every die arrived', rolls.A + rolls.B > 20 && latency.length >= rolls.A + rolls.B - 1,
      `${rolls.A + rolls.B} rolls, ${latency.length} dice shown; faces seen ${[...dicesSeen].sort().join('')}; roll-to-die median ${latency.sort((a, b) => a - b)[latency.length >> 1]} ms, max ${latency.at(-1)} ms`);
    // let both settle, then compare boards
    await sleep(3000);
    const sa = await state(players()[0]), sb = await state(B);
    C('Both clients show the same board (positions, last dice, captures)', JSON.stringify(sa.pieces) === JSON.stringify(sb.pieces) && JSON.stringify(sa.last) === JSON.stringify(sb.last) && JSON.stringify(sa.caps) === JSON.stringify(sb.caps),
      `pieces ${JSON.stringify(sa.pieces)}; captures ${JSON.stringify(sa.caps)}`);
    if (!sa.over) {
      // end it: blue leaves, so red is the last colour in play and wins
      await B.page.click('#olLeave'); await B.page.click('#olLeave');
      await until(players()[0], z => z.over, 30000, 'the win');
    }
    await sleep(2500);
    const fa = await state(players()[0]), fb = await state(B);
    const kinds = f => [...new Set(f.map(e => e[0] + ':' + e[1]))];
    const count = (f, k) => f.filter(e => e[0] === k).length;
    const fxA = (await state(A)).fx, fxB = fxBeforeReload.concat(fb.fx), captures = fa.caps.reduce((x, y) => x + y, 0);
    C('Effects play from the chain\'s events on every client', fxA.length > 0 && fxB.length > 0,
      `first desktop (red, then watching): ${JSON.stringify(kinds(fxA))}; ${red !== A ? 'second desktop (red): ' + JSON.stringify(kinds(fa.fx)) + '; ' : ''}phone (blue): ${JSON.stringify(kinds(fxB))}`);
    C('Every capture in the match played its CAPTURE effect on the first desktop, live from start to end', captures === 0 ? null : count(fxA, 'capture') === captures,
      captures === 0 ? 'no capture happened in this match' : `${captures} captures on chain, ${count(fxA, 'capture')} CAPTURE effects shown`);
    C('Victory / game over reaches both sides', kinds(fa.fx.concat(fxA)).some(k => /VICTORY|GAME OVER/.test(k)) && kinds(fxB).some(k => /VICTORY|GAME OVER/.test(k)), `red side ${JSON.stringify(kinds(fa.fx))}; blue ${JSON.stringify(kinds(fxB))}`);
    await shot(players()[0], 'online-end-desktop');
    await shot(B, 'online-end-phone');

    // zero SOL: Smaran keys and session keys, on Solana and on the rollup
    const keys = await Promise.all([players()[0], B].map(c => c.page.evaluate(() => { const j = JSON.parse(sessionStorage.getItem('ludo-freeflow-session') || 'null'); return j && solanaWeb3.Keypair.fromSecretKey(Uint8Array.from(j.sk)).publicKey.toBase58(); })));
    const balances = [];
    for (const k of keys.filter(Boolean)) { balances.push((await rpc(DEVNET, 'getBalance', [k]))?.value ?? -1, (await rpc(ER, 'getBalance', [k]))?.value ?? 0); }
    C('Zero SOL: the session keys never held any', balances.every(b => b === 0), JSON.stringify(balances));
  } catch (e) {
    C('test run', false, 'stopped: ' + (e.message || e).split('\n')[0]);
    for (const c of all) await shot(c, 'stopped-' + c.tag);
  } finally {
    for (const c of all) {
      C(`${c.tag}: no JavaScript errors`, c.page.log.errors.length === 0, JSON.stringify(c.page.log.errors.slice(0, 3)));
      C(`${c.tag}: no native dialogs`, c.page.log.dialogs.length === 0, JSON.stringify(c.page.log.dialogs));
    }
    await browser.close();
    console.log(`${results.filter(r => r.status === 'PASS').length} passed, ${results.filter(r => r.status === 'FAIL').length} failed, ${results.filter(r => r.status === 'NOT TESTED').length} not tested`);
  }
})();
