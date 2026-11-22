// SMARAN PLAY reader library. Plain ES module, no dependencies, runs in a browser, Deno or any WebCrypto runtime.
// It reads the PLAY ledger accounts from Solana through a public RPC and answers: who earned which achievement, who
// owns which relic. It never talks to an application server, and it fails closed: any doubt is reported as a state,
// never as an answer.
//
// PRIVATE BUILD. Layout constants must match play/program/src/lib.rs (checked against a fixture exported from the
// program's own host tests, see test/reader_test.ts).

export const DELEGATION_PROGRAM = "DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh";
export const LEDGER_MAGIC = "PPLEDGR1";
export const VERSION = 1;
export const KIND = { REGISTRY: 0, ACHIEVEMENTS: 1, RELICS: 2 };
export const SHARDS = 4;
export const SHARD_SIZE = 10240;
export const REGISTRY_SIZE = 512;
export const HEADER = 16;
export const ACH_SLOT = 96;
export const ACH_PER = 106;
export const REL_SLOT = 112;
export const REL_PER = 91;
export const GAME_MAGIC_OVER = "magic-over";
export const ACH_SIX_HITTER = 1;
export const FLAGS = { 0: "eligible", 1: "relic-requested", 2: "relic-minted" };

// ---------------------------------------------------------------- base58, hex, base64
const B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
export function b58encode(bytes) {
  let zeros = 0;
  while (zeros < bytes.length && bytes[zeros] === 0) zeros++;
  let n = 0n;
  for (const b of bytes) n = (n << 8n) | BigInt(b);
  let s = "";
  while (n > 0n) {
    s = B58[Number(n % 58n)] + s;
    n /= 58n;
  }
  return "1".repeat(zeros) + s;
}
export function b58decode(str) {
  let zeros = 0;
  while (zeros < str.length && str[zeros] === "1") zeros++;
  let n = 0n;
  for (const c of str) {
    const i = B58.indexOf(c);
    if (i < 0) throw new Error("invalid base58");
    n = n * 58n + BigInt(i);
  }
  const out = [];
  while (n > 0n) {
    out.unshift(Number(n & 0xffn));
    n >>= 8n;
  }
  return new Uint8Array([...new Array(zeros).fill(0), ...out]);
}
export function toHex(b) {
  return [...b].map((x) => x.toString(16).padStart(2, "0")).join("");
}
export function fromHex(h) {
  const o = new Uint8Array(h.length / 2);
  for (let i = 0; i < o.length; i++) o[i] = parseInt(h.slice(i * 2, i * 2 + 2), 16);
  return o;
}
export function b64decode(s) {
  const bin = atob(s);
  const o = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) o[i] = bin.charCodeAt(i);
  return o;
}
export function b64encode(b) {
  let s = "";
  for (const x of b) s += String.fromCharCode(x);
  return btoa(s);
}
const enc = new TextEncoder();
const dec = new TextDecoder();
function cat(...parts) {
  const n = parts.reduce((a, p) => a + p.length, 0);
  const o = new Uint8Array(n);
  let k = 0;
  for (const p of parts) {
    o.set(p, k);
    k += p.length;
  }
  return o;
}
export async function sha256(...parts) {
  return new Uint8Array(await crypto.subtle.digest("SHA-256", cat(...parts)));
}

