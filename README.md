# Smaran: One Login for Every Solana App
### A simple login for sunset-proof apps. A username and a password: no wallet, no seed phrase, no crypto.

[![Registry on Solana Devnet](https://img.shields.io/badge/Registry-Solana%20Devnet-black)](https://explorer.solana.com/address/CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P?cluster=devnet)
[![Built on MagicBlock](https://img.shields.io/badge/Rollups-MagicBlock-b33a2b)](https://docs.magicblock.gg)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

**Try it:** [ch4itu.github.io/MagicOver/blitz](https://ch4itu.github.io/MagicOver/blitz/) · **Video:** [Smaran demo](https://ch4itu.github.io/MagicOver/media/smaran-blitz-demo.mp4) · **Blitz v10 judges:** [what we built this week](blitz/README.md#for-the-judges-magicblock-solana-blitz-v10)

| At a glance | |
| --- | --- |
| **Problem** | To use a Solana app, a person first needs a wallet, a seed phrase and some SOL. Most people stop there, and in many countries crypto is hard or risky to use. |
| **Solution** | Smaran: one username and password for every app that supports it, games or not. Each person's key is kept on Solana, locked with their password. |
| **Sustainability** | People never pay. Costs are to be covered by non-intrusive ads only, plus one save fund that anyone can top up. There is no server to switch off. |
| **MagicBlock** | Its rollups make every action instant and free for people, its private rollup keeps a bowler's secret hidden, and its VRF makes the draws fair. |

---

## The Problem

> **Today: install a wallet, write down 24 words, buy some SOL. Only then can you start.**

| What stands in the way | What it means for people |
| --- | --- |
| **A wallet to install** | One more app before they can even look around |
| **A seed phrase to keep** | 12 or 24 words; lose them and the wallet is gone |
| **SOL to buy first** | An exchange, an ID check and a card, just to begin |
| **Crypto rules** | In many countries buying or holding crypto is banned, restricted, heavily taxed or legally unclear |
| **Apps that close** | When an app's company stops, its logins and what its players earned usually go with its servers |

**Most people stop here.** So apps lose people before they see what the app does, and where crypto is banned or restricted, many cannot start at all.

---

## The Solution

People already know usernames and passwords. Smaran keeps that, and puts the account itself on Solana.

```
Usual Solana app                     With Smaran
────────────────                     ───────────
Install a wallet                     Pick a username
Write down a seed phrase             Pick a password
Buy SOL for fees                     Start: every action costs 0 SOL
       ↓                                    ↓
Most people stop here                The same login works in every Smaran app
```

```
┌──────────────────────────────────────────────────────────────────┐
│ APPS        Magic Over · EternalBliss · SmaranWorld              │
│             Ludo Freeflow · Smaran Streaks · yours               │
└──────────────────────────────────────────────────────────────────┘
                                 │ one username and password
                                 ▼
┌──────────────────────────────────────────────────────────────────┐
│ SMARAN      a key for each person, locked with their password,   │
│             unlocked only in their own browser                   │
└──────────────────────────────────────────────────────────────────┘
                                 │ actions that cost people 0 SOL
                                 ▼
┌──────────────────────────────────────────────────────────────────┐
│ MAGICBLOCK  public rollup: logins, worlds, tables, habits        │
│             private rollup: matches, relics                      │
│             VRF: fair draws                                      │
└──────────────────────────────────────────────────────────────────┘
                                 │ saved
                                 ▼
┌──────────────────────────────────────────────────────────────────┐
│ SOLANA      the lasting copy of logins, relics and records       │
└──────────────────────────────────────────────────────────────────┘
```

**How it works**

1. The browser makes a new Solana key for the person. The key does not come from the password.
2. It locks the key with the password (Argon2id and XChaCha20-Poly1305) and stores the locked copy in Smaran's registry on MagicBlock's public rollup, which saves it to Solana.
3. On any device, the same username and password unlock it again, in the browser. No server ever sees the password.
4. Apps then use that key on MagicBlock's rollups, where every action costs the person 0 SOL.

Smaran is Sanskrit for remembering: all a person needs is to remember a username and a password. Full guide: [smaran/README.md](smaran/README.md).

---

## Proof: Five Working Apps on One Login

### 1. Magic Over: 3D cricket duel
Bat against the computer or a friend. Hit a four or a six while signed in, win or lose, and claim a relic.

| | |
| --- | --- |
| **Play** | [blitz/magic-over](https://ch4itu.github.io/MagicOver/blitz/magic-over/) |
| **Read more** | [MAGIC_OVER.md](MAGIC_OVER.md) |
| **Uses MagicBlock** | Every match on the private rollup, so the bowler's hidden variation stays hidden until the ball is played; VRF for the AI's choices |

### 2. EternalBliss: adventure with shared worlds
Quests, battles, shared worlds and duels. Your relics show in its menu.

| | |
| --- | --- |
| **Play** | [blitz/eternalbliss](https://ch4itu.github.io/MagicOver/blitz/eternalbliss/) |
| **Read more** | [eternalbliss/README.md](eternalbliss/README.md) |
| **Uses MagicBlock** | Shared worlds, duels and character books on the public rollup, saved to Solana every few minutes |

### 3. SmaranWorld: 144 villages to walk and talk in
A shared world of villages in open country: fields, forests, hills, lakes, desert and tundra. Each village is kept by a relic holder.

| | |
| --- | --- |
| **Play** | [blitz/smaranworld](https://ch4itu.github.io/MagicOver/blitz/smaranworld/) |
| **Read more** | [blitz/smaranworld/README.md](blitz/smaranworld/README.md) |
| **Uses MagicBlock** | Where players stand, what they say and each village's sign, on the public rollup |

### 4. Ludo Freeflow: Ludo with no turns
Everyone rolls and moves at once. Pick a table online.

| | |
| --- | --- |
| **Play** | [blitz/ludo-freeflow](https://ch4itu.github.io/MagicOver/blitz/ludo-freeflow/) |
| **Read more** | [ludo-freeflow/README.md](ludo-freeflow/README.md) |
| **Uses MagicBlock** | Tables on the public rollup; every die from VRF |

### 5. Smaran Streaks: daily habits (not a game)
Keep up to three habits, one check-in a day. It shows the same login works for everyday apps too.

| | |
| --- | --- |
| **Play** | [blitz/streaks](https://ch4itu.github.io/MagicOver/blitz/streaks/) |
| **Read more** | [programs/streaks/README.md](programs/streaks/README.md) |
| **Uses MagicBlock** | Check-ins on the public rollup, saved to Solana at most once an hour |

**Relics tie them together.** A relic is a record of something you did, like a medal. You earn it in Magic Over, it shows in EternalBliss and SmaranWorld, and you can give it to another player. It has no money value: no buying, no selling, no marketplace, no betting. More: [programs/play/README.md](programs/play/README.md).

> **One username, one password, five apps. A relic earned in one shows in the others.**

---

## How MagicBlock Is Used

| MagicBlock | What it does here |
| --- | --- |
| **Public rollup** (devnet-as) | Smaran logins, EternalBliss's worlds and duels, SmaranWorld's players, chat and signs, Ludo Freeflow's tables, Streaks' check-ins |
| **Private rollup** (devnet-tee) | Every Magic Over match, the relics and achievements, and the leaderboard. A ball's hidden variation can be read only by a human bowler, never by the batter or a watcher |
| **VRF** | The AI's choices in Magic Over, which relic a player gets, every Ludo die |
| **0-SOL actions** | People sign every action on the rollups with a key that holds no SOL. No wallet, no fee |
| **Saves to Solana** | Each shared account is saved from the rollup to Solana. MagicBlock does 25 saves per delegation free, then the account pays 0.0001 SOL per save from its own small reserve |
| **Delegation** | When a reserve runs low, anyone can bring the account back to Solana, top it up from the save fund and delegate it again with the same settings |

Each app's README has its own "How MagicBlock is used" section.

---

## Sustainability

| Question | Answer |
| --- | --- |
| **Who pays?** | Never the people using the apps. Whoever runs the apps pays the chain costs, mostly once, as account deposits. |
| **How is that covered?** | Non-intrusive ads only: a sponsor's name on a stadium board or a village sign, never anything that interrupts play. There are no ads in this demo, and the plan is not proven yet. |
| **Can others help?** | Yes. Anyone can send SOL to the [save fund](programs/save-fund), `9NLZkhdU3vG1kDi2RBW29pRyLcVW9MHSANWKMdE61myj`. It can only top up these apps' accounts when they run low, at most 0.1 SOL a day, and has no withdraw. It is a gift, not an investment: nothing comes back. |
| **What if we stop?** | There is no server to switch off. The pages are static and open source, so anyone can host them. Logins and relics are on Solana, so people can still sign in and keep what they earned. |
| **How long does it last?** | The aim is as long as Solana and MagicBlock run. That needs SOL in the save fund, someone to run its refill script (anyone can) and our upgrade key given up, which we plan for mainnet. Devnet can be reset, so lasting records need mainnet. Details: [Built to outlive us](blitz/README.md#built-to-outlive-us). |

**Who it is for:** people who will never touch crypto, and places where buying or holding it is hard. People never buy, hold or send any coin or token, there is no KYC step, and nothing has cash value. This is a design note, not legal advice.

---

## What Makes This Different?

| Usual approach | Smaran |
| --- | --- |
| A wallet app or extension | A username and a password, in any modern browser, on a phone or a computer |
| A seed phrase to write down | A password the person picks |
| A login on the app company's server | No login server: the locked key is on Solana and the password never leaves the browser |
| One account per app | One account for every Smaran app |
| A fee on every action | 0 SOL for people, for every action on the rollups |
| Progress in one app's database | Relics and records on Solana, under the person's own key |

**What Smaran is not:** a wallet (its accounts hold no money) or proof of a unique person (one person can make many usernames). A forgotten password cannot be reset: that account is lost and the person starts a new one. More: [what Smaran is, and what it is not](smaran/README.md#what-smaran-is-and-what-it-is-not).

---

## Programs on Solana Devnet

| Program | Address | Runs on |
| --- | --- | --- |
| Smaran registry | [`CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P`](https://explorer.solana.com/address/CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P?cluster=devnet) | public rollup |
| Magic Over | [`HwXp4A62RG3nBkmxTTaHbSi1wNj8vaVqZ6Kn5VkYBswT`](https://explorer.solana.com/address/HwXp4A62RG3nBkmxTTaHbSi1wNj8vaVqZ6Kn5VkYBswT?cluster=devnet) | private rollup |
| Magic Over result register | [`BcS2Gb1Gihfzqs1UkViNnbQzjGcCGyvXu81VhwsFrUzU`](https://explorer.solana.com/address/BcS2Gb1Gihfzqs1UkViNnbQzjGcCGyvXu81VhwsFrUzU?cluster=devnet) | private rollup |
| Relics and achievements | [`HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B`](https://explorer.solana.com/address/HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B?cluster=devnet) | private rollup |
| EternalBliss | [`6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb`](https://explorer.solana.com/address/6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb?cluster=devnet) | public rollup |
| SmaranWorld | [`HPEA1Ht9rBnqEQVdaEQ9QuqwiWAbY6fgCKs2ZPuQvcCW`](https://explorer.solana.com/address/HPEA1Ht9rBnqEQVdaEQ9QuqwiWAbY6fgCKs2ZPuQvcCW?cluster=devnet) | public rollup |
| Ludo Freeflow | [`2i2REZWzDeAqKuJBEypH9fBtVZB6jmZbBRX7YH4fiETp`](https://explorer.solana.com/address/2i2REZWzDeAqKuJBEypH9fBtVZB6jmZbBRX7YH4fiETp?cluster=devnet) | public rollup |
| Smaran Streaks | [`A98As1brXWi2DuJFChgat6bL9XbmaRRiF5vMFZswALoi`](https://explorer.solana.com/address/A98As1brXWi2DuJFChgat6bL9XbmaRRiF5vMFZswALoi?cluster=devnet) | public rollup |
| Save fund | [`PRU4HhX5x3Kf5wpTCt7amrjTbwXZ88NxU3DwwnHrcuo`](https://explorer.solana.com/address/PRU4HhX5x3Kf5wpTCt7amrjTbwXZ88NxU3DwwnHrcuo?cluster=devnet) | Solana only |

Source for every program is in [`programs/`](programs) and [`eternalbliss/program`](eternalbliss/program).

---

## Quick Start

### For people
Open [ch4itu.github.io/MagicOver/blitz](https://ch4itu.github.io/MagicOver/blitz/), pick any app, press **Sign in** and create an account. Use the same username and password in the other apps.

### For app builders
Add Smaran's sign-in window to your page. Your site never sees the password or the key, and the person approves every signature in Smaran's window:

```html
<script src="https://ch4itu.github.io/MagicOver/smaran/connect.js"></script>
<script>
  signInButton.onclick = async () => {
    const player = await SmaranConnect.connect(); // opens Smaran's sign-in window
    player.name;                                  // 'asha-k'
    player.publicKey;                             // the person's Solana address
    await player.signTransaction(tx);             // the person approves it in the window
  };
</script>
```

Everything else, including `smaran.js` for your own page: [smaran/README.md](smaran/README.md#add-it-to-your-app).

---

## Documentation

| Document | What it covers |
| --- | --- |
| [Smaran](smaran/README.md) | The login: how it works, how to add it, pros, cons and the numbers |
| [Smaran for founders](smaran/FOUNDERS.md) | What Smaran means for an app's founders |
| [Blitz v10 entry](blitz/README.md) | What we built this week, how to try it in two minutes, costs and limits |
| [Magic Over](MAGIC_OVER.md) | The cricket game, its rules and how it uses MagicBlock |
| [EternalBliss](eternalbliss/README.md) | The adventure game and its shared worlds |
| [SmaranWorld](blitz/smaranworld/README.md) | The world of 144 villages and how its land is made |
| [Ludo Freeflow](ludo-freeflow/README.md) | Ludo with no turns, and how an online match works |
| [Smaran Streaks](programs/streaks/README.md) | The daily habit app and its rules |
| [Relics](programs/play/README.md) | Relics and achievements |
| [Save fund](programs/save-fund/README.md) | The one address that keeps saves going, and its refill script |

---

## Security and Limits

### Built-in protections
- The password never leaves the browser, and the unlocked key is kept only in memory.
- Argon2id (64 MiB, three passes) makes each password guess slow; XChaCha20-Poly1305 locks the key.
- A small proof of work slows sign-up bots.
- Smaran's sign-in window shows the real address of the site asking, asks the person for every signature, and never signs a sign-up or password change for a site.
- The programs check what they can see, such as a four or a six in the match's own count.

### Honest limits
- **Devnet prototype.** Devnet can be reset and erase test data.
- **No password reset.** The locked copy is public, so a weak password can be guessed offline. Smaran accounts hold no money.
- **One person can make many usernames.** A login tied to a key is not proof of a unique person.
- **Not cheat-proof.** A modified page can still play badly or skip recording a result.
- **Our upgrade key** can still change the programs' rules until we give it up at mainnet.

More: [Blitz v10 limits](blitz/README.md#limits) and [Smaran's cons](smaran/README.md#cons).

---

## Roadmap

### Done
- [x] Smaran sign-up and sign-in on MagicBlock's public rollup, saved to Solana
- [x] Magic Over on the private rollup, with VRF and a friend leaderboard
- [x] Smaran's sign-in window and `smaran.js` for other sites
- [x] EternalBliss on MagicBlock with Smaran sign-in
- [x] Relics and achievements with VRF draws, shown across apps
- [x] SmaranWorld: 144 villages, chat, signs and one shared clock
- [x] Ludo Freeflow online, with VRF dice
- [x] Smaran Streaks, the first app that is not a game
- [x] The save fund: one address anyone can fund, and refills anyone can run

### Planned
- [ ] Mainnet
- [ ] Give up or share the upgrade key once the rules are settled
- [ ] Non-intrusive sponsor boards and village signs, to cover the running costs
- [ ] SmaranWorld past 144 villages (a small program upgrade), and a village for every player
- [ ] More deeds to earn, and relic perks in EternalBliss and Ludo Freeflow
- [ ] Better cricket motion, teams and smooth mobile play in Magic Over

---

## Hackathons

- **MagicBlock Solana Blitz v10** (20 to 27 November 2026): relics, SmaranWorld, Ludo Freeflow online, Smaran Streaks and the save fund were built this week. [What is new and what is reused](blitz/README.md#for-the-judges-magicblock-solana-blitz-v10).
- **MagicBlock Forge:** Smaran began here, with Magic Over.

---

## Team

**Chaitanya Kumar Jagabathula** ([ch4itu](https://github.com/ch4itu))

---

## License

Project code: [MIT](LICENSE). Bundled libraries: [third-party notices](THIRD_PARTY_NOTICES.md).

---

## Acknowledgments

- **MagicBlock:** Ephemeral Rollups, the private rollup and VRF.
- **Solana:** the lasting home of every login and relic.
- **GitHub Pages:** static hosting, with no server of our own.
- **AI assistance:** code, tests and documents were built with help from AI tools.
