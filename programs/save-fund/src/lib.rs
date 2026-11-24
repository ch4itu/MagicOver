//! Smaran save fund: one address anyone can send SOL to. It only pays for saving the Smaran apps' accounts to Solana.
//!
//! The apps keep their shared accounts on MagicBlock rollups. Each account pays MagicBlock for its own saves to Solana
//! from a small reserve of SOL above its rent. When a reserve runs low, anyone can hand the account back to Solana (each
//! program allows that once fewer than ten paid saves are left), ask this fund to top it up, and delegate it again.
//!
//! What the fund will do, and nothing else:
//!   - top up one of the accounts listed below, and only while it is on Solana (owned by its own program, so not
//!     delegated) and low: under 0.001 SOL above rent. It tops up to 0.01 SOL above rent.
//!   - pay at most 0.1 SOL a day (UTC) in all, and always keep its own rent.
//! There is no withdraw, no admin and no other way to move its SOL. Anyone may call Refill; the caller pays only the
//! transaction fee. SOL sent to the fund's address with a plain transfer adds to it.
//!
//! Account: the fund, PDA ["smaran-save-fund"], 64 bytes: magic "SMFUND01", version 8, bump 9, the UTC day of the last
//! refill u64 at 16, lamports paid that day u64 at 24, lamports paid in all u64 at 32, refills in all u32 at 40.
//!
//! Instructions (first data byte):
//!   0 Init    [payer s w, fund w, system]    creates the fund; anyone, once; the payer pays its rent
//!   1 Refill  [fund w, target w]             data: list entry u8, a u8, b u8 (the target's seeds, see LIST)
#![cfg_attr(target_arch = "bpf", no_std)]
use solana_compiler_builtins as _;

#[cfg(target_arch = "bpf")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { sys::abort() }
}

// ---------------------------------------------------------------- the accounts the fund may pay
const SYSTEM_PROGRAM: [u8; 32] = [0u8; 32];
// CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P: Smaran's username registry
const REGISTRY: [u8; 32] = [
    170, 231, 160, 244, 115, 185, 145, 46, 33, 18, 198, 117, 183, 167, 8, 38, 234, 143, 9, 10, 45, 116, 91, 253, 130, 232,
    127, 183, 140, 34, 0, 106,
];
// HUS4FPeXKqybWdZh4MGb6bjhH87A1oRCoSNQQck2SN5B: achievements and relics
const PLAY: [u8; 32] = [
    244, 194, 0, 234, 130, 224, 7, 68, 147, 99, 27, 17, 179, 124, 95, 142, 200, 3, 116, 94, 19, 166, 91, 92, 146, 238, 128,
    200, 144, 248, 154, 254,
];
// A98As1brXWi2DuJFChgat6bL9XbmaRRiF5vMFZswALoi: Smaran Streaks
const STREAKS: [u8; 32] = [
    135, 205, 239, 157, 151, 233, 9, 229, 140, 119, 195, 6, 63, 246, 43, 144, 216, 39, 122, 251, 160, 227, 110, 156, 17,
    171, 52, 19, 153, 32, 139, 73,
];
// BcS2Gb1Gihfzqs1UkViNnbQzjGcCGyvXu81VhwsFrUzU: Magic Over's leaderboard
const BOARD: [u8; 32] = [
    157, 168, 125, 106, 106, 12, 16, 51, 35, 166, 234, 75, 117, 131, 142, 95, 221, 100, 166, 153, 46, 204, 136, 66, 9, 160,
    175, 237, 210, 205, 175, 25,
];
// 6e5ied8gFYVSEoc6wWGwBwVSFAC9DZNYAgKg53M1rGdb: EternalBliss
const ETERNAL: [u8; 32] = [
    83, 202, 66, 98, 173, 220, 96, 43, 241, 100, 112, 220, 51, 90, 10, 177, 91, 30, 72, 184, 98, 226, 125, 59, 193, 172,
    168, 78, 63, 116, 101, 14,
];

