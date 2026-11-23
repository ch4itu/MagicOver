//! SMARAN PLAY: program-validated achievements, VRF-backed relic rewards and an owner-signed relic ledger.
//!
//! PRIVATE BUILD. Not deployed. Written for the MagicBlock private rollup (TEE), on the same validator as the Magic Over
//! tables it reads. Same runtime style as the other programs in this repository: no_std, no allocator, no SDK.
//!
//! Accounts (all PDAs of this program, all delegated to one validator):
//!   registry  ["play-reg"]       512 bytes   admin, odds_frozen, up to 8 games, up to 8 relic kinds
//!   ach shard ["play-ach", i]    10,240      i in 0..4, 106 slots of 96 bytes after a 16 byte header
//!   rel shard ["play-rel", i]    10,240      i in 0..4,  91 slots of 112 bytes after a 16 byte header
//!
//! Instructions (first data byte):
//!   0 InitAccount    base [operator s w, pda w, system]               kind, idx, lamports u64
//!   1 Delegate       base [payer s w, pda w, this program, buffer w, record w, metadata w, delegation, system]
//!                                                                      kind, idx, has_validator, validator 32, commit_ms u32
//!   2 Commit         ER   [payer s, pda w, magic program, context w, vault w]   (anyone; only while the account has unsaved changes)
//!   3 CommitUndelegate ER [operator s w, pda w, magic program, context w]      kind, idx
//!   4 RegisterGame   ER   [operator s, registry w]                    game_id 16, program_id 32, enabled
//!   5 SetOdds        ER   [operator s, registry w]                    n, n x (kind, rarity, weight u16); only before FreezeOdds
//!   6 FreezeOdds     ER   [operator s, registry w]
//!   7 ClaimAchievement ER [player s, registry, ach0 w, ach1 w, ach2 w, ach3 w, game table, magic, context w, vault w]
//!                                                                      ach_id u16, table idx
//!   8 RequestRelic   ER   [player s w, ach shard w, identity, vrf program, queue w, system, slot hashes, registry]   slot u16
//!  "PLAYVRF1" VRF callback [vrf identity s, ach shard w, registry, rel0 w, rel1 w, rel2 w, rel3 w]
//!                                                                      randomness 32, ach shard u8, slot u16, seq u32
//!  10 TransferRelic  ER   [owner s, relic shard w, magic, context w, vault w]   slot u16, new owner 32
//!  MagicBlock undelegate callback (8 byte discriminator)
#![cfg_attr(target_arch = "bpf", no_std)]
#[cfg(target_arch = "bpf")]
use solana_compiler_builtins as _;

#[cfg(target_arch = "bpf")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { sys::abort() }
}

// ---------------------------------------------------------------- program and layout constants
const SYSTEM_PROGRAM: [u8; 32] = [0u8; 32];
// DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh
const DELEGATION_PROGRAM: [u8; 32] = [
    181, 183, 0, 225, 242, 87, 58, 192, 204, 6, 34, 1, 52, 74, 207, 151, 184, 53, 6, 235, 140, 229,
    25, 152, 204, 98, 126, 24, 147, 128, 167, 62,
];
// Magic11111111111111111111111111111111111111
const MAGIC_PROGRAM: [u8; 32] = [
    5, 69, 180, 36, 176, 218, 112, 149, 236, 185, 214, 222, 195, 119, 215, 40, 145, 182, 231, 142,
    146, 234, 18, 214, 223, 187, 58, 64, 0, 0, 0, 0,
];
// MagicContext1111111111111111111111111111111
const MAGIC_CONTEXT: [u8; 32] = [
    5, 69, 180, 36, 196, 165, 40, 191, 95, 180, 3, 47, 68, 82, 130, 142, 187, 56, 171, 193, 210,
    220, 151, 247, 63, 139, 148, 84, 128, 0, 0, 0,
];
// Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz
const VRF_PROGRAM: [u8; 32] = [
    7, 100, 104, 119, 200, 241, 253, 43, 158, 185, 243, 92, 234, 235, 93, 25, 63, 162, 158, 196, 92,
    7, 181, 101, 242, 27, 167, 177, 39, 172, 247, 127,
];
// 5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc (ER oracle queue)
const VRF_QUEUE: [u8; 32] = [
    69, 185, 193, 160, 226, 118, 43, 231, 240, 17, 186, 10, 117, 95, 214, 26, 143, 3, 61, 185, 132,
    75, 237, 89, 242, 59, 5, 128, 136, 40, 202, 167,
];
// SysvarS1otHashes111111111111111111111111111
const SLOT_HASHES: [u8; 32] = [
    6, 167, 213, 23, 25, 47, 10, 175, 198, 242, 101, 227, 251, 119, 204, 122, 218, 130, 197, 41, 208,
    190, 59, 19, 110, 45, 0, 85, 32, 0, 0, 0,
];
// MTEWGuqxUpYZGFJQcp8tLN7x5v9BSeoFHYWQQ3n3xzo (MagicBlock TEE validator, the one Magic Over's tables live on).
// If S2 forces the Ludo Freeflow fallback on the public rollup, this is the one constant to change.
const VALIDATOR: [u8; 32] = [
    5, 61, 71, 26, 133, 158, 115, 46, 104, 11, 201, 88, 248, 65, 7, 43, 143, 63, 188, 25, 115, 155,
    230, 151, 196, 198, 129, 18, 111, 140, 31, 116,
];
const EXTERNAL_UNDELEGATE: [u8; 8] = [196, 28, 41, 206, 48, 37, 51, 167];
const VRF_CALLBACK: [u8; 8] = *b"PLAYVRF1";
const IDENTITY_SEED: &[u8] = b"identity";

/// The operator and deploy key, 32 raw bytes written into operator.bin by scripts/set_operator.ts before a build.
/// An all-zero file means "not configured": every operator instruction is refused.
#[cfg(not(test))]
const OPERATOR: [u8; 32] = *include_bytes!("../operator.bin");
#[cfg(test)]
const OPERATOR: [u8; 32] = [0x0e; 32];

const REG_SEED: &[u8] = b"play-reg";
const ACH_SEED: &[u8] = b"play-ach";
const REL_SEED: &[u8] = b"play-rel";
const MAGIC_LEDGER: &[u8; 8] = b"PPLEDGR1";
const VERSION: u8 = 1;
const KIND_REG: u8 = 0;
const KIND_ACH: u8 = 1;
const KIND_REL: u8 = 2;
const SHARDS: usize = 4;
const SHARD_SIZE: usize = 10_240;
const REG_SIZE: usize = 512;
const HEADER: usize = 16;
const H_KIND: usize = 8;
const H_VERSION: usize = 9;
const H_IDX: usize = 10;
const H_DIRTY: usize = 11; // 1 while the account holds changes not yet scheduled for commit
const H_SAVED_TS: usize = 12; // u32 unix seconds of the last save this program scheduled; 0 = never (bytes were unused before)

