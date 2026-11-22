import { ACH_SIX_HITTER, GAME_MAGIC_OVER, readLedger } from "./reader.js";
import { verifyRelicOwnership } from "./verify/vault.js";
import { relicSvg } from "./art.js";

export const pointsByRarity = Object.freeze([1, 2, 4, 8]);
const kinds = Object.freeze({ 1: "Ember", 2: "Compass", 3: "Bell", 4: "Crest" });
const zeroKey = "11111111111111111111111111111111";
let panelSerial = 0;

export function rankedOwners(relics) {
  const owners = new Map();
  const ids = new Set();
  for (const relic of relics) {
    if (ids.has(relic.id)) throw new Error("Duplicate relic in finalized records.");
    ids.add(relic.id);
    const row = owners.get(relic.owner) ?? { owner: relic.owner, points: 0, count: 0 };
    row.points += relic.points;
    row.count++;
    owners.set(relic.owner, row);
  }
  return [...owners.values()].sort((a, b) => b.points - a.points || (a.owner < b.owner ? -1 : Number(a.owner > b.owner)));
}

export function createRelicReader({ programId, rpcUrl, fetchImpl = globalThis.fetch }) {
  let lastSlot = 0;
  return async function finalizedRelics() {
    const finalizedFetch = (url, options) => fetchImpl(url, {
      ...options, redirect: "error", signal: AbortSignal.timeout(8000),
    });
    const ledger = await readLedger({ programId, rpcUrl, commitment: "finalized", includeMoved: true, fetchImpl: finalizedFetch });
    if (!ledger.complete || !Number.isSafeInteger(ledger.slot) || ledger.slot < lastSlot) {
      throw new Error("Finalized ownership is unavailable. Try again.");
    }
    const ids = new Set(), relics = [];
    let slot = ledger.slot;
    // The program retains relocated relic IDs and their original metadata
    // in the original ledger. Resolve the authoritative vault owner.
    for (const entry of ledger.relics) {
      const points = pointsByRarity[entry.rarity], name = kinds[entry.kind];
      if (!points || !name || /^0{64}$/.test(entry.id) || ids.has(entry.id)) {
        throw new Error("Finalized relic records disagree.");
      }
      ids.add(entry.id);
      let owner = entry.owner;
      if (entry.moved) {
        const ver = await verifyRelicOwnership({ programId, rpcUrl, relicId: entry.id, fetchImpl: finalizedFetch, minContextSlot: slot });
        if (ver.status !== "OWNED" || ver.source !== "vault-finalized") throw new Error("Vault ownership is unavailable.");
        owner = ver.owner;
        slot = ver.slot;
      }
      if (owner === zeroKey) throw new Error("Finalized relic has no owner.");
      relics.push({ ...entry, owner, points, name, originGame: "Magic Over" });
    }
    if (ledger.slot < lastSlot) throw new Error("Finalized ownership changed during refresh. Try again.");
    lastSlot = slot;
    return { relics, rankings: rankedOwners(relics), achievements: ledger.achievements, slot };
  };
}

export function mountRelicPanels({ root, read, currentPlayer }) {
  const doc = root.ownerDocument;
  const make = (tag, text) => {
    const el = doc.createElement(tag);
    if (text) el.textContent = text;
    return el;
  };
  const controls = make("div");
  controls.className = "relic-controls";
  const mine = make("button", "My Relics");
  const ranking = make("button", "Relic Leaderboard");
  mine.type = ranking.type = "button";
  const dialog = make("dialog"), heading = make("h2", "Relics");
  dialog.className = "relic-dialog";
  heading.id = "relic-panel-title-" + ++panelSerial;
  dialog.setAttribute("aria-labelledby", heading.id);
  const status = make("p"), body = make("div"), close = make("button", "Close"), refresh = make("button", "Refresh");
  status.setAttribute("role", "status");
  close.type = refresh.type = "button";
  dialog.append(heading, status, body, refresh, close);
  controls.append(mine, ranking, dialog);
  root.append(controls);
  let generation = 0, mode;
  close.onclick = () => dialog.close();
  dialog.addEventListener("close", () => { generation++; body.replaceChildren(); });
  function render(data, publicKey) {
    if (mode === "leaderboard") {
      if (!data.rankings.length) { status.textContent = "No finalized relics yet."; return; }
      const list = make("ol");
      for (const row of data.rankings) {
        const text = (row.owner === publicKey ? "You · " : "") + row.owner + " · " + row.points + " pts";
        list.append(make("li", text));
      }
      body.append(list);
      return;
    }
    const owned = data.relics.filter((relic) => relic.owner === publicKey);
    status.textContent = owned.length ? owned.length + " verified collectible" + (owned.length === 1 ? "" : "s") : "No collectible yet.";
    const list = make("ul");
    list.className = "relic-cards";
    for (const relic of owned) {
      const card = make("li"), art = make("img");
      art.src = "data:image/svg+xml," + encodeURIComponent(relicSvg(relic, 64));
      art.alt = relic.name;
      art.width = art.height = 64;
      card.append(art, make("h3", relic.name + " · " + relic.points + " pts"), make("p", "Earned in " + relic.originGame),
        make("p", "Finalized owner · " + relic.owner));
      list.append(card);
    }
    body.append(list);
    const history = data.achievements.filter((a) => a.player === publicKey && a.achievement === ACH_SIX_HITTER && a.game === GAME_MAGIC_OVER);
    if (history.length) {
      const section = make("section"), entries = make("ul");
      section.append(make("h3", "Lifetime achievements"), entries);
      for (const a of history) entries.append(make("li", "Six Hitter · Magic Over match " + a.matchNumber));
      body.append(section);
    }
  }
  async function load() {
    const request = ++generation, player = currentPlayer();
    body.replaceChildren();
    if (mode === "mine" && !player) { status.textContent = "Sign in to see your relics."; return; }
    status.textContent = "Checking finalized ownership…";
    try {
      const data = await read();
      if (request !== generation || !dialog.open || player !== currentPlayer()) return;
      status.textContent = "";
      render(data, player);
    } catch {
      if (request === generation && dialog.open) status.textContent = "Ownership could not be checked. Try again.";
    }
  }
  function open(next) {
    mode = next;
    heading.textContent = next === "mine" ? "My Relics" : "Relic Leaderboard";
    dialog.showModal();
    void load();
  }
  mine.onclick = () => open("mine");
  ranking.onclick = () => open("leaderboard");
  refresh.onclick = () => void load();
  return { reset() { if (dialog.open) dialog.close(); generation++; body.replaceChildren(); } };
}