/// (program, seed, form). Form 0: the address is [seed]; 1: [seed, a]; 2: [seed, a, b].
const LIST: [(&[u8; 32], &[u8], u8); 9] = [
    (&REGISTRY, b"sabha-reg", 1),      // 0 registry shards of the first step: a = shard
    (&REGISTRY, b"sabha-reg", 2),      // 1 registry shards of later steps: a = step, b = shard
    (&PLAY, b"play-reg", 0),           // 2 the achievement and relic register
    (&PLAY, b"play-ach", 1),           // 3 achievement shards: a = shard
    (&PLAY, b"play-rel", 1),           // 4 relic shards: a = shard
    (&STREAKS, b"smaran-streaks", 1),  // 5 Streaks groups: a = group
    (&BOARD, b"mo-board", 1),          // 6 Magic Over leaderboards: a = board
    (&ETERNAL, b"eb-land", 1),         // 7 EternalBliss worlds: a = world
    (&ETERNAL, b"eb-book", 1),         // 8 EternalBliss character books: a = book
];

/// MagicBlock's charge per save once a delegation's free saves are used.
const SAVE_FEE: u64 = 100_000;
/// An account is low below rent plus this: fewer than ten paid saves left. The same line each program uses for its
/// hand-back to Solana.
const LOW: u64 = 10 * SAVE_FEE;
/// A refill brings an account to rent plus this: one hundred paid saves.
const TOP_UP: u64 = 100 * SAVE_FEE;
/// The most the fund pays in one UTC day, in all.
const DAILY_CAP: u64 = 100_000_000;
const DAY_SECS: u64 = 86_400;

// ---------------------------------------------------------------- the fund account
const FUND_SEED: &[u8] = b"smaran-save-fund";
const FUND_SIZE: usize = 64;
const MAGIC: &[u8; 8] = b"SMFUND01";
const F_VERSION: usize = 8;
const F_BUMP: usize = 9;
const F_DAY: usize = 16;
const F_PAID_TODAY: usize = 24;
const F_PAID_TOTAL: usize = 32;
const F_REFILLS: usize = 40;

const MAX_PERMITTED_DATA_INCREASE: usize = 10_240;
const MAX_ACCOUNTS: usize = 4;

// errors
const ERR_ACCOUNTS: u32 = 1;
const ERR_SIGNER: u32 = 2;
const ERR_OWNER: u32 = 3;
const ERR_DATA: u32 = 4;
const ERR_PDA: u32 = 9;
const ERR_CPI: u32 = 10;
const ERR_TAG: u32 = 11;
/// No such entry in the list.
const ERR_ENTRY: u32 = 40;
/// The account still has ten or more paid saves in its reserve.
const ERR_NOT_LOW: u32 = 41;
/// Today's refills would go over the daily limit.
const ERR_DAILY_CAP: u32 = 42;
/// The fund cannot pay this refill and keep its own rent.
const ERR_FUND_LOW: u32 = 43;
/// The fund already exists.
const ERR_EXISTS: u32 = 44;

// ---------------------------------------------------------------- runtime
/// One account as the loader hands it over, or as a host test builds it.
#[cfg_attr(not(target_arch = "bpf"), allow(dead_code))]
#[derive(Clone, Copy)]
struct Acc {
    key: *const [u8; 32],
    owner: *mut [u8; 32],
    lamports: *mut u64,
    data_len: *mut u64,
    data: *mut u8,
    rent_epoch: u64,
    is_signer: bool,
    is_writable: bool,
    executable: bool,
}
impl Acc {
    const fn empty() -> Acc {
        Acc {
            key: core::ptr::null(),
            owner: core::ptr::null_mut(),
            lamports: core::ptr::null_mut(),
            data_len: core::ptr::null_mut(),
            data: core::ptr::null_mut(),
            rent_epoch: 0,
            is_signer: false,
            is_writable: false,
            executable: false,
        }
    }
    fn key(&self) -> &[u8; 32] {
        unsafe { &*self.key }
    }
    fn owner(&self) -> &[u8; 32] {
        unsafe { &*self.owner }
    }
    fn len(&self) -> usize {
        unsafe { *self.data_len as usize }
    }
    fn lamports(&self) -> u64 {
        unsafe { *self.lamports }
    }
    #[allow(clippy::mut_from_ref)]
    fn data(&self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.data, self.len()) }
    }
}

