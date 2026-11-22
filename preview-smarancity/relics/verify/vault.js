import {
  DELEGATION_PROGRAM, HEADER, REL_PER, REL_SLOT, SHARD_SIZE, b58decode, b58encode,
  b64decode, findProgramAddress, fromHex, ledgerAddresses, toHex,
} from "../reader.js";

const enc = new TextEncoder(), dec = new TextDecoder();
const zero = new Uint8Array(32);
const SYSTEM = "11111111111111111111111111111111";
const eq = (a, b) => a.length === b.length && a.every((v, i) => v === b[i]);
const view = (b) => new DataView(b.buffer, b.byteOffset, b.byteLength);
const field = (b, start, length) => b.subarray(start, start + length);
const appAccount = (account, programId) => account && account.executable === false && account.owner === programId;
const bytes = (account) => {
  if (!Array.isArray(account?.data) || account.data[1] !== "base64") throw new Error("RPC account encoding");
  return b64decode(account.data[0]);
};

export async function vaultPageAddress(programId, owner, page) {
  const key = b58decode(owner);
  if (key.length !== 32 || eq(key, zero) || !Number.isInteger(page) || page < 0 || page > 0xffffffff) {
    throw new Error("Invalid vault page coordinates");
  }
  const pageBytes = new Uint8Array(4);
  view(pageBytes).setUint32(0, page, true);
  return b58encode((await findProgramAddress([enc.encode("play-vault-v4"), key, pageBytes], programId)).address);
}

export async function relicIndexAddress(programId, id) {
  if (id.length !== 32 || eq(id, zero)) throw new Error("Invalid relic ID");
  return b58encode((await findProgramAddress([enc.encode("play-relic-v4"), id], programId)).address);
}

function parseIndex(account, programId, id) {
  if (!appAccount(account, programId)) throw new Error("Unexpected vault index owner");
  const d = bytes(account);
  if (d.length !== 136 || dec.decode(field(d, 0, 8)) !== "PRLX0004"
    || d[12] >= 4 || view(d).getUint16(13, true) >= REL_PER || d[15] !== 0
    || d[24] !== 1 || !eq(field(d, 32, 32), id) || eq(field(d, 64, 32), zero)) {
    throw new Error("Invalid vault index");
  }
  return { d, owner: b58encode(field(d, 64, 32)), page: view(d).getUint32(8, true),
    shard: d[12], legacySlot: view(d).getUint16(13, true), revision: view(d).getBigUint64(16, true) };
}

function matchingLegacy(accounts, programId, id) {
  const matches = [];
  for (const [shard, a] of accounts.entries()) {
    if (!appAccount(a, programId) && !appAccount(a, DELEGATION_PROGRAM)) throw new Error("Missing or unrecognized legacy shard");
    const d = bytes(a);
    if (d.length !== SHARD_SIZE || dec.decode(field(d, 0, 8)) !== "PPLEDGR1"
      || d[8] !== 2 || d[9] !== 1 || d[10] !== shard) throw new Error("Invalid legacy shard");
    for (let slot = 0; slot < REL_PER; slot++) {
      const row = field(d, HEADER + slot * REL_SLOT, REL_SLOT);
      if (row[0] > 2) throw new Error("Unrecognized legacy relic state");
      if (row[0] && eq(field(row, 8, 32), id)) {
        matches.push({ shard, slot, row, delegated: a.owner === DELEGATION_PROGRAM });
      }
    }
  }
  if (matches.length > 1) throw new Error("Ambiguous legacy relic lineage");
  return matches[0];
}

/** Independent finalized RPC lookup. programId and rpcUrl are verifier configuration,
 * and should be pinned externally. A false RPC response is outside this trust model.
 * Only status OWNED includes an owner. Legacy results describe committed state. */
export async function verifyRelicOwnership({ programId, rpcUrl, relicId, fetchImpl = fetch, minContextSlot = 0 }) {
  let slot = minContextSlot;
  try {
    if (!Number.isSafeInteger(slot) || slot < 0 || !/^[a-f0-9]{64}$/i.test(relicId)
      || b58decode(programId).length !== 32) throw new Error("Invalid verifier input");
    const id = fromHex(relicId);
    const [legacy, index] = await Promise.all([ledgerAddresses(programId), relicIndexAddress(programId, id)]);
    const read = async (keys) => {
      const response = await fetchImpl(rpcUrl, {
        method: "POST", headers: { "Content-Type": "application/json" },
        signal: AbortSignal.timeout(8000),
        body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getMultipleAccounts",
          params: [keys, { encoding: "base64", commitment: "finalized", minContextSlot: slot }] }),
      });
      if (!response.ok) throw new Error("RPC HTTP " + response.status);
      const { result, error } = await response.json();
      if (error || !Array.isArray(result?.value) || result.value.length !== keys.length
        || !Number.isSafeInteger(result.context?.slot) || result.context.slot < slot) throw new Error("Incomplete or stale RPC response");
      slot = result.context.slot;
      return result.value;
    };
    const keys = [index, ...legacy.relics];
    const [first, ...shards] = await read(keys);
    if (first === null || (appAccount(first, SYSTEM) && bytes(first).length === 0)) {
      const old = matchingLegacy(shards, programId, id);
      if (!old) return { status: "NOT_FOUND", slot };
      if (old.row[0] !== 1 || eq(field(old.row, 40, 32), zero)) throw new Error("Legacy record has no valid ownership index");
      return { status: "OWNED", source: "legacy-committed", slot, owner: b58encode(field(old.row, 40, 32)), delegated: old.delegated };
    }
    const hint = parseIndex(first, programId, id);
    const pageKey = await vaultPageAddress(programId, hint.owner, hint.page);
    const [current, ...state] = await read([...keys, pageKey]);
    const pageAccount = state.pop();
    const vault = parseIndex(current, programId, id);
    if (vault.owner !== hint.owner || vault.page !== hint.page || vault.revision !== hint.revision) throw new Error("Ownership changed during verification. Retry.");
    const old = matchingLegacy(state, programId, id);
    if (!old || old.row[0] !== 2 || old.shard !== vault.shard || old.slot !== vault.legacySlot
      || !eq(field(old.row, 72, 40), field(vault.d, 96, 40))
      || !eq(field(old.row, 1, 3), field(vault.d, 25, 3))
      || BigInt(view(old.row).getUint32(4, true)) + vault.revision !== BigInt(view(vault.d).getUint32(28, true))
      || (vault.revision === 0n && !eq(field(old.row, 40, 32), field(vault.d, 64, 32)))) throw new Error("Legacy relocation evidence disagrees");
    if (!appAccount(pageAccount, programId)) throw new Error("Missing or unrecognized vault page");
    const page = bytes(pageAccount);
    if (page.length !== 112 || dec.decode(field(page, 0, 8)) !== "PVLT0004"
      || b58encode(field(page, 8, 32)) !== vault.owner || view(page).getUint32(40, true) !== vault.page
      || view(page).getUint32(44, true) !== 0
      || Number(eq(field(page, 48, 32), id)) + Number(eq(field(page, 80, 32), id)) !== 1) throw new Error("Vault page disagrees with ownership index");
    return { status: "OWNED", source: "vault-finalized", slot, owner: vault.owner,
      index, page: pageKey, revision: vault.revision.toString(), relicId: toHex(id) };
  } catch (error) {
    return { status: "UNVERIFIED", reason: String(error?.message ?? error) };
  }
}
