# Smaran

Smaran lets a player sign up for a Solana app with a player name and a passphrase. There is no wallet app, no seed phrase and no SOL to buy. The same name and passphrase bring the same player back on any device, and in any app that uses Smaran. Smaran is Sanskrit for remembering: remembering a name and a passphrase is all a player needs.

It is the sign-up that [Magic Over](../README.md) runs, packaged as one file, `smaran.js`, that any web page can load.

**Try it:** [ch4itu.github.io/MagicOver/smaran](https://ch4itu.github.io/MagicOver/smaran/). If you already play Magic Over, sign in there with the same name and passphrase and you get the same player key.

## Add it to a page

```html
<script src="smaran.js"></script>
<script type="module">
  // a new player
  const player = await Smaran.signUp({ name: 'asha-k', passphrase: 'a long passphrase you will remember' });

  // the same player later, on any device
  const again = await Smaran.signIn({ name: 'asha-k', passphrase: 'a long passphrase you will remember' });

  again.publicKey;          // base58 Solana address of the player
  again.sign('hello');      // 64-byte Ed25519 signature
  again.keypair();          // a Solana web3.js Keypair, for signing transactions
  again.forget();           // overwrite the secret key in memory (sign out)
</script>
```

| Call | What it does |
| --- | --- |
| `Smaran.signUp({ name, passphrase, onProgress })` | Creates the player and returns it. `onProgress` gets `{ phase: 'encrypt' \| 'proof' \| 'send' }`. Takes a few seconds. |
| `Smaran.signIn({ name, passphrase })` | Returns the same player on any device. Fails on a wrong passphrase. |
| `Smaran.lookup(name)` | `{ name, exists, publicKey, step }`, without a passphrase. |
| `Smaran.status()` | `{ players, capacity, steps }` of the registry. |
| `Smaran.checkName(name)` | The stored form of a name (lower case). Throws if the name is not allowed. |
| `Smaran.checkPassphrase(passphrase, name)` | `{ ok, errors }` before you call `signUp`. |
| `player.publicKey`, `player.secretKey` | The player's Solana address, and the usual 64-byte secret key. |
| `player.sign(message)`, `player.keypair()`, `player.forget()` | Sign bytes or text, get a web3.js `Keypair`, sign out. |

Names are 3 to 32 characters: a to z, 0 to 9, `_` or `-`. Passphrases need at least 16 characters.

The player has 0 SOL. It can sign zero-fee transactions on MagicBlock's Ephemeral Rollup for accounts your program has delegated there, the way Magic Over's moves work. Anything on Solana itself still needs a sponsor to pay.

## How it works

1. The browser makes a random signing key. It does not come from the passphrase.
2. Argon2id (64 MiB, three passes) turns the passphrase into a key that locks a copy of the signing key with XChaCha20-Poly1305. The passphrase never leaves the page.
3. After a small proof of work, the page stores the name's hash, the public key and the locked copy in the player registry on MagicBlock's rollup. The player signs this itself, with 0 SOL. The rollup then saves the registry to Solana.
4. Signing in reads the locked copy, unlocks it in the browser and checks it matches the registered public key.

More detail is in the [Magic Over README](../README.md#how-sign-up-works).

## Who pays

The app's sponsor pays the registry's storage once, never the player. The registry grows in capacity steps of 35-name groups at about 0.055 SOL per group, roughly 0.0016 SOL per name at today's rent. A name goes into the first step with room and never moves.

Today `smaran.js` uses the shared Magic Over registry on Solana Devnet, with 420 names of room in two open steps.

## Limits and safety

- **One name, everywhere:** names are shared by every app on the same registry. A name taken in Magic Over is taken in your app too, and it is the same player.
- **Apps see the key:** every app a player signs in to gets the full player key in memory. Players should only sign in to apps they trust. On Devnet nothing of value is at stake. This has to be solved before Smaran holds anything of value on mainnet.
- **Passphrase strength:** the locked copy is public, so anyone can try passphrases against it offline. Argon2id makes each guess slow, but a long passphrase is what keeps a player safe.
- **No reset or change:** a lost passphrase means a lost player, and the registry cannot yet change a passphrase.
- **Bots:** the proof of work slows scripted sign-ups but cannot stop a determined bot. Capacity opens one paid step at a time, so a flood can only fill the step that is open.
- **Devnet only:** the registry address, the rollup and the network are fixed in `smaran.js`. Using your own registry, or mainnet, means deploying `programs/registry` and changing those values.

## Files

- `smaran.js`: the file to load, about 550 KB. It is built, not hand-written: `python3 smaran/build.py` copies the Solana web3.js, crypto, TweetNaCl and registry code unchanged from Magic Over's `index.html` and adds `api.js`. So the kit runs exactly the code the game runs.
- `api.js`: the small `Smaran` interface above.
- `index.html`: the demo page.

Project code: [MIT](../LICENSE). Bundled libraries keep their own licenses: see [third-party notices](../THIRD_PARTY_NOTICES.md).