// ---------------------------------------------------------------- program derived addresses
const P = (1n << 255n) - 19n;
const D = 37095705934669439343138083508754565189542113879843219016388785533085940283555n;
function mod(a) {
  const r = a % P;
  return r >= 0n ? r : r + P;
}
function pow(b, e) {
  let r = 1n;
  b = mod(b);
  while (e > 0n) {
    if (e & 1n) r = mod(r * b);
    b = mod(b * b);
    e >>= 1n;
  }
  return r;
}
/** True when the 32 bytes decode to a point on the ed25519 curve (such a value cannot be a program address). */
export function isOnCurve(bytes) {
  if (bytes.length !== 32) return false;
  let y = 0n;
  for (let i = 31; i >= 0; i--) y = (y << 8n) | BigInt(i === 31 ? bytes[i] & 0x7f : bytes[i]);
  y = mod(y);
  const u = mod(y * y - 1n);
  const v = mod(D * y * y + 1n);
  if (v === 0n) return false;
  if (u === 0n) return true;
  const x2 = mod(u * pow(v, P - 2n));
  return pow(x2, (P - 1n) / 2n) === 1n;
}
export async function findProgramAddress(seeds, programId) {
  const pid = typeof programId === "string" ? b58decode(programId) : programId;
  for (const s of seeds) if (s.length > 32) throw new Error("seed too long");
  for (let bump = 255; bump >= 0; bump--) {
    const h = await sha256(...seeds, new Uint8Array([bump]), pid, enc.encode("ProgramDerivedAddress"));
    if (!isOnCurve(h)) return { address: h, bump };
  }
  throw new Error("no viable bump");
}
/** The nine ledger addresses of a PLAY deployment. Identity comes from this derivation, not from account ownership. */
export async function ledgerAddresses(programId) {
  const out = { registry: null, achievements: [], relics: [] };
  out.registry = b58encode((await findProgramAddress([enc.encode("play-reg")], programId)).address);
  for (let i = 0; i < SHARDS; i++) {
    out.achievements.push(b58encode((await findProgramAddress([enc.encode("play-ach"), new Uint8Array([i])], programId)).address));
    out.relics.push(b58encode((await findProgramAddress([enc.encode("play-rel"), new Uint8Array([i])], programId)).address));
  }
  return out;
}

// ---------------------------------------------------------------- parsing
function u16(d, o) {
  return d[o] | (d[o + 1] << 8);
}
function u32(d, o) {
  return (d[o] | (d[o + 1] << 8) | (d[o + 2] << 16) | (d[o + 3] << 24)) >>> 0;
}
function u64(d, o) {
  return Number(BigInt(u32(d, o)) | (BigInt(u32(d, o + 4)) << 32n));
}
function trimNul(b) {
  let n = b.length;
  while (n > 0 && b[n - 1] === 0) n--;
  return dec.decode(b.slice(0, n));
}
function isZero(b) {
  return b.every((x) => x === 0);
}
function header(d, kind, size, idx) {
  if (!(d instanceof Uint8Array) || d.length !== size) throw new Error("BAD_LAYOUT: size");
  if (dec.decode(d.slice(0, 8)) !== LEDGER_MAGIC) throw new Error("BAD_LAYOUT: magic");
  if (d[9] !== VERSION) throw new Error("BAD_LAYOUT: version");
  if (d[8] !== kind) throw new Error("BAD_LAYOUT: kind");
  if (idx !== undefined && d[10] !== idx) throw new Error("BAD_LAYOUT: index");
  return { unsaved: d[11] === 1 };
}
export function parseRegistry(d) {
  header(d, KIND.REGISTRY, REGISTRY_SIZE);
  const games = [];
  for (let i = 0; i < Math.min(d[49], 8); i++) {
    const o = 64 + i * 49;
    games.push({ id: trimNul(d.slice(o, o + 16)), program: b58encode(d.slice(o + 16, o + 48)), enabled: d[o + 48] === 1 });
  }
  const kinds = [];
  for (let i = 0; i < Math.min(d[50], 8); i++) {
    const o = 456 + i * 4;
    kinds.push({ kind: d[o], rarity: d[o + 1], basisPoints: u16(d, o + 2) });
  }
  return { admin: b58encode(d.slice(16, 48)), oddsFrozen: d[48] === 1, games, kinds };
}
export function parseAchievementShard(d, idx) {
  const h = header(d, KIND.ACHIEVEMENTS, SHARD_SIZE, idx);
  const items = [];
  for (let s = 0; s < ACH_PER; s++) {
    const o = HEADER + s * ACH_SLOT;
    if (d[o] !== 1) continue;
    items.push({
      shard: idx,
      slot: s,
      state: FLAGS[d[o + 1]] ?? "unknown",
      achievement: u16(d, o + 2),
      game: trimNul(d.slice(o + 4, o + 20)),
      player: b58encode(d.slice(o + 20, o + 52)),
      matchNumber: u32(d, o + 52),
      tableIndex: d[o + 56],
      issuedAt: u64(d, o + 60),
      requestSeq: u32(d, o + 68),
      requestedAt: u64(d, o + 72),
    });
  }
  return { unsaved: h.unsaved, items };
}
export function parseRelicShard(d, idx, includeMoved = false) {
  const h = header(d, KIND.RELICS, SHARD_SIZE, idx);
  const items = [];
  for (let s = 0; s < REL_PER; s++) {
    const o = HEADER + s * REL_SLOT;
    if (includeMoved && d[o] > 2) throw new Error("Invalid relic state");
    if (d[o] !== 1 && !(includeMoved && d[o] === 2)) continue;
    const id = d.slice(o + 8, o + 40);
    items.push({
      shard: idx,
      slot: s,
      id: toHex(id),
      kind: d[o + 1],
      rarity: d[o + 2],
      ...(includeMoved ? { moved: d[o] === 2 } : {}),
      transferCount: u32(d, o + 4),
      owner: b58encode(d.slice(o + 40, o + 72)),
      origin: b58encode(d.slice(o + 72, o + 104)),
      mintedAt: u64(d, o + 104),
      ownerIsZero: isZero(d.slice(o + 40, o + 72)),
    });
  }
  return { unsaved: h.unsaved, items };
}