// achievement slot, 96 bytes
const ACH_SLOT: usize = 96;
const ACH_PER: usize = 106;
const A_STATE: usize = 0;
const A_FLAGS: usize = 1; // 0 eligible, 1 relic requested, 2 relic minted
const A_ID: usize = 2;
const A_GAME: usize = 4;
const A_PLAYER: usize = 20;
const A_EVENT: usize = 52;
const A_ISSUED: usize = 60;
const A_REQ_SEQ: usize = 68;
const A_REQ_AT: usize = 72;
const F_ELIGIBLE: u8 = 0;
const F_PENDING: u8 = 1;
const F_MINTED: u8 = 2;

// relic slot, 112 bytes
const REL_SLOT: usize = 112;
const REL_PER: usize = 91;
const L_STATE: usize = 0;
const L_KIND: usize = 1;
const L_RARITY: usize = 2;
const L_COUNT: usize = 4;
const L_ID: usize = 8;
const L_OWNER: usize = 40;
const L_ORIGIN: usize = 72;
const L_MINTED: usize = 104;

// registry
const R_ADMIN: usize = 16;
const R_FROZEN: usize = 48;
const R_NGAMES: usize = 49;
const R_NKINDS: usize = 50;
const R_GAMES: usize = 64;
const GAME_ENTRY: usize = 49; // game_id 16, program_id 32, enabled 1
const MAX_GAMES: usize = 8;
const R_KINDS: usize = 456;
const KIND_ENTRY: usize = 4; // kind 1, rarity 1, weight u16
const MAX_KINDS: usize = 8;

// the first achievement
const ACH_SIX_HITTER: u16 = 1;
const GAME_MAGIC_OVER: [u8; 16] = *b"magic-over\0\0\0\0\0\0";
const REPLACE_AFTER_SECS: i64 = 8;
/// A save may be re-requested for an account with no unsaved changes only this long after the last scheduled save.
const RESAVE_AFTER_SECS: i64 = 30;
const DRAW_LIMIT: u32 = 60_000;
const ODDS_TOTAL: u32 = 10_000;
const SPONSOR_MARGIN: u64 = 100_000;

// Magic Over's table, read only (programs/game/src/lib.rs at commit 397a389).
const T_SEED: &[u8] = b"so-table";
const T_SIZE: usize = 384;
const T_MAGIC: &[u8; 8] = b"SOVER001";
const T_PHASE: usize = 9;
const T_WINNER: usize = 14; // 0 none, 1 side A, 2 side B, 3 tie
const T_A: usize = 16;
#[allow(dead_code)]
const T_B: usize = 48;
const T_MATCH: usize = 80; // u32, only increments
const T_TOTALS: usize = 304; // per side: runs, fours, sixes, wickets lost, u16 each
const T_TOTALS_ON: usize = 320;
const T_FINISHED: u8 = 6;
const T_MAX_IDX: u8 = 16;

const MAX_PERMITTED_DATA_INCREASE: usize = 10_240;
const MAX_ACCOUNTS: usize = 12;

// errors
const ERR_ACCOUNTS: u32 = 1;
const ERR_SIGNER: u32 = 2;
const ERR_OWNER: u32 = 3;
const ERR_DATA: u32 = 4;
const ERR_NOT_FINISHED: u32 = 5;
const ERR_NOT_WINNER: u32 = 6;
const ERR_NO_SIXES: u32 = 7;
const ERR_SEQ: u32 = 8;
const ERR_PDA: u32 = 9;
const ERR_CPI: u32 = 10;
const ERR_TAG: u32 = 11;
const ERR_RANGE: u32 = 12;
const ERR_IDENTITY: u32 = 13;
const ERR_ALREADY_CLAIMED: u32 = 14;
const ERR_CAPACITY: u32 = 15;
const ERR_NOT_FROZEN: u32 = 16;
const ERR_FROZEN: u32 = 17;
const ERR_GAME_OFF: u32 = 18;
const ERR_NO_ACH: u32 = 19;
const ERR_MINTED: u32 = 20;
const ERR_TOO_EARLY: u32 = 21;
const ERR_NOT_PENDING: u32 = 22;
const ERR_NOT_OWNER: u32 = 23;
const ERR_SAME_OWNER: u32 = 24;
const ERR_NO_TOTALS: u32 = 25;
const ERR_DRAW: u32 = 26;
const ERR_NOT_CONFIGURED: u32 = 27;
const ERR_TABLE: u32 = 28;
const ERR_NOTHING_TO_SAVE: u32 = 29;
const ERR_SPONSOR_LOW: u32 = 30;
const ERR_ODDS: u32 = 31;
const ERR_PROGRAM_FIXED: u32 = 32;
const ERR_TOO_SOON: u32 = 33;

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
        fn sol_sha256(vals: *const SolBytes, val_len: u64, hash_result: *mut u8) -> u64;
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
    pub fn sha256(parts: &[&[u8]], out: &mut [u8; 32]) {
        let mut sb: [SolBytes; 4] = [NULL_BYTES, NULL_BYTES, NULL_BYTES, NULL_BYTES];
        for (i, p) in parts.iter().enumerate() {
            sb[i] = SolBytes { addr: p.as_ptr(), len: p.len() as u64 };
        }
        unsafe {
            sol_sha256(sb.as_ptr(), parts.len() as u64, out.as_mut_ptr());
        }
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
        let mut m: [SolAccountMeta; 8] = [NULL_META; 8];
        let mut infos: [SolAccountInfo; 8] = [info_of(&accs[0]); 8];
        for (i, (idx, w, s)) in cpi.metas.iter().enumerate() {
            m[i] = SolAccountMeta { pubkey: accs[*idx].key, is_writable: *w, is_signer: *s };
            infos[i] = info_of(&accs[*idx]);
        }
        let mut seed_bytes: [[SolBytes; 4]; 2] =
            [[NULL_BYTES, NULL_BYTES, NULL_BYTES, NULL_BYTES], [NULL_BYTES, NULL_BYTES, NULL_BYTES, NULL_BYTES]];
        let mut signers: [SolSignerSeeds; 2] =
            [SolSignerSeeds { addr: core::ptr::null(), len: 0 }, SolSignerSeeds { addr: core::ptr::null(), len: 0 }];
        for (si, seeds) in cpi.seeds.iter().enumerate() {
            for (j, s) in seeds.iter().enumerate() {
                seed_bytes[si][j] = SolBytes { addr: s.as_ptr(), len: s.len() as u64 };
            }
            signers[si] = SolSignerSeeds { addr: seed_bytes[si].as_ptr(), len: seeds.len() as u64 };
        }
        let ix = SolInstruction {
            program_id: cpi.program,
            accounts: m.as_ptr(),
            account_len: cpi.metas.len() as u64,
            data: cpi.data.as_ptr(),
            data_len: cpi.data.len() as u64,
        };
        let rc = unsafe {
            sol_invoke_signed_c(&ix, infos.as_ptr(), cpi.metas.len() as u64, signers.as_ptr(), cpi.seeds.len() as u64)
        };
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

use sys::{clock, find_pda, invoke, log, sha256};

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
fn is_zero32(a: &[u8]) -> bool {
    let mut d = 0u8;
    for i in 0..32 {
        d |= a[i];
    }
    d == 0
}
fn rd16(d: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([d[o], d[o + 1]])
}
fn wr16(d: &mut [u8], o: usize, v: u16) {
    d[o..o + 2].copy_from_slice(&v.to_le_bytes());
}
fn rd32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}
fn wr32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
fn rd64(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3], d[o + 4], d[o + 5], d[o + 6], d[o + 7]])
}
fn wr64(d: &mut [u8], o: usize, v: u64) {
    d[o..o + 8].copy_from_slice(&v.to_le_bytes());
}

