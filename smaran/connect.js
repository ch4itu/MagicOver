/*! Smaran connect: players sign in with a name and a passphrase in Smaran's own window, so this site never sees the
 * passphrase or the key. https://github.com/ch4itu/MagicOver/tree/main/smaran (MIT). No dependencies.
 *
 *   button.onclick = async () => {
 *     const player = await SmaranConnect.connect();      // call it from a click, or the browser blocks the window
 *     player.name; player.publicKey;                       // the player name and Solana address (base58)
 *     await player.signMessage('text or bytes');           // the player approves it in the window; 64-byte signature
 *     await player.signTransaction(tx);                    // web3.js Transaction or VersionedTransaction: signed in place
 *     player.onDisconnect(reason => {});                   // the player signed out or closed the window
 *     player.disconnect();
 *   };
 */
globalThis.SmaranConnect = (() => {
  'use strict';
  const SIGNIN_URL = 'https://ch4itu.github.io/MagicOver/smaran/signin.html';
  let current = null; // one sign-in window at a time

  const fail = (message, code) => Object.assign(new Error('Smaran: ' + message), { code });

  function connect({ signinUrl = SIGNIN_URL } = {}) {
    const url = new URL(signinUrl, location.href);
    const origin = url.origin;
    if (current) current('replaced');
    const w = 420, h = 640;
    const left = Math.round(Math.max(0, (window.screenX || 0) + ((window.outerWidth || w) - w) / 2));
    const top = Math.round(Math.max(0, (window.screenY || 0) + 60));
    const popup = window.open(url.href, '_blank', `popup=yes,width=${w},height=${h},left=${left},top=${top}`);
    if (!popup) return Promise.reject(fail('the browser blocked the sign-in window. Call connect() from a click.', 'blocked'));

    return new Promise((resolve, reject) => {
      let player = null, done = false, seq = 0;
      const pending = new Map();
      const listeners = new Set();

      function request(type, body) {
        if (done) return Promise.reject(fail('the player is signed out.', 'signed-out'));
        const id = ++seq;
        return new Promise((res, rej) => {
          pending.set(id, { res, rej });
          popup.postMessage({ smaran: 1, type, id, ...body }, origin);
          try { popup.focus(); } catch (e) { /* the browser may not allow it */ }
        });
      }

      // what the window signs (message bytes), and how the signature goes back into the caller's object
      function prepare(tx) {
        if (tx instanceof Uint8Array) return { bytes: new Uint8Array(tx), attach: sig => sig };
        if (tx && typeof tx.serializeMessage === 'function') { // web3.js Transaction
          const bytes = new Uint8Array(tx.serializeMessage());
          return {
            bytes,
            attach: sig => {
              const pair = tx.signatures.find(p => p.publicKey.toBase58() === player.publicKey);
              if (!pair) throw fail('the transaction changed while it was being signed.', 'changed');
              tx.addSignature(pair.publicKey, sig);
              return tx;
            }
          };
        }
        if (tx && tx.message && typeof tx.message.serialize === 'function' && Array.isArray(tx.signatures)) { // VersionedTransaction
          const bytes = new Uint8Array(tx.message.serialize());
          return {
            bytes,
            attach: sig => {
              const signers = tx.message.staticAccountKeys.slice(0, tx.message.header.numRequiredSignatures);
              const i = signers.findIndex(k => k.toBase58() === player.publicKey);
              if (i < 0) throw fail('the transaction does not need this player\'s signature.', 'refused');
              tx.signatures[i] = sig;
              return tx;
            }
          };
        }
        throw new TypeError('Smaran: pass a web3.js Transaction or VersionedTransaction, or transaction message bytes.');
      }

      function end(reason, tell) {
        if (done) return;
        done = true;
        window.removeEventListener('message', onMessage);
        window.removeEventListener('pagehide', onLeave);
        clearInterval(watch);
        if (current === end) current = null;
        if (tell) { try { popup.postMessage({ smaran: 1, type: 'disconnect' }, origin); } catch (e) { /* already gone */ } }
        try { popup.close(); } catch (e) { /* already gone */ }
        for (const p of pending.values()) p.rej(fail('the player signed out.', 'signed-out'));
        pending.clear();
        if (!player) return reject(fail('the sign-in window was closed.', 'closed'));
        for (const fn of listeners) { try { fn(reason); } catch (e) { /* a listener's own error */ } }
      }

      function onMessage(e) {
        if (e.source !== popup || e.origin !== origin) return;
        const m = e.data;
        if (!m || typeof m !== 'object' || m.smaran !== 1) return;
        if (m.type === 'ready') {
          if (player) return end('reloaded', false); // the window started over, so the player is signed out
          popup.postMessage({ smaran: 1, type: 'connect' }, origin);
        } else if (m.type === 'connected' && !player) {
          player = Object.freeze({
            name: m.name,
            publicKey: m.publicKey,
            signMessage(message) {
              const bytes = typeof message === 'string' ? new TextEncoder().encode(message) : new Uint8Array(message);
              return request('signMessage', { message: bytes }).then(s => s[0]);
            },
            async signTransaction(tx) {
              return (await player.signAllTransactions([tx]))[0];
            },
            async signAllTransactions(txs) {
              const items = Array.from(txs, prepare);
              const sigs = await request('signTransactions', { messages: items.map(i => i.bytes) });
              return items.map((item, i) => item.attach(sigs[i]));
            },
            onDisconnect(fn) {
              listeners.add(fn);
              return () => listeners.delete(fn);
            },
            disconnect() { end('disconnected', true); }
          });
          resolve(player);
        } else if (m.type === 'result' || m.type === 'error') {
          const p = pending.get(m.id);
          if (!p) return;
          pending.delete(m.id);
          if (m.type === 'result') p.res(m.signatures);
          else p.rej(fail(String(m.message || 'refused.'), m.code));
        } else if (m.type === 'disconnect') {
          end('signed-out', false);
        }
      }
      // only when this page is really going away, not when the browser keeps it to come back to (persisted)
      const onLeave = e => { if (!e.persisted) end('left', true); };
      window.addEventListener('message', onMessage);
      window.addEventListener('pagehide', onLeave);
      const watch = setInterval(() => { if (popup.closed) end('closed', false); }, 500);
      current = end;
    });
  }

  return Object.freeze({ connect, signinUrl: SIGNIN_URL });
})();
