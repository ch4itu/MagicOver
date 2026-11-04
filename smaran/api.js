/* smaran api */
// Smaran: sign up for a Solana app with a player name and a passphrase. No wallet, no seed phrase, no SOL.
// Exposes globalThis.Smaran. Everything below it in smaran.js (Solana web3.js, the ChainKey crypto, TweetNaCl and
// the registry client) is copied unchanged from Magic Over's index.html by build.py.
globalThis.Smaran = (() => {
  'use strict';
  const S = globalThis.SabhaSol, C = globalThis.SabhaChainKey, nacl = globalThis.nacl, W = globalThis.solanaWeb3;
  if (!S || !C || !nacl || !W) throw new Error('Smaran: a bundled library is missing. Load smaran.js once, as one file.');
  const where = { erUrl: S.DEFAULT_ER, baseUrl: S.DEFAULT_BASE };

  // The signed-in player. secretKey is the usual 64-byte Solana secret key (seed, then public key). It lives only in
  // this page's memory; forget() overwrites it.
  function player(account, name) {
    let gone = false;
    const live = () => { if (gone) throw new Error('Smaran: this player was signed out with forget().'); };
    return Object.freeze({
      name,
      publicKey: account.addr,
      get secretKey() { live(); return account.sk; },
      sign(message) {
        live();
        const bytes = typeof message === 'string' ? C.te.encode(message) : new Uint8Array(message);
        return nacl.sign.detached(bytes, account.sk);
      },
      keypair() { live(); return W.Keypair.fromSecretKey(account.sk); },
      forget() { if (!gone) { gone = true; C.wipeAccount(account); } }
    });
  }

  // Creates the player: a random key, locked with the passphrase in this browser, stored in the registry on
  // MagicBlock's rollup. onProgress gets { phase: 'encrypt' | 'proof' | 'send', attempts }.
  async function signUp({ name, passphrase, onProgress } = {}) {
    const made = await S.createPlayer({ name, password: passphrase, ...where, onProgress });
    // ask the rollup to save the registry to Solana; sign-up does not wait for it
    S.commitShard({ account: made.account, shard: made.shard, erUrl: where.erUrl }).catch(() => {});
    return player(made.account, made.name);
  }

  // Brings the same player back on any device: reads the locked copy and unlocks it in this browser.
  async function signIn({ name, passphrase } = {}) {
    const back = await S.restorePlayer({ name, password: passphrase, ...where });
    return player(back.account, back.name);
  }

  // Locks the same player key with a new passphrase. The current passphrase is needed. The player key does not change,
  // so the player is the same everywhere; every app on the registry then asks for the new passphrase.
  // onProgress gets { phase: 'unlock' | 'encrypt' | 'send' }.
  async function changePassphrase({ name, passphrase, newPassphrase, onProgress } = {}) {
    const done = await S.changePassword({ name, password: passphrase, newPassword: newPassphrase, ...where, onProgress });
    S.commitShard({ account: done.account, shard: done.shard, erUrl: where.erUrl }).catch(() => {});
    return player(done.account, done.name);
  }

  async function lookup(name) {
    const found = await S.lookup(name, where);
    return {
      name: found.normalized,
      exists: !!found.entry,
      publicKey: found.entry ? S.b58encode(found.entry.owner) : null,
      step: found.entry ? found.gen : null,
      from: found.source
    };
  }

  async function status() {
    const s = await S.registryStatus(where);
    return { players: s.players, capacity: s.capacity, steps: s.steps, rollupOk: s.erOk, solanaOk: s.baseOk };
  }

  return Object.freeze({
    version: '1.1.0',
    network: 'devnet',
    registry: S.PROGRAM_ID,
    rollup: where.erUrl,
    signUp,
    signIn,
    changePassphrase,
    lookup,
    status,
    checkName: name => C.normalizeName(name), // the stored form of a name; throws with the rule if it is not allowed
    checkPassphrase: (passphrase, name = '') => C.validatePassword(passphrase, name) // { ok, errors }
  });
})();
