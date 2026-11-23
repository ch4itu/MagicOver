//! Base-layer vault ownership and legacy relocation. Vaults are never delegated.
use super::*;

pub(super) const PAGE_SEED: &[u8] = b"play-vault-v4";
pub(super) const INDEX_SEED: &[u8] = b"play-relic-v4";
pub(super) const PAGE_LEN: usize = 112;
pub(super) const INDEX_LEN: usize = 136;
const PAGE_HEADER: usize = 48;
const RECORD_OFFSET: usize = 24;
const PAGE_MAGIC: &[u8; 8] = b"PVLT0004";
const INDEX_MAGIC: &[u8; 8] = b"PRLX0004";
pub(super) const MOVED: u8 = 2;

fn page_key(owner: &[u8; 32], number: u32, program: &[u8; 32]) -> ([u8; 32], u8) {
    find_pda(&[PAGE_SEED, owner, &number.to_le_bytes()], program)
}

fn page(c: &Ctx, i: usize, owner: &[u8; 32], number: u32, writable: bool) -> Result<(), u32> {
    let a = &c.accs[i];
    if a.key() != &page_key(owner, number, c.program_id).0 {
        return Err(ERR_PDA);
    }
    if a.owner() != c.program_id || a.executable {
        return Err(ERR_OWNER);
    }
    if writable && !a.is_writable {
        return Err(ERR_ACCOUNTS);
    }
    let d = a.data();
    if d.len() != PAGE_LEN || &d[..8] != PAGE_MAGIC || &d[8..40] != owner
        || rd32(d, 40) != number || d[44..48] != [0; 4]
        || (!is_zero32(&d[48..80]) && d[48..80] == d[80..112])
    {
        return Err(ERR_DATA);
    }
    Ok(())
}

fn empty_slot(d: &[u8]) -> Result<usize, u32> {
    (PAGE_HEADER..PAGE_LEN).step_by(32)
        .find(|&o| is_zero32(&d[o..o + 32])).ok_or(ERR_CAPACITY)
}

#[inline(never)]
fn create(c: &Ctx, target: usize, len: usize, seed: &[&[u8]]) -> Result<(), u32> {
    let payer = 1;
    let (p, a, s) = (&c.accs[payer], &c.accs[target], &c.accs[target + 1]);
    if s.key() != &SYSTEM_PROGRAM || !s.executable || !p.is_writable || !a.is_writable
        || p.key() == a.key() || a.executable
    {
        return Err(ERR_ACCOUNTS);
    }
    if !p.is_signer {
        return Err(ERR_SIGNER);
    }
    if p.owner() != &SYSTEM_PROGRAM || p.len() != 0 || p.executable || a.owner() != &SYSTEM_PROGRAM || a.len() != 0 {
        return Err(ERR_OWNER);
    }
    // Allocate and assign also work if someone has sent lamports to the PDA.
    let missing = rent_minimum(len).saturating_sub(a.lamports());
    if missing > p.lamports() {
        return Err(ERR_SPONSOR_LOW);
    }
    if missing > 0 {
        let mut transfer = [0; 12];
        transfer[0] = 2;
        wr64(&mut transfer, 4, missing);
        invoke(c.accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(payer, true, true), (target, true, false)], data: &transfer, seeds: &[] })?;
    }
    let mut allocate = [0; 12];
    allocate[0] = 8;
    wr64(&mut allocate, 4, len as u64);
    invoke(c.accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(target, true, true)], data: &allocate, seeds: &[seed] })?;
    let mut assign = [0; 36];
    assign[0] = 1;
    assign[4..].copy_from_slice(c.program_id);
    invoke(c.accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(target, true, true)], data: &assign, seeds: &[seed] })
}

/// [player signer, sponsor signer writable, new page writable, system, previous page if expanding]
#[inline(never)]
pub(super) fn open(c: &Ctx) -> Result<(), u32> {
    if c.ix.len() != 5 || !(4..=5).contains(&c.n) {
        return Err(ERR_DATA);
    }
    let n = rd32(c.ix, 1);
    let owner = c.accs[0].key();
    if !c.accs[0].is_signer || owner == c.accs[1].key() || is_zero32(owner) {
        return Err(ERR_SIGNER);
    }
    match n.checked_sub(1) {
        Some(prev) if c.n == 5 => {
            page(c, 4, owner, prev, false)?;
            if empty_slot(c.accs[4].data()).is_ok() {
                return Err(ERR_DATA);
            }
        }
        None if c.n == 4 => (),
        _ => return Err(ERR_ACCOUNTS),
    }
    let (address, bump) = page_key(owner, n, c.program_id);
    if c.accs[2].key() != &address {
        return Err(ERR_PDA);
    }
    create(c, 2, PAGE_LEN, &[PAGE_SEED, owner, &n.to_le_bytes(), &[bump]])?;
    let d = c.accs[2].data();
    d[..8].copy_from_slice(PAGE_MAGIC);
    d[8..40].copy_from_slice(owner);
    wr32(d, 40, n);
    Ok(())
}