struct Ctx<'a> {
    accs: &'a [Acc],
    n: usize,
    ix: &'a [u8],
    program_id: &'a [u8; 32],
}

// ---------------------------------------------------------------- account checks
fn seed_of(kind: u8) -> Option<&'static [u8]> {
    match kind {
        KIND_REG => Some(REG_SEED),
        KIND_ACH => Some(ACH_SEED),
        KIND_REL => Some(REL_SEED),
        _ => None,
    }
}

/// The address of one of this program's ledger accounts.
fn ledger_address(kind: u8, idx: u8, program_id: &[u8; 32]) -> ([u8; 32], u8) {
    let seed = seed_of(kind).unwrap_or(REG_SEED);
    if kind == KIND_REG {
        find_pda(&[seed], program_id)
    } else {
        find_pda(&[seed, &[idx]], program_id)
    }
}

/// Checks that `a` is a genuine ledger account of `kind`, owned by this program, with the right size, header and
/// address. Returns its shard index (0 for the registry).
fn check_ledger(c: &Ctx, a: &Acc, kind: u8, want_writable: bool) -> Result<u8, u32> {
    if want_writable && !a.is_writable {
        return Err(ERR_ACCOUNTS);
    }
    if !eq32(a.owner(), c.program_id) {
        return Err(ERR_OWNER);
    }
    let want = if kind == KIND_REG { REG_SIZE } else { SHARD_SIZE };
    let d = a.data();
    if d.len() != want || &d[..8] != MAGIC_LEDGER || d[H_VERSION] != VERSION || d[H_KIND] != kind {
        return Err(ERR_DATA);
    }
    let idx = d[H_IDX];
    if kind != KIND_REG && idx as usize >= SHARDS {
        return Err(ERR_DATA);
    }
    let (key, _) = ledger_address(kind, idx, c.program_id);
    if !eq32(&key, a.key()) {
        return Err(ERR_PDA);
    }
    Ok(idx)
}

fn operator_with(op: &[u8; 32], a: &Acc) -> Result<(), u32> {
    if is_zero32(op) {
        return Err(ERR_NOT_CONFIGURED);
    }
    if !a.is_signer || !eq32(a.key(), op) {
        return Err(ERR_SIGNER);
    }
    Ok(())
}

fn operator(_c: &Ctx, a: &Acc) -> Result<(), u32> {
    operator_with(&OPERATOR, a)
}

// ---------------------------------------------------------------- registry
fn game_entry<'a>(reg: &'a [u8], game_id: &[u8]) -> Option<&'a [u8]> {
    let n = (reg[R_NGAMES] as usize).min(MAX_GAMES);
    (0..n).map(|i| &reg[R_GAMES + i * GAME_ENTRY..R_GAMES + (i + 1) * GAME_ENTRY]).find(|e| &e[..16] == game_id)
}

/// The relic a 32 byte random value draws. Sixteen little-endian u16 values are tried in order; the first below 60,000
/// is reduced modulo 10,000, so every residue has exactly six sources and the result is exactly uniform. The kind is the
/// first whose running total of basis points exceeds that number.
fn draw(reg: &[u8], rnd: &[u8]) -> Result<(u8, u8), u32> {
    let mut v: u32 = 0;
    let mut ok = false;
    for i in 0..16 {
        let x = rd16(rnd, i * 2) as u32;
        if x < DRAW_LIMIT {
            v = x % ODDS_TOTAL;
            ok = true;
            break;
        }
    }
    if !ok {
        return Err(ERR_DRAW);
    }
    let n = (reg[R_NKINDS] as usize).min(MAX_KINDS);
    let mut cum = 0u32;
    for k in 0..n {
        let o = R_KINDS + k * KIND_ENTRY;
        cum += rd16(reg, o + 2) as u32;
        if v < cum {
            return Ok((reg[o], reg[o + 1]));
        }
    }
    Err(ERR_ODDS)
}

fn ix_register_game(c: &Ctx) -> Result<(), u32> {
    if c.n < 2 || c.ix.len() != 1 + 16 + 32 + 1 {
        return Err(ERR_DATA);
    }
    operator(c, &c.accs[0])?;
    check_ledger(c, &c.accs[1], KIND_REG, true)?;
    let reg = c.accs[1].data();
    let (id, program) = (&c.ix[1..17], &c.ix[17..49]);
    let enabled = c.ix[49];
    if enabled > 1 || is_zero32(program) {
        return Err(ERR_DATA);
    }
    let n = (reg[R_NGAMES] as usize).min(MAX_GAMES);
    if let Some(i) = (0..n).find(|i| &reg[R_GAMES + i * GAME_ENTRY..R_GAMES + i * GAME_ENTRY + 16] == id) {
        let o = R_GAMES + i * GAME_ENTRY;
        // a registered game's program never changes: it can only be switched on or off
        if !eq32(&reg[o + 16..o + 48], program) {
            return Err(ERR_PROGRAM_FIXED);
        }
        reg[o + 48] = enabled;
    } else {
        if n >= MAX_GAMES {
            return Err(ERR_CAPACITY);
        }
        let o = R_GAMES + n * GAME_ENTRY;
        reg[o..o + 16].copy_from_slice(id);
        reg[o + 16..o + 48].copy_from_slice(program);
        reg[o + 48] = enabled;
        reg[R_NGAMES] = (n + 1) as u8;
    }
    reg[H_DIRTY] = 1;
    log("play: game registered");
    Ok(())
}

fn ix_set_odds(c: &Ctx) -> Result<(), u32> {
    if c.n < 2 || c.ix.len() < 2 {
        return Err(ERR_DATA);
    }
    operator(c, &c.accs[0])?;
    check_ledger(c, &c.accs[1], KIND_REG, true)?;
    let reg = c.accs[1].data();
    if reg[R_FROZEN] == 1 {
        return Err(ERR_FROZEN);
    }
    let n = c.ix[1] as usize;
    if n == 0 || n > MAX_KINDS || c.ix.len() != 2 + n * KIND_ENTRY {
        return Err(ERR_DATA);
    }
    let mut total = 0u32;
    for i in 0..n {
        let o = 2 + i * KIND_ENTRY;
        let w = rd16(c.ix, o + 2) as u32;
        if w == 0 {
            return Err(ERR_ODDS);
        }
        total += w;
        if (0..i).any(|j| c.ix[2 + j * KIND_ENTRY] == c.ix[o]) {
            return Err(ERR_ODDS);
        }
    }
    if total != ODDS_TOTAL {
        return Err(ERR_ODDS);
    }
    for b in reg[R_KINDS..R_KINDS + MAX_KINDS * KIND_ENTRY].iter_mut() {
        *b = 0;
    }
    reg[R_KINDS..R_KINDS + n * KIND_ENTRY].copy_from_slice(&c.ix[2..2 + n * KIND_ENTRY]);
    reg[R_NKINDS] = n as u8;
    reg[H_DIRTY] = 1;
    Ok(())
}