// ---------------------------------------------------------------- reading from Solana
/**
 * Reads the whole ledger. `fetchImpl` is injectable for tests. The result always says whether it is complete.
 * States per account: ok, NOT_FOUND, WRONG_OWNER, BAD_LAYOUT. A whole-read failure is {status:"RPC_ERROR"}.
 * A delegated account is read from its last committed state on Solana and flagged `delegated`.
 */
export async function readLedger({ rpcUrl, programId, fetchImpl = globalThis.fetch, commitment = "confirmed", includeMoved = false }) {
  const fail = (detail) => ({ status: "RPC_ERROR", detail, complete: false, slot: null, registry: null, achievements: [], relics: [], accounts: [], problems: [] });
  const addrs = await ledgerAddresses(programId);
  const keys = [addrs.registry, ...addrs.achievements, ...addrs.relics];
  let json;
  try {
    const r = await fetchImpl(rpcUrl, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getMultipleAccounts", params: [keys, { encoding: "base64", commitment }] }),
    });
    if (!r.ok) return fail("HTTP " + r.status);
    json = await r.json();
  } catch (e) {
    return fail(String(e && e.message ? e.message : e));
  }
  if (!json || json.error || !json.result || !Array.isArray(json.result.value) || json.result.value.length !== keys.length) {
    return fail(json && json.error ? JSON.stringify(json.error) : "unexpected response");
  }
  const accounts = [];
  const problems = [];
  const out = { status: "OK", complete: false, slot: json.result.context ? json.result.context.slot : null, registry: null, achievements: [], relics: [], accounts, problems };
  const take = (i, label, parse) => {
    const v = json.result.value[i];
    if (!v) {
      accounts.push({ address: keys[i], label, state: "NOT_FOUND" });
      problems.push({ label, state: "NOT_FOUND" });
      return null;
    }
    const delegated = v.owner === DELEGATION_PROGRAM;
    if (v.owner !== programId && !delegated) {
      accounts.push({ address: keys[i], label, state: "WRONG_OWNER", owner: v.owner });
      problems.push({ label, state: "WRONG_OWNER", owner: v.owner });
      return null;
    }
    try {
      const parsed = parse(b64decode(v.data[0]));
      accounts.push({ address: keys[i], label, state: "ok", delegated, unsaved: parsed.unsaved ?? false });
      return parsed;
    } catch (e) {
      accounts.push({ address: keys[i], label, state: "BAD_LAYOUT" });
      problems.push({ label, state: "BAD_LAYOUT", detail: String(e.message) });
      return null;
    }
  };
  out.registry = take(0, "registry", (d) => ({ ...parseRegistry(d), unsaved: d[11] === 1 }));
  for (let i = 0; i < SHARDS; i++) {
    const a = take(1 + i, "achievements-" + i, (d) => parseAchievementShard(d, i));
    if (a) out.achievements.push(...a.items);
    const r = take(1 + SHARDS + i, "relics-" + i, (d) => parseRelicShard(d, i, includeMoved));
    if (r) out.relics.push(...r.items);
  }
  out.complete = problems.length === 0;
  return out;
}