/// [owner, sponsor, page, index, system, delegation record, relic shards 0..4].
/// The selected legacy shard must be writable. All four must be undelegated.
#[inline(never)]
pub(super) fn migrate(c: &Ctx) -> Result<(), u32> {
    if c.ix.len() != 40 || c.n != 10 {
        return Err(ERR_DATA);
    }
    let owner = c.accs[0].key();
    if !c.accs[0].is_signer || owner == c.accs[1].key() {
        return Err(ERR_SIGNER);
    }
    let shard = c.ix[1] as usize;
    let slot = rd16(c.ix, 2) as usize;
    let n = rd32(c.ix, 4);
    let id = &c.ix[8..40];
    if shard >= SHARDS || slot >= REL_PER || is_zero32(id) {
        return Err(ERR_RANGE);
    }
    for i in 0..SHARDS {
        if check_ledger(c, &c.accs[6 + i], KIND_REL, i == shard)? as usize != i {
            return Err(ERR_ACCOUNTS);
        }
    }
    let legacy = &c.accs[6 + shard];
    let record = &c.accs[5];
    if record.key() != &find_pda(&[b"delegation", legacy.key()], &DELEGATION_PROGRAM).0
        || record.owner() != &SYSTEM_PROGRAM || record.len() != 0 || record.executable
    {
        return Err(ERR_OWNER);
    }
    let o = HEADER + slot * REL_SLOT;
    let old: [u8; REL_SLOT] = legacy.data()[o..o + REL_SLOT].try_into().map_err(|_| ERR_DATA)?;
    if old[L_STATE] != 1 || &old[L_ID..L_ID + 32] != id || &old[L_OWNER..L_OWNER + 32] != owner {
        return Err(ERR_NOT_OWNER);
    }
    let count = (0..SHARDS).flat_map(|i| c.accs[6 + i].data()[HEADER..].chunks_exact(REL_SLOT))
        .filter(|r| matches!(r[L_STATE], 1 | MOVED) && &r[L_ID..L_ID + 32] == id).count();
    if count != 1 {
        return Err(ERR_DATA);
    }
    page(c, 2, owner, n, true)?;
    let dest = empty_slot(c.accs[2].data())?;
    let (key, bump) = find_pda(&[INDEX_SEED, id], c.program_id);
    if c.accs[3].key() != &key {
        return Err(ERR_PDA);
    }
    create(c, 3, INDEX_LEN, &[INDEX_SEED, id, &[bump]])?;
    let index = c.accs[3].data();
    index[..8].copy_from_slice(INDEX_MAGIC);
    wr32(index, 8, n);
    index[12] = shard as u8;
    wr16(index, 13, slot as u16);
    index[RECORD_OFFSET..].copy_from_slice(&old);
    c.accs[2].data()[dest..dest + 32].copy_from_slice(id);
    legacy.data()[o + L_STATE] = MOVED;
    Ok(())
}

/// [current owner, source page, destination page, index]. The instruction pins the
/// exact relic, recipient, pages and revision even when ownership cycles back.
#[inline(never)]
pub(super) fn transfer(c: &Ctx) -> Result<(), u32> {
    if c.ix.len() != 81 || c.n != 4 {
        return Err(ERR_DATA);
    }
    let owner = c.accs[0].key();
    let from = rd32(c.ix, 1);
    let to = rd32(c.ix, 5);
    let revision = rd64(c.ix, 9);
    let id = &c.ix[17..49];
    let recipient: &[u8; 32] = c.ix[49..81].try_into().map_err(|_| ERR_DATA)?;
    if !c.accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    if owner == recipient || is_zero32(recipient) || is_zero32(id) {
        return Err(ERR_DATA);
    }
    page(c, 1, owner, from, true)?;
    page(c, 2, recipient, to, true)?;
    let a = &c.accs[3];
    if a.key() != &find_pda(&[INDEX_SEED, id], c.program_id).0 {
        return Err(ERR_PDA);
    }
    if a.owner() != c.program_id || a.executable || !a.is_writable {
        return Err(ERR_OWNER);
    }
    let d = a.data();
    let record = RECORD_OFFSET;
    if d.len() != INDEX_LEN || &d[..8] != INDEX_MAGIC || &d[record + L_ID..record + L_ID + 32] != id
        || d[record + L_STATE] != 1 || d[12] as usize >= SHARDS || rd16(d, 13) as usize >= REL_PER || d[15] != 0
    {
        return Err(ERR_DATA);
    }
    if &d[record + L_OWNER..record + L_OWNER + 32] != owner || rd32(d, 8) != from || rd64(d, 16) != revision {
        return Err(ERR_NOT_OWNER);
    }
    let source = c.accs[1].data();
    let target = c.accs[2].data();
    let src = (PAGE_HEADER..PAGE_LEN).step_by(32).find(|&o| &source[o..o + 32] == id).ok_or(ERR_DATA)?;
    if target[PAGE_HEADER..].chunks_exact(32).any(|x| x == id) {
        return Err(ERR_DATA);
    }
    let dst = empty_slot(target)?;
    let next = revision.checked_add(1).ok_or(ERR_RANGE)?;
    let count = rd32(d, record + L_COUNT).checked_add(1).ok_or(ERR_RANGE)?;
    source[src..src + 32].fill(0);
    target[dst..dst + 32].copy_from_slice(id);
    d[record + L_OWNER..record + L_OWNER + 32].copy_from_slice(recipient);
    wr32(d, record + L_COUNT, count);
    wr32(d, 8, to);
    wr64(d, 16, next);
    Ok(())
}