fn ix_freeze_odds(c: &Ctx) -> Result<(), u32> {
    if c.n < 2 || c.ix.len() != 1 {
        return Err(ERR_DATA);
    }
    operator(c, &c.accs[0])?;
    check_ledger(c, &c.accs[1], KIND_REG, true)?;
    let reg = c.accs[1].data();
    if reg[R_FROZEN] == 1 {
        return Err(ERR_FROZEN);
    }
    let n = (reg[R_NKINDS] as usize).min(MAX_KINDS);
    let total: u32 = (0..n).map(|k| rd16(reg, R_KINDS + k * KIND_ENTRY + 2) as u32).sum();
    if n == 0 || total != ODDS_TOTAL {
        return Err(ERR_ODDS);
    }
    reg[R_FROZEN] = 1;
    reg[H_DIRTY] = 1;
    log("play: odds frozen");
    Ok(())
}

// ---------------------------------------------------------------- saving to Solana
/// Asks the rollup to commit one ledger account. Callers decide what a missing reserve means.
fn schedule_commit(c: &Ctx, pda: usize, context: usize, vault: usize) -> Result<(), u32> {
    let a = &c.accs[pda];
    let d = a.data();
    let (kind, idx) = (d[H_KIND], d[H_IDX]);
    let (fee_vault, _) = find_pda(&[b"magic-fee-vault", &VALIDATOR], &DELEGATION_PROGRAM);
    if !eq32(c.accs[vault].key(), &fee_vault) || !eq32(c.accs[context].key(), &MAGIC_CONTEXT) {
        return Err(ERR_ACCOUNTS);
    }
    let (_, bump) = ledger_address(kind, idx, c.program_id);
    let (idx_b, bump_b) = ([idx], [bump]);
    let metas: &[(usize, bool, bool)] = &[(pda, true, true), (context, true, false), (vault, true, false), (pda, true, false)];
    let reg_seeds: &[&[u8]] = &[REG_SEED, &bump_b];
    let shard_seeds: &[&[u8]] = &[if kind == KIND_ACH { ACH_SEED } else { REL_SEED }, &idx_b, &bump_b];
    let seeds: &[&[&[u8]]] = if kind == KIND_REG { &[reg_seeds] } else { &[shard_seeds] };
    invoke(c.accs, &Cpi { program: &MAGIC_PROGRAM, metas, data: &[1, 0, 0, 0], seeds })
}

fn reserve_ok(a: &Acc) -> bool {
    let size = a.len();
    a.lamports() >= rent_minimum(size) + SPONSOR_MARGIN
}

/// After a state change: mark the account as holding unsaved changes and, when its reserve allows, schedule the save
/// in the same transaction. When it does not, the change stands and anyone can call Commit later.
fn mark_and_save(c: &Ctx, pda: usize, context: usize, vault: usize) -> Result<(), u32> {
    let d = c.accs[pda].data();
    d[H_DIRTY] = 1;
    if c.n > vault && reserve_ok(&c.accs[pda]) {
        d[H_DIRTY] = 0;
        let (_, now) = clock();
        wr32(d, H_SAVED_TS, now as u32);
        schedule_commit(c, pda, context, vault)?;
    } else {
        log("play: save not scheduled");
    }
    Ok(())
}

fn ix_commit(c: &Ctx) -> Result<(), u32> {
    // [payer s, pda w, magic program, context w, vault w]; data: tag, optional force byte
    // Without force: only while the account has unsaved changes. With force = 1: also when it is clean, but at most
    // once per RESAVE_AFTER_SECS, which lets anyone retry a save that never reached the base layer.
    if c.n < 5 || c.ix.is_empty() || c.ix.len() > 2 || !c.accs[0].is_signer || !c.accs[1].is_writable {
        return Err(ERR_ACCOUNTS);
    }
    let force = c.ix.len() == 2 && c.ix[1] == 1;
    if c.ix.len() == 2 && c.ix[1] > 1 {
        return Err(ERR_DATA);
    }
    if !eq32(c.accs[2].key(), &MAGIC_PROGRAM) {
        return Err(ERR_ACCOUNTS);
    }
    let kind = c.accs[1].data().get(H_KIND).copied().unwrap_or(9);
    if kind > KIND_REL {
        return Err(ERR_DATA);
    }
    check_ledger(c, &c.accs[1], kind, true)?;
    let d = c.accs[1].data();
    let (_, now) = clock();
    if d[H_DIRTY] != 1 {
        if !force {
            return Err(ERR_NOTHING_TO_SAVE);
        }
        let last = rd32(d, H_SAVED_TS) as i64;
        if last != 0 && now < last + RESAVE_AFTER_SECS {
            return Err(ERR_TOO_SOON);
        }
    }
    if !reserve_ok(&c.accs[1]) {
        return Err(ERR_SPONSOR_LOW);
    }
    d[H_DIRTY] = 0;
    wr32(d, H_SAVED_TS, now as u32);
    schedule_commit(c, 1, 3, 4)
}

fn ix_commit_undelegate(c: &Ctx) -> Result<(), u32> {
    // [operator s w, pda w, magic program, context w]; data: tag, kind, idx
    if c.n < 4 || c.ix.len() != 3 {
        return Err(ERR_DATA);
    }
    operator(c, &c.accs[0])?;
    if !eq32(c.accs[2].key(), &MAGIC_PROGRAM) || !eq32(c.accs[3].key(), &MAGIC_CONTEXT) {
        return Err(ERR_ACCOUNTS);
    }
    let (kind, idx) = (c.ix[1], c.ix[2]);
    if seed_of(kind).is_none() {
        return Err(ERR_DATA);
    }
    let (key, _) = ledger_address(kind, idx, c.program_id);
    if !eq32(&key, c.accs[1].key()) {
        return Err(ERR_PDA);
    }
    invoke(
        c.accs,
        &Cpi { program: &MAGIC_PROGRAM, metas: &[(0, c.accs[0].is_writable, true), (3, true, false), (1, true, false)], data: &[2, 0, 0, 0], seeds: &[] },
    )?;
    log("play: commit and undelegate scheduled");
    Ok(())
}