/// A cross-program call. `metas` lists (index into the account list, writable, signer).
struct Cpi<'a> {
    program: &'a [u8; 32],
    metas: &'a [(usize, bool, bool)],
    data: &'a [u8],
    seeds: &'a [&'a [&'a [u8]]],
}

#[cfg(target_arch = "bpf")]
mod sys {
    use super::{Acc, Cpi, ERR_CPI};
    #[repr(C)]
    pub struct SolBytes {
        pub addr: *const u8,
        pub len: u64,
    }
    #[repr(C)]
    struct SolAccountMeta {
        pubkey: *const [u8; 32],
        is_writable: bool,
        is_signer: bool,
    }
    #[repr(C)]
    struct SolInstruction {
        program_id: *const [u8; 32],
        accounts: *const SolAccountMeta,
        account_len: u64,
        data: *const u8,
        data_len: u64,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct SolAccountInfo {
        key: *const [u8; 32],
        lamports: *mut u64,
        data_len: u64,
        data: *mut u8,
        owner: *const [u8; 32],
        rent_epoch: u64,
        is_signer: bool,
        is_writable: bool,
        executable: bool,
    }
    #[repr(C)]
    struct SolSignerSeeds {
        addr: *const SolBytes,
        len: u64,
    }
    extern "C" {
        fn sol_log_(message: *const u8, len: u64);
        fn sol_try_find_program_address(
            seeds: *const SolBytes,
            seeds_len: u64,
            program_id: *const u8,
            address: *mut u8,
            bump_seed: *mut u8,
        ) -> u64;
        fn sol_invoke_signed_c(
            instruction: *const SolInstruction,
            account_infos: *const SolAccountInfo,
            account_infos_len: u64,
            signers_seeds: *const SolSignerSeeds,
            signers_seeds_len: u64,
        ) -> u64;
        fn sol_get_clock_sysvar(addr: *mut u8) -> u64;
        fn sol_get_rent_sysvar(addr: *mut u8) -> u64;
        pub fn abort() -> !;
    }
    const NULL_BYTES: SolBytes = SolBytes { addr: core::ptr::null(), len: 0 };