// ---------------------------------------------------------------- questions the apps ask
/** Relics owned by `owner`. `complete` is false when part of the ledger could not be read: then "none" is not an answer. */
export function relicsOwnedBy(ledger, owner) {
  if (ledger.status !== "OK") return { status: ledger.status, complete: false, relics: [] };
  return { status: "OK", complete: ledger.complete, relics: ledger.relics.filter((r) => r.owner === owner) };
}
export function achievementsOf(ledger, player) {
  if (ledger.status !== "OK") return { status: ledger.status, complete: false, achievements: [] };
  return { status: "OK", complete: ledger.complete, achievements: ledger.achievements.filter((a) => a.player === player) };
}
/** Who holds relic `idHex`, as of the last committed state. `owner` is null when unknown or the read was incomplete. */
export function ownerOfRelic(ledger, idHex) {
  if (ledger.status !== "OK") return { status: ledger.status, owner: null };
  const r = ledger.relics.find((x) => x.id === idHex);
  if (r) return { status: "OK", owner: r.owner, relic: r };
  return { status: ledger.complete ? "NOT_FOUND" : "INCOMPLETE", owner: null };
}

// ---------------------------------------------------------------- proving control of a key to another app
/**
 * A challenge is bound to the asking origin and a fresh nonce, so a signature given to one site cannot be replayed to
 * another. The signature is checked with WebCrypto Ed25519; where the runtime lacks it the check fails closed.
 */
export function challengeMessage({ origin, nonce, relic, issuedAt }) {
  if (!origin || !nonce || !relic || !issuedAt) throw new Error("incomplete challenge");
  return `SMARAN PLAY ownership proof v1\norigin:${origin}\nnonce:${nonce}\nrelic:${relic}\nissued:${issuedAt}`;
}
export class ChallengeBook {
  constructor({ ttlSeconds = 120, now = () => Math.floor(Date.now() / 1000) } = {}) {
    this.ttl = ttlSeconds;
    this.now = now;
    this.open = new Map();
  }
  issue(origin, relic) {
    const b = crypto.getRandomValues(new Uint8Array(16));
    const nonce = toHex(b);
    const issuedAt = this.now();
    this.open.set(nonce, { origin, relic, issuedAt });
    return { origin, relic, nonce, issuedAt, message: challengeMessage({ origin, nonce, relic, issuedAt }) };
  }
  /** Consumes the nonce whether or not the signature is good, so each challenge gets exactly one attempt. */
  async verify({ nonce, signature, publicKey }) {
    const c = this.open.get(nonce);
    if (!c) return { ok: false, reason: "UNKNOWN_NONCE" };
    this.open.delete(nonce);
    if (this.now() - c.issuedAt > this.ttl) return { ok: false, reason: "EXPIRED" };
    let key;
    try {
      key = await crypto.subtle.importKey("raw", b58decode(publicKey), { name: "Ed25519" }, false, ["verify"]);
    } catch (_e) {
      return { ok: false, reason: "UNSUPPORTED_OR_BAD_KEY" };
    }
    try {
      const msg = enc.encode(challengeMessage({ origin: c.origin, nonce, relic: c.relic, issuedAt: c.issuedAt }));
      const good = await crypto.subtle.verify({ name: "Ed25519" }, key, signature, msg);
      return good ? { ok: true, relic: c.relic, publicKey } : { ok: false, reason: "BAD_SIGNATURE" };
    } catch (_e) {
      return { ok: false, reason: "UNSUPPORTED" };
    }
  }
}

/** Ownership as an app sees it: a verified key plus the committed ledger. Either part failing means no access. */
export function grantsAccess(ledger, relicId, verifiedPublicKey) {
  const o = ownerOfRelic(ledger, relicId);
  return o.status === "OK" && o.owner === verifiedPublicKey;
}