// ---------------------------------------------------------------- achievements
/// The A1 rule against Magic Over's table, eased on 23 Nov 2026 ("Boundary Hitter"): the table must be finished
/// with a result (side A, side B or a tie); the claimant must be one of its two players; that player's side must have
/// hit at least one four or six over the match. Winning is no longer needed. A match against the AI counts (decision
/// of 20 Nov 2026: the achievement is free and has no financial value). Only a person holds a key, so the AI side (the
/// zero key) can never claim. Returns the match number the claim names.
fn six_hitter(table: &[u8], claimant: &[u8; 32]) -> Result<u32, u32> {
    if table.len() != T_SIZE || &table[..8] != T_MAGIC {
        return Err(ERR_TABLE);
    }
    if table[T_PHASE] != T_FINISHED || !matches!(table[T_WINNER], 1 | 2 | 3) {
        return Err(ERR_NOT_FINISHED);
    }
    let side = (0..2usize)
        .find(|&s| {
            let key = &table[T_A + s * 32..T_A + s * 32 + 32];
            !is_zero32(key) && eq32(key, claimant)
        })
        .ok_or(ERR_NOT_WINNER)?;
    if table[T_TOTALS_ON] != 1 {
        return Err(ERR_NO_TOTALS);
    }
    let boundaries = rd16(table, T_TOTALS + side * 8 + 2) as u32 + rd16(table, T_TOTALS + side * 8 + 4) as u32;
    if boundaries < 1 {
        return Err(ERR_NO_SIXES);
    }
    let m = rd32(table, T_MATCH);
    if m == 0 {
        return Err(ERR_TABLE);
    }
    Ok(m)
}

/// Where a (player, game, achievement) is recorded, if anywhere, scanning the four shards given.
fn find_claim(shards: [&[u8]; SHARDS], player: &[u8], game: &[u8], ach: u16) -> Option<(usize, usize)> {
    for (si, d) in shards.iter().enumerate() {
        for s in 0..ACH_PER {
            let o = HEADER + s * ACH_SLOT;
            if d[o + A_STATE] == 1 && rd16(d, o + A_ID) == ach && &d[o + A_GAME..o + A_GAME + 16] == game && &d[o + A_PLAYER..o + A_PLAYER + 32] == player {
                return Some((si, s));
            }
        }
    }
    None
}

fn first_free(d: &[u8], per: usize, slot: usize) -> Option<usize> {
    (0..per).find(|s| d[HEADER + s * slot] == 0)
}

fn ix_claim(c: &Ctx) -> Result<(), u32> {
    // [player s, registry, ach0 w, ach1 w, ach2 w, ach3 w, game table, magic, context w, vault w]
    if c.n < 10 || !matches!(c.ix.len(), 4 | 8) {
        return Err(ERR_DATA);
    }
    let player = &c.accs[0];
    if !player.is_signer {
        return Err(ERR_SIGNER);
    }
    let ach = rd16(c.ix, 1);
    let table_idx = c.ix[3];
    if ach != ACH_SIX_HITTER {
        return Err(ERR_NO_ACH);
    }
    if table_idx >= T_MAX_IDX {
        return Err(ERR_RANGE);
    }
    check_ledger(c, &c.accs[1], KIND_REG, false)?;
    for i in 0..SHARDS {
        if check_ledger(c, &c.accs[2 + i], KIND_ACH, true)? as usize != i {
            return Err(ERR_ACCOUNTS);
        }
    }
    let reg = c.accs[1].data();
    let entry = game_entry(reg, &GAME_MAGIC_OVER).ok_or(ERR_GAME_OFF)?;
    if entry[48] != 1 {
        return Err(ERR_GAME_OFF);
    }
    let mut game_program = [0u8; 32];
    game_program.copy_from_slice(&entry[16..48]);
    // the table must be owned by the registered game program and be that program's own table PDA
    let table = &c.accs[6];
    if !eq32(table.owner(), &game_program) {
        return Err(ERR_OWNER);
    }
    let (tkey, _) = find_pda(&[T_SEED, &[table_idx]], &game_program);
    if !eq32(&tkey, table.key()) {
        return Err(ERR_PDA);
    }
    let mut who = [0u8; 32];
    who.copy_from_slice(player.key());
    let matchno = six_hitter(table.data(), &who)?;
    if c.ix.len() == 8 && (rd32(c.ix, 4) != matchno || table.data()[10] != 0) {
        return Err(ERR_TABLE);
    }
    let shards = [c.accs[2].data() as &[u8], c.accs[3].data(), c.accs[4].data(), c.accs[5].data()];
    if find_claim(shards, &who, &GAME_MAGIC_OVER, ach).is_some() {
        return Err(ERR_ALREADY_CLAIMED);
    }
    let (si, s) = (0..SHARDS)
        .find_map(|i| first_free(c.accs[2 + i].data(), ACH_PER, ACH_SLOT).map(|s| (i, s)))
        .ok_or(ERR_CAPACITY)?;
    let (_, now) = clock();
    let d = c.accs[2 + si].data();
    let o = HEADER + s * ACH_SLOT;
    for b in d[o..o + ACH_SLOT].iter_mut() {
        *b = 0;
    }
    d[o + A_STATE] = 1;
    d[o + A_FLAGS] = F_ELIGIBLE;
    wr16(d, o + A_ID, ach);
    d[o + A_GAME..o + A_GAME + 16].copy_from_slice(&GAME_MAGIC_OVER);
    d[o + A_PLAYER..o + A_PLAYER + 32].copy_from_slice(&who);
    // event_ref: the table's match number, then its index
    wr32(d, o + A_EVENT, matchno);
    d[o + A_EVENT + 4] = table_idx;
    wr64(d, o + A_ISSUED, now as u64);
    log("play: achievement recorded");
    mark_and_save(c, 2 + si, 8, 9)
}