    pub fn log(msg: &str) {
        unsafe { sol_log_(msg.as_ptr(), msg.len() as u64) }
    }
    pub fn find_pda(seeds: &[&[u8]], program_id: &[u8; 32]) -> ([u8; 32], u8) {
        let mut sb: [SolBytes; 4] = [NULL_BYTES, NULL_BYTES, NULL_BYTES, NULL_BYTES];
        for (i, s) in seeds.iter().enumerate() {
            sb[i] = SolBytes { addr: s.as_ptr(), len: s.len() as u64 };
        }
        let mut out = [0u8; 32];
        let mut bump = 0u8;
        unsafe {
            sol_try_find_program_address(sb.as_ptr(), seeds.len() as u64, program_id.as_ptr(), out.as_mut_ptr(), &mut bump);
        }
        (out, bump)
    }
    /// (slot, unix timestamp in seconds)
    pub fn clock() -> (u64, i64) {
        let mut buf = [0u8; 40];
        unsafe {
            sol_get_clock_sysvar(buf.as_mut_ptr());
        }
        let mut a = [0u8; 8];
        a.copy_from_slice(&buf[0..8]);
        let mut b = [0u8; 8];
        b.copy_from_slice(&buf[32..40]);
        (u64::from_le_bytes(a), i64::from_le_bytes(b))
    }
    pub fn rent_per_byte_and_threshold() -> (u64, u64) {
        let mut buf = [0u8; 24];
        unsafe {
            sol_get_rent_sysvar(buf.as_mut_ptr());
        }
        let mut a = [0u8; 8];
        a.copy_from_slice(&buf[0..8]);
        let mut b = [0u8; 8];
        b.copy_from_slice(&buf[8..16]);
        (u64::from_le_bytes(a), u64::from_le_bytes(b))
    }
    #[inline(never)]
    pub fn invoke(accs: &[Acc], cpi: &Cpi) -> Result<(), u32> {
        const NULL_META: SolAccountMeta = SolAccountMeta { pubkey: core::ptr::null(), is_writable: false, is_signer: false };
        let info_of = |a: &Acc| SolAccountInfo {
            key: a.key,
            lamports: a.lamports,
            data_len: a.len() as u64,
            data: a.data,
            owner: a.owner,
            rent_epoch: a.rent_epoch,
            is_signer: a.is_signer,
            is_writable: a.is_writable,
            executable: a.executable,
        };
        let mut m: [SolAccountMeta; 4] = [NULL_META; 4];
        let mut infos: [SolAccountInfo; 4] = [info_of(&accs[0]); 4];
        for (i, (idx, w, s)) in cpi.metas.iter().enumerate() {
            m[i] = SolAccountMeta { pubkey: accs[*idx].key, is_writable: *w, is_signer: *s };
            infos[i] = info_of(&accs[*idx]);
        }
        let mut seed_bytes: [SolBytes; 4] = [NULL_BYTES, NULL_BYTES, NULL_BYTES, NULL_BYTES];
        let mut signers: [SolSignerSeeds; 1] = [SolSignerSeeds { addr: core::ptr::null(), len: 0 }];
        if let Some(seeds) = cpi.seeds.first() {
            for (j, s) in seeds.iter().enumerate() {
                seed_bytes[j] = SolBytes { addr: s.as_ptr(), len: s.len() as u64 };
            }
            signers[0] = SolSignerSeeds { addr: seed_bytes.as_ptr(), len: seeds.len() as u64 };
        }
        let ix = SolInstruction {
            program_id: cpi.program,
            accounts: m.as_ptr(),
            account_len: cpi.metas.len() as u64,
            data: cpi.data.as_ptr(),
            data_len: cpi.data.len() as u64,
        };
        let n_signers = if cpi.seeds.is_empty() { 0 } else { 1 };
        let rc = unsafe { sol_invoke_signed_c(&ix, infos.as_ptr(), cpi.metas.len() as u64, signers.as_ptr(), n_signers) };
        if rc != 0 {
            return Err(ERR_CPI);
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "bpf"))]
mod sys {
    pub(crate) use crate::host::*;
}

use sys::{clock, find_pda, invoke, log};

fn rent_minimum(len: usize) -> u64 {
    let (per_byte, thr_bits) = sys::rent_per_byte_and_threshold();
    let base = (128 + len as u64) * per_byte;
    if thr_bits == 0x3FF0_0000_0000_0000 {
        base
    } else {
        base * 2
    }
}

// ---------------------------------------------------------------- input parsing
unsafe fn read_u64(p: *const u8) -> u64 {
    core::ptr::read_unaligned(p as *const u64)
}

unsafe fn parse(input: *mut u8, accs: &mut [Acc; MAX_ACCOUNTS]) -> (usize, &'static [u8], &'static [u8; 32]) {
    let mut off: usize = 0;
    let n = read_u64(input) as usize;
    off += 8;
    let mut count = 0;
    for i in 0..n {
        let dup = *input.add(off);
        off += 1;
        if dup != 0xff {
            off += 7;
            if i < MAX_ACCOUNTS {
                accs[i] = accs[dup as usize];
                count = i + 1;
            }
            continue;
        }
        let is_signer = *input.add(off) != 0;
        let is_writable = *input.add(off + 1) != 0;
        let executable = *input.add(off + 2) != 0;
        off += 3 + 4;
        let key = input.add(off) as *const [u8; 32];
        off += 32;
        let owner = input.add(off) as *mut [u8; 32];
        off += 32;
        let lamports = input.add(off) as *mut u64;
        off += 8;
        let data_len_ptr = input.add(off) as *mut u64;
        let data_len = read_u64(input.add(off)) as usize;
        off += 8;
        let data = input.add(off);
        off += data_len + MAX_PERMITTED_DATA_INCREASE;
        off = (off + 7) & !7;
        let rent_epoch = read_u64(input.add(off));
        off += 8;
        if i < MAX_ACCOUNTS {
            accs[i] = Acc { key, owner, lamports, data_len: data_len_ptr, data, rent_epoch, is_signer, is_writable, executable };
            count = i + 1;
        }
    }
    let ix_len = read_u64(input.add(off)) as usize;
    off += 8;
    let ix = core::slice::from_raw_parts(input.add(off), ix_len);
    off += ix_len;
    let program_id = &*(input.add(off) as *const [u8; 32]);
    (count, ix, program_id)
}

// ---------------------------------------------------------------- byte helpers
fn eq32(a: &[u8], b: &[u8]) -> bool {
    let mut d = 0u8;
    for i in 0..32 {
        d |= a[i] ^ b[i];
    }
    d == 0
}
fn rd32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}
fn wr32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
fn rd64(d: &[u8], o: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[o..o + 8]);
    u64::from_le_bytes(b)
}
fn wr64(d: &mut [u8], o: usize, v: u64) {
    d[o..o + 8].copy_from_slice(&v.to_le_bytes());
}