// ---------------------------------------------------------------- relic request and callback
fn ix_request_relic(c: &Ctx) -> Result<(), u32> {
    // [player s, ach shard w, identity, vrf program, queue w, system, slot hashes, registry, optional payer s w]
    // data: tag, slot u16, optional settle byte (1 = the callback also schedules the save of the relic shard it writes)
    // The player authorizes the request. The optional ninth account only PAYS for the VRF call (a sponsor, as the
    // rollup refuses a writable account that is not the fee payer); it is not an identity and is never read as one.
    // Without it the player pays, exactly as before, and the player must then be writable.
    if c.n < 8 || (c.ix.len() != 3 && c.ix.len() != 4) {
        return Err(ERR_DATA);
    }
    let settle = c.ix.len() == 4;
    if settle && c.ix[3] != 1 {
        return Err(ERR_DATA);
    }
    let player = &c.accs[0];
    if !player.is_signer {
        return Err(ERR_SIGNER);
    }
    let payer = if c.n >= 9 { 8 } else { 0 };
    if payer == 8 && (!c.accs[8].is_signer || !c.accs[8].is_writable) {
        return Err(ERR_SIGNER);
    }
    let slot = rd16(c.ix, 1) as usize;
    if slot >= ACH_PER {
        return Err(ERR_RANGE);
    }
    let shard_idx = check_ledger(c, &c.accs[1], KIND_ACH, true)?;
    // odds must be frozen before a relic can be requested, so no VRF fee is spent on a draw that cannot be placed
    check_ledger(c, &c.accs[7], KIND_REG, false)?;
    if c.accs[7].data()[R_FROZEN] != 1 {
        return Err(ERR_NOT_FROZEN);
    }
    let reg_key = *c.accs[7].key();
    let (identity, id_bump) = find_pda(&[IDENTITY_SEED], c.program_id);
    if !eq32(&identity, c.accs[2].key())
        || !eq32(c.accs[3].key(), &VRF_PROGRAM)
        || !c.accs[3].executable
        || !eq32(c.accs[4].key(), &VRF_QUEUE)
        || !eq32(c.accs[4].owner(), &VRF_PROGRAM)
        || !c.accs[4].is_writable
        || !eq32(c.accs[5].key(), &SYSTEM_PROGRAM)
        || !eq32(c.accs[6].key(), &SLOT_HASHES)
    {
        return Err(ERR_ACCOUNTS);
    }
    let (slot_now, now) = clock();
    let d = c.accs[1].data();
    let o = HEADER + slot * ACH_SLOT;
    if d[o + A_STATE] != 1 {
        return Err(ERR_NO_ACH);
    }
    if !eq32(&d[o + A_PLAYER..o + A_PLAYER + 32], player.key()) {
        return Err(ERR_NOT_OWNER);
    }
    let seq = match d[o + A_FLAGS] {
        F_ELIGIBLE => 1u32,
        F_PENDING => {
            // a pending request may be replaced, by its player only, after a pause
            if now < rd64(d, o + A_REQ_AT) as i64 + REPLACE_AFTER_SECS {
                return Err(ERR_TOO_EARLY);
            }
            rd32(d, o + A_REQ_SEQ).checked_add(1).ok_or(ERR_RANGE)?
        }
        _ => return Err(ERR_MINTED),
    };
    d[o + A_FLAGS] = F_PENDING;
    wr32(d, o + A_REQ_SEQ, seq);
    wr64(d, o + A_REQ_AT, now as u64);
    d[H_DIRTY] = 1;
    // the seed comes from program state only: the shard, the slot, the sequence and the rollup slot
    let mut ss = [0u8; 7];
    ss[0] = shard_idx;
    ss[1..3].copy_from_slice(&(slot as u16).to_le_bytes());
    ss[3..7].copy_from_slice(&seq.to_le_bytes());
    let mut seed = [0u8; 32];
    sha256(&[c.accs[1].key(), &ss, &slot_now.to_le_bytes()], &mut seed);
    request_vrf(c, &seed, &reg_key, shard_idx, slot as u16, seq, id_bump, settle, payer)
}

/// RequestRandomness. The callback's accounts and data are fixed here, before any relic exists, so every relic shard is
/// named now; the callback chooses among them.
fn request_vrf(c: &Ctx, seed: &[u8; 32], reg_key: &[u8; 32], shard: u8, slot: u16, seq: u32, id_bump: u8, settle: bool, payer: usize) -> Result<(), u32> {
    let mut data = [0u8; 8 + 32 + 32 + 4 + 8 + 4 + 9 * 34 + 4 + 7];
    data[0] = 10;
    let mut o = 8;
    data[o..o + 32].copy_from_slice(seed);
    o += 32;
    data[o..o + 32].copy_from_slice(c.program_id);
    o += 32;
    data[o..o + 4].copy_from_slice(&8u32.to_le_bytes());
    o += 4;
    data[o..o + 8].copy_from_slice(&VRF_CALLBACK);
    o += 8;
    data[o..o + 4].copy_from_slice(&(if settle { 9u32 } else { 6u32 }).to_le_bytes());
    o += 4;
    let mut put = |key: &[u8; 32], writable: u8| {
        data[o..o + 32].copy_from_slice(key);
        data[o + 32] = 0;
        data[o + 33] = writable;
        o += 34;
    };
    put(c.accs[1].key(), 1);
    put(reg_key, 0);
    for i in 0..SHARDS as u8 {
        let (k, _) = ledger_address(KIND_REL, i, c.program_id);
        put(&k, 1);
    }
    if settle {
        // the callback can then schedule the save of the relic shard it writes: Magic program, context, fee vault
        let (fee_vault, _) = find_pda(&[b"magic-fee-vault", &VALIDATOR], &DELEGATION_PROGRAM);
        put(&MAGIC_PROGRAM, 0);
        put(&MAGIC_CONTEXT, 1);
        put(&fee_vault, 1);
    }
    data[o..o + 4].copy_from_slice(&7u32.to_le_bytes());
    o += 4;
    data[o] = shard;
    data[o + 1..o + 3].copy_from_slice(&slot.to_le_bytes());
    data[o + 3..o + 7].copy_from_slice(&seq.to_le_bytes());
    o += 7;
    let id_bump_b = [id_bump];
    invoke(
        c.accs,
        &Cpi {
            program: &VRF_PROGRAM,
            metas: &[(payer, true, true), (2, false, true), (4, true, false), (5, false, false), (6, false, false)],
            data: &data[..o],
            seeds: &[&[IDENTITY_SEED, &id_bump_b]],
        },
    )
}

/// Places a minted relic. It starts at shard relic_id[0] mod 4 and moves on, wrapping, when a shard is full. Returns
/// the (shard, slot) it was written to, or the capacity error with nothing changed.
fn mint(accs: &[Acc], base: usize, relic_id: &[u8], kind: u8, rarity: u8, owner: &[u8], now: i64) -> Result<(usize, usize), u32> {
    if is_zero32(relic_id) || (0..SHARDS).any(|i| {
        accs[base + i].data()[HEADER..].chunks_exact(REL_SLOT)
            .any(|r| matches!(r[L_STATE], 1 | vault::MOVED) && &r[L_ID..L_ID + 32] == relic_id)
    }) {
        return Err(ERR_DATA);
    }
    let start = relic_id[0] as usize % SHARDS;
    for k in 0..SHARDS {
        let si = (start + k) % SHARDS;
        let d = accs[base + si].data();
        if let Some(s) = first_free(d, REL_PER, REL_SLOT) {
            let o = HEADER + s * REL_SLOT;
            for b in d[o..o + REL_SLOT].iter_mut() {
                *b = 0;
            }
            d[o + L_STATE] = 1;
            d[o + L_KIND] = kind;
            d[o + L_RARITY] = rarity;
            wr32(d, o + L_COUNT, 0);
            d[o + L_ID..o + L_ID + 32].copy_from_slice(relic_id);
            d[o + L_OWNER..o + L_OWNER + 32].copy_from_slice(owner);
            d[o + L_ORIGIN..o + L_ORIGIN + 32].copy_from_slice(owner);
            wr64(d, o + L_MINTED, now as u64);
            d[H_DIRTY] = 1;
            return Ok((si, s));
        }
    }
    Err(ERR_CAPACITY)
}

fn ix_vrf_callback(c: &Ctx) -> Result<(), u32> {
    // [vrf identity s, ach shard w, registry, rel0 w, rel1 w, rel2 w, rel3 w]; data: disc, randomness 32, shard, slot u16, seq u32
    if !matches!(c.n, 7 | 10) || c.ix.len() != 8 + 32 + 7 {
        return Err(ERR_DATA);
    }
    let (expected, _) = find_pda(&[IDENTITY_SEED, c.program_id], &VRF_PROGRAM);
    if !c.accs[0].is_signer || !eq32(&expected, c.accs[0].key()) {
        return Err(ERR_IDENTITY);
    }
    let rnd = &c.ix[8..40];
    let (shard, slot, seq) = (c.ix[40], rd16(c.ix, 41) as usize, rd32(c.ix, 43));
    let idx = check_ledger(c, &c.accs[1], KIND_ACH, true)?;
    if idx != shard || slot >= ACH_PER {
        return Err(ERR_DATA);
    }
    check_ledger(c, &c.accs[2], KIND_REG, false)?;
    for i in 0..SHARDS {
        if check_ledger(c, &c.accs[3 + i], KIND_REL, true)? as usize != i {
            return Err(ERR_ACCOUNTS);
        }
    }
    let reg = c.accs[2].data();
    if reg[R_FROZEN] != 1 {
        return Err(ERR_NOT_FROZEN);
    }
    let d = c.accs[1].data();
    let o = HEADER + slot * ACH_SLOT;
    if d[o + A_STATE] != 1 || d[o + A_FLAGS] != F_PENDING {
        return Err(ERR_NOT_PENDING);
    }
    if rd32(d, o + A_REQ_SEQ) != seq {
        return Err(ERR_SEQ);
    }
    // callback layout with save accounts: [.., rel3, magic program, context, fee vault]. Check them before anything is written.
    let settle = c.n >= 10;
    if settle {
        let (fee_vault, _) = find_pda(&[b"magic-fee-vault", &VALIDATOR], &DELEGATION_PROGRAM);
        if !eq32(c.accs[7].key(), &MAGIC_PROGRAM) || !eq32(c.accs[8].key(), &MAGIC_CONTEXT) || !eq32(c.accs[9].key(), &fee_vault) {
            return Err(ERR_ACCOUNTS);
        }
    }
    let (kind, rarity) = draw(reg, rnd)?;
    let mut owner = [0u8; 32];
    owner.copy_from_slice(&d[o + A_PLAYER..o + A_PLAYER + 32]);
    let (_, now) = clock();
    let (placed, _) = mint(c.accs, 3, rnd, kind, rarity, &owner, now)?;
    d[o + A_FLAGS] = F_MINTED;
    d[H_DIRTY] = 1;
    log("play: relic minted");
    if settle {
        // the relic shard is what a verifier reads; the achievement shard stays marked unsaved for the next save
        mark_and_save(c, 3 + placed, 8, 9)?;
    }
    Ok(())
}

// ---------------------------------------------------------------- transfer
fn ix_transfer(c: &Ctx) -> Result<(), u32> {
    // [owner s, relic shard w, magic, context w, vault w]; data: tag, slot u16, new owner 32
    if c.n < 5 || c.ix.len() != 1 + 2 + 32 {
        return Err(ERR_DATA);
    }
    let owner = &c.accs[0];
    if !owner.is_signer {
        return Err(ERR_SIGNER);
    }
    check_ledger(c, &c.accs[1], KIND_REL, true)?;
    let slot = rd16(c.ix, 1) as usize;
    if slot >= REL_PER {
        return Err(ERR_RANGE);
    }
    let new_owner = &c.ix[3..35];
    if is_zero32(new_owner) {
        return Err(ERR_DATA);
    }
    let d = c.accs[1].data();
    let o = HEADER + slot * REL_SLOT;
    if d[o + L_STATE] != 1 {
        return Err(ERR_NO_ACH);
    }
    if !eq32(&d[o + L_OWNER..o + L_OWNER + 32], owner.key()) {
        return Err(ERR_NOT_OWNER);
    }
    if eq32(new_owner, owner.key()) {
        return Err(ERR_SAME_OWNER);
    }
    let count = rd32(d, o + L_COUNT).checked_add(1).ok_or(ERR_RANGE)?;
    d[o + L_OWNER..o + L_OWNER + 32].copy_from_slice(new_owner);
    wr32(d, o + L_COUNT, count);
    log("play: relic transferred");
    mark_and_save(c, 1, 3, 4)
}

// ---------------------------------------------------------------- account creation and delegation
fn ix_init(c: &Ctx) -> Result<(), u32> {
    // [operator s w, pda w, system]; data: tag, kind, idx, lamports u64
    if c.n < 3 || c.ix.len() != 11 {
        return Err(ERR_DATA);
    }
    operator(c, &c.accs[0])?;
    let (kind, idx) = (c.ix[1], c.ix[2]);
    if seed_of(kind).is_none() || (kind != KIND_REG && idx as usize >= SHARDS) || (kind == KIND_REG && idx != 0) {
        return Err(ERR_DATA);
    }
    let (key, bump) = ledger_address(kind, idx, c.program_id);
    if !eq32(&key, c.accs[1].key()) || !eq32(c.accs[2].key(), &SYSTEM_PROGRAM) {
        return Err(ERR_PDA);
    }
    let space = if kind == KIND_REG { REG_SIZE } else { SHARD_SIZE };
    let mut data = [0u8; 52];
    data[4..12].copy_from_slice(&c.ix[3..11]);
    data[12..20].copy_from_slice(&(space as u64).to_le_bytes());
    data[20..52].copy_from_slice(c.program_id);
    let (idx_b, bump_b) = ([idx], [bump]);
    let seed = seed_of(kind).unwrap_or(REG_SEED);
    let reg_seeds: &[&[u8]] = &[REG_SEED, &bump_b];
    let shard_seeds: &[&[u8]] = &[seed, &idx_b, &bump_b];
    let seeds: &[&[&[u8]]] = if kind == KIND_REG { &[reg_seeds] } else { &[shard_seeds] };
    invoke(c.accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(0, true, true), (1, true, true)], data: &data, seeds })?;
    let d = c.accs[1].data();
    d[..8].copy_from_slice(MAGIC_LEDGER);
    d[H_KIND] = kind;
    d[H_VERSION] = VERSION;
    d[H_IDX] = idx;
    if kind == KIND_REG {
        d[R_ADMIN..R_ADMIN + 32].copy_from_slice(c.accs[0].key());
    }
    log("play: account created");
    Ok(())
}