// ---------------------------------------------------------------- instructions
struct Ctx<'a> {
    accs: &'a [Acc],
    n: usize,
    ix: &'a [u8],
    program_id: &'a [u8; 32],
}

/// The address a list entry names, or an error for an unknown entry or seeds the entry does not use.
fn listed_address(entry: u8, a: u8, b: u8) -> Result<([u8; 32], &'static [u8; 32]), u32> {
    let (program, seed, form) = *LIST.get(entry as usize).ok_or(ERR_ENTRY)?;
    let (a_b, b_b) = ([a], [b]);
    let key = match form {
        0 if a == 0 && b == 0 => find_pda(&[seed], program).0,
        1 if b == 0 => find_pda(&[seed, &a_b], program).0,
        2 => find_pda(&[seed, &a_b, &b_b], program).0,
        _ => return Err(ERR_DATA),
    };
    Ok((key, program))
}

/// How much a refill pays: up to rent + TOP_UP, and only for a low account.
fn refill_amount(lamports: u64, rent: u64) -> Result<u64, u32> {
    if lamports >= rent + LOW {
        return Err(ERR_NOT_LOW);
    }
    Ok(rent + TOP_UP - lamports)
}

fn ix_init(c: &Ctx) -> Result<(), u32> {
    // [payer s w, fund w, system]
    let (accs, program_id) = (c.accs, c.program_id);
    if c.n < 3 || c.ix.len() != 1 {
        return Err(ERR_ACCOUNTS);
    }
    let (payer, fund) = (&accs[0], &accs[1]);
    if !payer.is_signer || !payer.is_writable || !fund.is_writable {
        return Err(ERR_SIGNER);
    }
    let (key, bump) = find_pda(&[FUND_SEED], program_id);
    if !eq32(&key, fund.key()) || !eq32(accs[2].key(), &SYSTEM_PROGRAM) {
        return Err(ERR_PDA);
    }
    if !eq32(fund.owner(), &SYSTEM_PROGRAM) || fund.len() != 0 {
        return Err(ERR_EXISTS);
    }
    let bump_b = [bump];
    let seeds: &[&[&[u8]]] = &[&[FUND_SEED, &bump_b]];
    let rent = rent_minimum(FUND_SIZE);
    if fund.lamports() == 0 {
        let mut data = [0u8; 52];
        data[4..12].copy_from_slice(&rent.to_le_bytes());
        data[12..20].copy_from_slice(&(FUND_SIZE as u64).to_le_bytes());
        data[20..52].copy_from_slice(program_id);
        invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(0, true, true), (1, true, true)], data: &data, seeds })?;
    } else {
        // SOL already sent to the address: top it up to rent, then give it its space and this program as owner
        if fund.lamports() < rent {
            let mut data = [0u8; 12];
            data[0] = 2;
            data[4..12].copy_from_slice(&(rent - fund.lamports()).to_le_bytes());
            invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(0, true, true), (1, true, false)], data: &data, seeds: &[] })?;
        }
        let mut alloc = [0u8; 12];
        alloc[0] = 8;
        alloc[4..12].copy_from_slice(&(FUND_SIZE as u64).to_le_bytes());
        invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(1, true, true)], data: &alloc, seeds })?;
        let mut assign = [0u8; 36];
        assign[0] = 1;
        assign[4..36].copy_from_slice(program_id);
        invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(1, true, true)], data: &assign, seeds })?;
    }
    let d = fund.data();
    d[..8].copy_from_slice(MAGIC);
    d[F_VERSION] = 1;
    d[F_BUMP] = bump;
    log("save fund: ready");
    Ok(())
}

fn ix_refill(c: &Ctx) -> Result<(), u32> {
    // [fund w, target w]; data: tag, list entry, a, b
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 2 || ix.len() != 4 {
        return Err(ERR_ACCOUNTS);
    }
    let (fund, target) = (&accs[0], &accs[1]);
    if !fund.is_writable || !target.is_writable {
        return Err(ERR_ACCOUNTS);
    }
    let d = fund.data();
    if !eq32(fund.owner(), program_id) || d.len() != FUND_SIZE || &d[..8] != MAGIC {
        return Err(ERR_OWNER);
    }
    let (key, _) = find_pda(&[FUND_SEED], program_id);
    if !eq32(&key, fund.key()) {
        return Err(ERR_PDA);
    }
    let (want, program) = listed_address(ix[1], ix[2], ix[3])?;
    if !eq32(&want, target.key()) {
        return Err(ERR_PDA);
    }
    // owned by its own program: it exists and is on Solana, not delegated to a rollup
    if !eq32(target.owner(), program) {
        return Err(ERR_OWNER);
    }
    let amount = refill_amount(target.lamports(), rent_minimum(target.len()))?;
    let today = clock().1.max(0) as u64 / DAY_SECS; // unsigned: SBPF v0 has no signed division
    let paid_today = if rd64(d, F_DAY) == today { rd64(d, F_PAID_TODAY) } else { 0 };
    if paid_today + amount > DAILY_CAP {
        return Err(ERR_DAILY_CAP);
    }
    if fund.lamports() < rent_minimum(FUND_SIZE) + amount {
        return Err(ERR_FUND_LOW);
    }
    unsafe {
        *fund.lamports -= amount;
        *target.lamports += amount;
    }
    wr64(d, F_DAY, today);
    wr64(d, F_PAID_TODAY, paid_today + amount);
    wr64(d, F_PAID_TOTAL, rd64(d, F_PAID_TOTAL) + amount);
    wr32(d, F_REFILLS, rd32(d, F_REFILLS) + 1);
    log("save fund: refilled");
    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    let mut accs: [Acc; MAX_ACCOUNTS] = [Acc::empty(); MAX_ACCOUNTS];
    let (n, ix, program_id) = parse(input, &mut accs);
    let c = Ctx { accs: &accs, n, ix, program_id };
    let r = match ix.first() {
        Some(0) => ix_init(&c),
        Some(1) => ix_refill(&c),
        _ => Err(ERR_TAG),
    };
    match r {
        Ok(()) => 0,
        Err(code) => code as u64,
    }
}

#[cfg(not(target_arch = "bpf"))]
mod host;
#[cfg(test)]
mod tests;