#[inline(never)]
fn ix_delegate(c: &Ctx) -> Result<(), u32> {
    // [payer s w, pda w, this program, buffer w, record w, metadata w, delegation program, system]
    // data: tag, kind, idx, has_validator, validator 32, commit_ms u32
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 8 || ix.len() != 40 {
        return Err(ERR_ACCOUNTS);
    }
    operator(c, &accs[0])?;
    let (payer, pda, buffer) = (&accs[0], &accs[1], &accs[3]);
    if !eq32(accs[2].key(), program_id) || !eq32(accs[6].key(), &DELEGATION_PROGRAM) {
        return Err(ERR_ACCOUNTS);
    }
    if !eq32(pda.owner(), program_id) {
        return Err(ERR_OWNER);
    }
    let (kind, idx) = (ix[1], ix[2]);
    let seed = seed_of(kind).ok_or(ERR_DATA)?;
    if (kind != KIND_REG && idx as usize >= SHARDS) || (kind == KIND_REG && idx != 0) {
        return Err(ERR_DATA);
    }
    let (key, bump) = ledger_address(kind, idx, program_id);
    if !eq32(&key, pda.key()) {
        return Err(ERR_PDA);
    }
    let (idx_b, bump_b) = ([idx], [bump]);
    let reg_seeds: &[&[u8]] = &[REG_SEED, &bump_b];
    let shard_seeds: &[&[u8]] = &[seed, &idx_b, &bump_b];
    let pda_seeds: &[&[&[u8]]] = if kind == KIND_REG { &[reg_seeds] } else { &[shard_seeds] };
    let (buf_key, buf_bump) = find_pda(&[b"buffer", pda.key()], program_id);
    if !eq32(&buf_key, buffer.key()) {
        return Err(ERR_PDA);
    }
    let buf_bump_b = [buf_bump];
    let len = pda.len();
    let mut data = [0u8; 52];
    data[12..20].copy_from_slice(&(len as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(0, true, true), (3, true, true)], data: &data, seeds: &[&[b"buffer", pda.key(), &buf_bump_b]] })?;
    {
        let src = pda.data();
        buffer.data().copy_from_slice(src);
        for x in src.iter_mut() {
            *x = 0;
        }
    }
    unsafe {
        *pda.owner = SYSTEM_PROGRAM;
    }
    let mut assign = [0u8; 36];
    assign[0] = 1;
    assign[4..36].copy_from_slice(&DELEGATION_PROGRAM);
    invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(1, true, true)], data: &assign, seeds: pda_seeds })?;
    // delegation arguments: commit_ms, seeds (one or two), optional validator
    let mut args = [0u8; 8 + 4 + 4 + (4 + 8) + (4 + 1) + 1 + 32];
    let mut o = 8;
    args[o..o + 4].copy_from_slice(&ix[36..40]);
    o += 4;
    let nseeds: u32 = if kind == KIND_REG { 1 } else { 2 };
    args[o..o + 4].copy_from_slice(&nseeds.to_le_bytes());
    o += 4;
    args[o..o + 4].copy_from_slice(&(seed.len() as u32).to_le_bytes());
    o += 4;
    args[o..o + seed.len()].copy_from_slice(seed);
    o += seed.len();
    if kind != KIND_REG {
        args[o..o + 4].copy_from_slice(&1u32.to_le_bytes());
        o += 4;
        args[o] = idx;
        o += 1;
    }
    if ix[3] == 1 {
        args[o] = 1;
        o += 1;
        args[o..o + 32].copy_from_slice(&ix[4..36]);
        o += 32;
    } else {
        args[o] = 0;
        o += 1;
    }
    invoke(
        accs,
        &Cpi {
            program: &DELEGATION_PROGRAM,
            metas: &[(0, true, true), (1, true, true), (2, false, false), (3, true, false), (4, true, false), (5, true, false), (7, false, false)],
            data: &args[..o],
            seeds: pda_seeds,
        },
    )?;
    unsafe {
        *payer.lamports += *buffer.lamports;
        *buffer.lamports = 0;
        *buffer.data_len = 0;
        *buffer.owner = SYSTEM_PROGRAM;
    }
    log("play: delegated");
    Ok(())
}

#[inline(never)]
fn ix_undelegate_callback(c: &Ctx) -> Result<(), u32> {
    // [pda w, buffer s, payer s w, system]; data: disc, borsh Vec<Vec<u8>> seeds
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 4 {
        return Err(ERR_ACCOUNTS);
    }
    let (pda, buffer) = (&accs[0], &accs[1]);
    if !buffer.is_signer || !eq32(buffer.owner(), &DELEGATION_PROGRAM) {
        return Err(ERR_SIGNER);
    }
    let (ub, _) = find_pda(&[b"undelegate-buffer", pda.key()], &DELEGATION_PROGRAM);
    if !eq32(&ub, buffer.key()) {
        return Err(ERR_PDA);
    }
    let a = &ix[8..];
    if a.len() < 8 {
        return Err(ERR_DATA);
    }
    let count = rd32(a, 0);
    let l0 = rd32(a, 4) as usize;
    if a.len() < 8 + l0 {
        return Err(ERR_DATA);
    }
    let seed = &a[8..8 + l0];
    let (kind, idx) = if seed == REG_SEED && count == 1 && a.len() == 8 + l0 {
        (KIND_REG, 0u8)
    } else if (seed == ACH_SEED || seed == REL_SEED) && count == 2 && a.len() == 8 + l0 + 4 + 1 && rd32(a, 8 + l0) == 1 {
        (if seed == ACH_SEED { KIND_ACH } else { KIND_REL }, a[a.len() - 1])
    } else {
        return Err(ERR_DATA);
    };
    if kind != KIND_REG && idx as usize >= SHARDS {
        return Err(ERR_DATA);
    }
    let (key, bump) = ledger_address(kind, idx, program_id);
    if !eq32(&key, pda.key()) {
        return Err(ERR_PDA);
    }
    let (idx_b, bump_b) = ([idx], [bump]);
    let reg_seeds: &[&[u8]] = &[REG_SEED, &bump_b];
    let shard_seeds: &[&[u8]] = &[seed, &idx_b, &bump_b];
    let seeds: &[&[&[u8]]] = if kind == KIND_REG { &[reg_seeds] } else { &[shard_seeds] };
    let space = buffer.len();
    let mut data = [0u8; 52];
    data[4..12].copy_from_slice(&rent_minimum(space).to_le_bytes());
    data[12..20].copy_from_slice(&(space as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(2, true, true), (0, true, true)], data: &data, seeds })?;
    pda.data().copy_from_slice(buffer.data());
    log("play: returned to Solana");
    Ok(())
}

// ---------------------------------------------------------------- entry
fn dispatch(c: &Ctx) -> Result<(), u32> {
    let ix = c.ix;
    if ix.is_empty() {
        return Err(ERR_TAG);
    }
    if ix.len() >= 8 && ix[..8] == EXTERNAL_UNDELEGATE {
        return ix_undelegate_callback(c);
    }
    if ix.len() >= 8 && ix[..8] == VRF_CALLBACK {
        return ix_vrf_callback(c);
    }
    match ix[0] {
        0 => ix_init(c),
        1 => ix_delegate(c),
        2 => ix_commit(c),
        3 => ix_commit_undelegate(c),
        4 => ix_register_game(c),
        5 => ix_set_odds(c),
        6 => ix_freeze_odds(c),
        7 => ix_claim(c),
        8 => ix_request_relic(c),
        10 => ix_transfer(c),
        11 => vault::open(c),
        12 => vault::migrate(c),
        13 => vault::transfer(c),
        _ => Err(ERR_TAG),
    }
}

/// The program's entry as the loader (or a host test) calls it. Returns 0 or the program error code.
///
/// # Safety
/// `input` must point at the loader's serialized parameters.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    let mut accs: [Acc; MAX_ACCOUNTS] = [Acc::empty(); MAX_ACCOUNTS];
    let (n, ix, program_id) = parse(input, &mut accs);
    let c = Ctx { accs: &accs, n, ix, program_id };
    match dispatch(&c) {
        Ok(()) => 0,
        Err(e) => e as u64,
    }
}

mod vault;

#[cfg(not(target_arch = "bpf"))]
mod host;
#[cfg(test)]
mod tests;
