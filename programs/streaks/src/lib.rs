//! Smaran Streaks: a daily habit check-in for anyone with a Smaran username, on MagicBlock's devnet rollup.
//!
//! Not a game. One check-in per habit per day. The day is set by this program's clock, in the time zone the player
//! started with: the offset is fixed at the first check-in, so changing it later cannot win back a missed day.
//! Streaks are public: each record shows the player's username, checked against the Smaran registry when the record is
//! made. Players never hold SOL: every check-in is a zero-fee rollup transaction signed by the player's own Smaran key.
//! A streak shows that someone checked in, not that they did the habit.
//!
//! Accounts: 4 groups, PDA ["smaran-streaks", idx], 10,176 bytes each: a 64-byte header and 79 records of 128 bytes.
//! The operator creates them on Solana (rent plus a save reserve) and delegates them to MagicBlock's devnet rollup.
//!
//! Instructions (first data byte):
//!   0 Init        base [operator s w, group w, system]                                      idx, lamports u64
//!   1 Delegate    base [operator s w, group w, this program, buffer w, record w, metadata w, delegation, system]
//!                                                                                           idx, commit_ms u32
//!   2 CheckIn     ER   [player s, group 0..3 w, registry group (, magic program, magic context w, fee vault w)]
//!                      habit slot (0-2), habit kind (1-16), offset minutes i16, name_len, name
//!                      The first check-in makes the player's record: the username must belong to the signer in the
//!                      Smaran registry, and the offset (-720..840, a multiple of 15) is fixed from then on. A new kind in
//!                      an empty slot starts that habit. With the last three accounts, and an hour since the group last
//!                      saved, the group also saves itself to Solana, paid from its own reserve.
//!   3 SetHabit    ER   [player s, group 0..3 w]                          habit slot, kind (0 removes); resets that habit
//!   4 Save        ER   [caller s, group w, magic program, magic context w, fee vault w]      idx
//!                      anyone, at most once an hour, when the group has unsaved changes; the group pays from its reserve
//!   5 Forget      ER   [player s, group 0..3 w]                          removes the signer's record
//!   8 CommitUndelegate ER [operator s w, group w, magic program, magic context w]            idx
//!   MagicBlock undelegate callback (8-byte discriminator)
#![cfg_attr(target_arch = "bpf", no_std)]
use solana_compiler_builtins as _;

#[cfg(target_arch = "bpf")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { sys::abort() }
}

// ---------------------------------------------------------------- program ids
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
// MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57: MagicBlock's devnet rollup (devnet-as.magicblock.app), where the registry lives
const VALIDATOR: [u8; 32] = [
    5, 42, 71, 171, 131, 122, 182, 12, 61, 216, 159, 196, 200, 74, 97, 158, 42, 140, 82, 47, 63, 129,
    136, 254, 226, 67, 138, 88, 169, 118, 23, 178,
];
// HDbeKypeJHXMM5Xs2RHKj8XGQgJeP4jnQ1Jiugf8mXPn: the devnet operator (creates, delegates and returns the groups)
const OPERATOR: [u8; 32] = [
    240, 245, 33, 212, 39, 39, 71, 159, 130, 37, 245, 248, 117, 196, 199, 238, 238, 219, 127, 236, 6,
    3, 251, 190, 108, 86, 112, 232, 229, 213, 0, 49,
];
// CW9AQnTKvTnBfsdUChQiYKfCPhWb7FWkUWyVxv1iaN5P: the Smaran registry
const REGISTRY: [u8; 32] = [
    170, 231, 160, 244, 115, 185, 145, 46, 33, 18, 198, 117, 183, 167, 8, 38, 234, 143, 9, 10, 45, 116,
    91, 253, 130, 232, 127, 183, 140, 34, 0, 106,
];
const EXTERNAL_UNDELEGATE: [u8; 8] = [196, 28, 41, 206, 48, 37, 51, 167];

// ---------------------------------------------------------------- the Smaran registry, as read here
const NAME_DOMAIN: &[u8] = b"SABHA-CHAINKEY-NAME-V1\0";
const REG_SEED: &[u8] = b"sabha-reg";
const REG_MAGIC: &[u8; 8] = b"SABREG01";
const REG_SIZE: usize = 10_144;
const REG_HEADER: usize = 64;
const REG_ENTRY: usize = 288;
const REG_SLOTS: usize = 35;
const REG_MAX_GEN: u8 = 5;

// ---------------------------------------------------------------- layout
const SEED: &[u8] = b"smaran-streaks";
const MAGIC: &[u8; 8] = b"SMSTRK01";
const NGROUPS: usize = 4;
const HEADER: usize = 64;
const REC: usize = 128;
const RECS: usize = 79;
const GROUP_SIZE: usize = HEADER + RECS * REC; // 10,176 bytes
const _: () = assert!(GROUP_SIZE <= 10_240);
// header
const H_IDX: usize = 8;
const H_VERSION: usize = 9;
const H_USED: usize = 10; // u16 records in use
const H_UNSAVED: usize = 12; // 1 while the group has changes no save has been scheduled for
const H_SAVED: usize = 16; // u32 unix seconds of the last save scheduled
const H_CHECKINS: usize = 20; // u32 check-ins ever made here (wraps)
// record
const R_OWNER: usize = 0; // the Smaran key
const R_NAME: usize = 32; // length + up to 32 bytes: the username, as in the registry
const R_OFFSET: usize = 65; // i16 minutes east of UTC, fixed at the first check-in
const R_FLAGS: usize = 67; // 1 in use
const R_JOINED: usize = 68; // u32 unix seconds of the first check-in
const R_HABITS: usize = 72; // NHABITS habits of HABIT bytes
const NHABITS: usize = 3;
const HABIT: usize = 14;
const _: () = assert!(R_HABITS + NHABITS * HABIT <= REC);
// habit (relative)
const A_KIND: usize = 0; // 0 empty, 1..=NKINDS
const A_STREAK: usize = 2; // u16 days in a row, ending with the last check-in
const A_BEST: usize = 4; // u16 longest streak
const A_TOTAL: usize = 6; // u32 check-ins
const A_LAST: usize = 10; // u32 unix seconds of the last check-in
const NKINDS: u8 = 16;
const NAME_MAX: usize = 32;
const DAY: i64 = 86_400;
const SAVE_EVERY: i64 = 3_600;
const LIVE_COMMIT_FEE: u64 = 100_000; // MagicBlock's charge per save once a delegation's free saves are used
/// Below rent plus this (fewer than ten paid saves left) anyone may hand a group back to Solana, where anyone can
/// refill it and delegate it again. Above it only the operator can.
const LOW_RESERVE: u64 = 10 * LIVE_COMMIT_FEE;
/// The commit setting anyone but the operator must delegate with: no timed commits, the operator's setting.
const OPEN_COMMIT_MS: u32 = u32::MAX;

const MAX_PERMITTED_DATA_INCREASE: usize = 10_240;
const MAX_ACCOUNTS: usize = 10;

// ---------------------------------------------------------------- errors
const ERR_ACCOUNTS: u32 = 1;
const ERR_SIGNER: u32 = 2;
const ERR_OWNER: u32 = 3;
const ERR_DATA: u32 = 4;
const ERR_PDA: u32 = 9;
const ERR_CPI: u32 = 10;
const ERR_TAG: u32 = 11;
const ERR_RANGE: u32 = 12;
const ERR_FULL: u32 = 17;
const ERR_OPERATOR: u32 = 22;
const ERR_NOT_REGISTERED: u32 = 30;
const ERR_TODAY: u32 = 31;
const ERR_HABIT: u32 = 32;
const ERR_NO_RECORD: u32 = 33;
const ERR_RESERVE: u32 = 34;
const ERR_UNCHANGED: u32 = 35;
const ERR_OFFSET: u32 = 36;
const ERR_TOO_SOON: u32 = 37;

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
fn now_unix() -> i64 {
    clock().1
}

// ---------------------------------------------------------------- rules (pure, tested on the host)
/// The day number of `ts` in a time zone `offset` minutes east of UTC.
/// Unsigned arithmetic only: the on-chain target has no signed division. The bias keeps the sum positive for any time
/// after the year -700, so the result equals floor((ts + offset * 60) / DAY).
fn day_of(ts: i64, offset: i16) -> i64 {
    const BIAS_DAYS: i64 = 1_000_000;
    let shifted = (ts + offset as i64 * 60 + BIAS_DAYS * DAY) as u64;
    (shifted / DAY as u64) as i64 - BIAS_DAYS
}

fn valid_offset(offset: i16) -> bool {
    (-720..=840).contains(&offset) && (offset + 720) as u16 % 15 == 0
}

/// A Smaran username as the registry stores it: 3 to 32 of a-z, 0-9, '-' and '_', starting with a letter or digit.
fn valid_name(name: &[u8]) -> bool {
    let alnum = |c: u8| c.is_ascii_lowercase() || c.is_ascii_digit();
    (3..=NAME_MAX).contains(&name.len()) && alnum(name[0]) && name.iter().all(|&c| alnum(c) || c == b'-' || c == b'_')
}

/// One check-in for a habit at `now`: a new day continues the streak if it follows the last check-in's day, else
/// starts it again at 1. A second check-in on the same day (or an earlier one) is refused.
fn check_in(h: &mut [u8], now: i64, offset: i16) -> Result<(), u32> {
    let last = rd32(h, A_LAST) as i64;
    let today = day_of(now, offset);
    let streak: u32 = if last == 0 {
        1
    } else {
        let last_day = day_of(last, offset);
        if today <= last_day {
            return Err(ERR_TODAY);
        }
        if today == last_day + 1 {
            rd16(h, A_STREAK) as u32 + 1
        } else {
            1
        }
    };
    let streak = streak.min(u16::MAX as u32) as u16;
    wr16(h, A_STREAK, streak);
    if streak > rd16(h, A_BEST) {
        wr16(h, A_BEST, streak);
    }
    wr32(h, A_TOTAL, rd32(h, A_TOTAL).saturating_add(1));
    wr32(h, A_LAST, now as u32);
    Ok(())
}

/// (group, record) of `who`'s record, if any.
fn find(gs: &[&mut [u8]; NGROUPS], who: &[u8]) -> Option<(usize, usize)> {
    for (g, t) in gs.iter().enumerate() {
        for r in 0..RECS {
            let o = HEADER + r * REC;
            if t[o + R_FLAGS] & 1 == 1 && eq32(&t[o + R_OWNER..o + R_OWNER + 32], who) {
                return Some((g, r));
            }
        }
    }
    None
}

/// The first free record, in the first group with room.
fn free(gs: &[&mut [u8]; NGROUPS]) -> Option<(usize, usize)> {
    for (g, t) in gs.iter().enumerate() {
        if rd16(t, H_USED) as usize >= RECS {
            continue;
        }
        for r in 0..RECS {
            if t[HEADER + r * REC + R_FLAGS] & 1 == 0 {
                return Some((g, r));
            }
        }
    }
    None
}

/// True when the registry group `reg` holds `name` for `owner` (the same check EternalBliss makes).
fn registered(reg: &Acc, owner: &[u8; 32], name: &[u8]) -> bool {
    if !valid_name(name) || !eq32(reg.owner(), &REGISTRY) || reg.len() != REG_SIZE {
        return false;
    }
    let d = reg.data();
    if &d[..8] != REG_MAGIC || d[15] > REG_MAX_GEN {
        return false;
    }
    let mut hash = [0u8; 32];
    sha256(&[NAME_DOMAIN, name], &mut hash);
    let gen = d[15];
    let index = if gen == 0 { hash[0] % 4 } else { hash[2 + gen as usize] % (4u8 << gen) };
    let (gen_b, idx_b) = ([gen], [index]);
    let expected = if gen == 0 {
        find_pda(&[REG_SEED, &idx_b], &REGISTRY).0
    } else {
        find_pda(&[REG_SEED, &gen_b, &idx_b], &REGISTRY).0
    };
    if d[8] != index || !eq32(reg.key(), &expected) {
        return false;
    }
    let initial = (hash[1] as usize | (hash[2] as usize) << 8) % REG_SLOTS;
    for k in 0..REG_SLOTS {
        let e = REG_HEADER + ((initial + k) % REG_SLOTS) * REG_ENTRY;
        if d[e] == 0 {
            break;
        }
        if eq32(&d[e + 8..e + 40], &hash) && eq32(&d[e + 40..e + 72], owner) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------- context
struct Ctx<'a> {
    accs: &'a [Acc],
    n: usize,
    ix: &'a [u8],
    program_id: &'a [u8; 32],
}

/// accs[i] must be group `idx` of this program: right address, owned here, writable, initialised.
fn group_at(c: &Ctx, i: usize, idx: usize) -> Result<&'static mut [u8], u32> {
    if c.n <= i || idx >= NGROUPS {
        return Err(ERR_ACCOUNTS);
    }
    let a = &c.accs[i];
    let idx_b = [idx as u8];
    let (pda, _) = find_pda(&[SEED, &idx_b], c.program_id);
    if !eq32(&pda, a.key()) {
        return Err(ERR_PDA);
    }
    if !eq32(a.owner(), c.program_id) || !a.is_writable {
        return Err(ERR_OWNER);
    }
    let t = unsafe { core::slice::from_raw_parts_mut(a.data, a.len()) };
    if t.len() != GROUP_SIZE || &t[0..8] != MAGIC || t[H_IDX] != idx as u8 {
        return Err(ERR_DATA);
    }
    Ok(t)
}

/// accs[1..=4] must be groups 0 to 3, in order.
fn groups(c: &Ctx) -> Result<[&'static mut [u8]; NGROUPS], u32> {
    Ok([group_at(c, 1, 0)?, group_at(c, 2, 1)?, group_at(c, 3, 2)?, group_at(c, 4, 3)?])
}

fn reserve_ok(a: &Acc) -> bool {
    a.lamports() >= rent_minimum(GROUP_SIZE) + LIVE_COMMIT_FEE
}

/// Schedules a save of group `idx` (account `gi`) to Solana, with the group as MagicBlock's delegated fee payer: any
/// live fee comes from the group's own reserve. `ci` is the magic context, `vi` the rollup validator's fee vault.
fn save(c: &Ctx, idx: usize, gi: usize, ci: usize, vi: usize) -> Result<(), u32> {
    let (vault, _) = find_pda(&[b"magic-fee-vault", &VALIDATOR], &DELEGATION_PROGRAM);
    if !eq32(c.accs[vi].key(), &vault) {
        return Err(ERR_ACCOUNTS);
    }
    let idx_b = [idx as u8];
    let (_, bump) = find_pda(&[SEED, &idx_b], c.program_id);
    let bump_b = [bump];
    invoke(
        c.accs,
        &Cpi {
            program: &MAGIC_PROGRAM,
            metas: &[(gi, true, true), (ci, true, false), (vi, true, false), (gi, true, false)],
            data: &[1, 0, 0, 0],
            seeds: &[&[SEED, &idx_b, &bump_b]],
        },
    )
}

// ---------------------------------------------------------------- instructions
#[inline(never)]
fn ix_init(c: &Ctx) -> Result<(), u32> {
    // [operator s w, group w, system]; data: tag, idx, lamports u64
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 3 || ix.len() != 10 {
        return Err(ERR_DATA);
    }
    if !accs[0].is_signer || !eq32(accs[0].key(), &OPERATOR) {
        return Err(ERR_OPERATOR);
    }
    let idx = ix[1];
    if idx as usize >= NGROUPS {
        return Err(ERR_RANGE);
    }
    let idx_b = [idx];
    let (pda, bump) = find_pda(&[SEED, &idx_b], program_id);
    if !eq32(&pda, accs[1].key()) || !eq32(accs[2].key(), &SYSTEM_PROGRAM) {
        return Err(ERR_PDA);
    }
    let lamports = u64::from_le_bytes([ix[2], ix[3], ix[4], ix[5], ix[6], ix[7], ix[8], ix[9]]);
    if lamports < rent_minimum(GROUP_SIZE) {
        return Err(ERR_RESERVE);
    }
    let bump_b = [bump];
    let mut data = [0u8; 52];
    data[4..12].copy_from_slice(&lamports.to_le_bytes());
    data[12..20].copy_from_slice(&(GROUP_SIZE as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(0, true, true), (1, true, true)], data: &data, seeds: &[&[SEED, &idx_b, &bump_b]] })?;
    let t = accs[1].data();
    t[0..8].copy_from_slice(MAGIC);
    t[H_IDX] = idx;
    t[H_VERSION] = 1;
    log("streaks: group ready");
    Ok(())
}

#[inline(never)]
fn ix_check_in(c: &Ctx) -> Result<(), u32> {
    // [player s, group 0..3 w, registry group (, magic program, magic context w, fee vault w)]
    // data: tag, slot, kind, offset i16, name_len, name
    let ix = c.ix;
    if c.n < 6 || ix.len() < 6 {
        return Err(ERR_DATA);
    }
    let player = &c.accs[0];
    if !player.is_signer {
        return Err(ERR_SIGNER);
    }
    let (slot, kind) = (ix[1] as usize, ix[2]);
    let offset = i16::from_le_bytes([ix[3], ix[4]]);
    let nl = ix[5] as usize;
    if slot >= NHABITS || kind == 0 || kind > NKINDS || nl > NAME_MAX || ix.len() != 6 + nl {
        return Err(ERR_DATA);
    }
    let gs = groups(c)?;
    let who = player.key();
    let now = now_unix();
    let (g, r) = match find(&gs, who) {
        Some(found) => found,
        None => {
            let name = &ix[6..6 + nl];
            if !valid_offset(offset) {
                return Err(ERR_OFFSET);
            }
            if !registered(&c.accs[5], who, name) {
                return Err(ERR_NOT_REGISTERED);
            }
            let (g, r) = free(&gs).ok_or(ERR_FULL)?;
            let t = &mut *gs[g];
            let o = HEADER + r * REC;
            for b in t[o..o + REC].iter_mut() {
                *b = 0;
            }
            t[o + R_OWNER..o + R_OWNER + 32].copy_from_slice(who);
            t[o + R_NAME] = nl as u8;
            t[o + R_NAME + 1..o + R_NAME + 1 + nl].copy_from_slice(name);
            wr16(t, o + R_OFFSET, offset as u16);
            t[o + R_FLAGS] = 1;
            wr32(t, o + R_JOINED, now as u32);
            let used = rd16(t, H_USED) + 1;
            wr16(t, H_USED, used);
            log("streaks: welcome");
            (g, r)
        }
    };
    let t = &mut *gs[g];
    let o = HEADER + r * REC;
    let off = rd16(t, o + R_OFFSET) as i16;
    let h = o + R_HABITS + slot * HABIT;
    if t[h + A_KIND] == 0 {
        // a new habit in an empty slot, unless another slot already keeps it
        for s in 0..NHABITS {
            if t[o + R_HABITS + s * HABIT + A_KIND] == kind {
                return Err(ERR_HABIT);
            }
        }
        for b in t[h..h + HABIT].iter_mut() {
            *b = 0;
        }
        t[h + A_KIND] = kind;
    } else if t[h + A_KIND] != kind {
        return Err(ERR_HABIT);
    }
    check_in(&mut t[h..h + HABIT], now, off)?;
    wr32(t, H_CHECKINS, rd32(t, H_CHECKINS).wrapping_add(1));
    t[H_UNSAVED] = 1;
    if c.n >= 9 {
        if !eq32(c.accs[6].key(), &MAGIC_PROGRAM) || !eq32(c.accs[7].key(), &MAGIC_CONTEXT) {
            return Err(ERR_ACCOUNTS);
        }
        let saved = rd32(t, H_SAVED) as i64;
        if now - saved >= SAVE_EVERY {
            if reserve_ok(&c.accs[1 + g]) {
                save(c, g, 1 + g, 7, 8)?;
                wr32(t, H_SAVED, now as u32);
                t[H_UNSAVED] = 0;
                log("streaks: save scheduled, paid by the group");
            } else {
                log("streaks: save reserve low, not saved");
            }
        }
    }
    Ok(())
}

#[inline(never)]
fn ix_set_habit(c: &Ctx) -> Result<(), u32> {
    // [player s, group 0..3 w]; data: tag, slot, kind (0 removes). The slot starts again from nothing.
    let ix = c.ix;
    if c.n < 5 || ix.len() != 3 {
        return Err(ERR_DATA);
    }
    if !c.accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    let (slot, kind) = (ix[1] as usize, ix[2]);
    if slot >= NHABITS || kind > NKINDS {
        return Err(ERR_DATA);
    }
    let gs = groups(c)?;
    let (g, r) = find(&gs, c.accs[0].key()).ok_or(ERR_NO_RECORD)?;
    let t = &mut *gs[g];
    let o = HEADER + r * REC;
    if kind != 0 {
        for s in 0..NHABITS {
            if s != slot && t[o + R_HABITS + s * HABIT + A_KIND] == kind {
                return Err(ERR_HABIT);
            }
        }
    }
    let h = o + R_HABITS + slot * HABIT;
    for b in t[h..h + HABIT].iter_mut() {
        *b = 0;
    }
    t[h + A_KIND] = kind;
    t[H_UNSAVED] = 1;
    Ok(())
}

#[inline(never)]
fn ix_forget(c: &Ctx) -> Result<(), u32> {
    // [player s, group 0..3 w]; data: tag
    if c.n < 5 || c.ix.len() != 1 {
        return Err(ERR_DATA);
    }
    if !c.accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    let gs = groups(c)?;
    let (g, r) = find(&gs, c.accs[0].key()).ok_or(ERR_NO_RECORD)?;
    let t = &mut *gs[g];
    let o = HEADER + r * REC;
    for b in t[o..o + REC].iter_mut() {
        *b = 0;
    }
    let used = rd16(t, H_USED).saturating_sub(1);
    wr16(t, H_USED, used);
    t[H_UNSAVED] = 1;
    log("streaks: record removed");
    Ok(())
}

#[inline(never)]
fn ix_save(c: &Ctx) -> Result<(), u32> {
    // [caller s, group w, magic program, magic context w, fee vault w]; data: tag, idx
    if c.n < 5 || c.ix.len() != 2 {
        return Err(ERR_DATA);
    }
    if !c.accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    if !eq32(c.accs[2].key(), &MAGIC_PROGRAM) || !eq32(c.accs[3].key(), &MAGIC_CONTEXT) {
        return Err(ERR_ACCOUNTS);
    }
    let idx = c.ix[1] as usize;
    let t = group_at(c, 1, idx)?;
    let operator = eq32(c.accs[0].key(), &OPERATOR);
    let now = now_unix();
    if !operator {
        if t[H_UNSAVED] == 0 {
            return Err(ERR_UNCHANGED);
        }
        if now - (rd32(t, H_SAVED) as i64) < SAVE_EVERY {
            return Err(ERR_TOO_SOON);
        }
    }
    if !reserve_ok(&c.accs[1]) {
        return Err(ERR_RESERVE);
    }
    save(c, idx, 1, 3, 4)?;
    wr32(t, H_SAVED, now as u32);
    t[H_UNSAVED] = 0;
    log("streaks: save scheduled, paid by the group");
    Ok(())
}

#[inline(never)]
fn ix_delegate(c: &Ctx) -> Result<(), u32> {
    // [payer, group, this program, buffer, record, metadata, delegation program, system]; data: tag, idx, commit_ms u32
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 8 || ix.len() != 6 {
        return Err(ERR_ACCOUNTS);
    }
    let (payer, pda, buffer) = (&accs[0], &accs[1], &accs[3]);
    // Anyone may delegate a group that is back on Solana. The validator is always the one below; anyone but the
    // operator must also use the operator's commit setting.
    if !payer.is_signer || (!eq32(payer.key(), &OPERATOR) && rd32(ix, 2) != OPEN_COMMIT_MS) {
        return Err(ERR_OPERATOR);
    }
    if !eq32(accs[2].key(), program_id) || !eq32(accs[6].key(), &DELEGATION_PROGRAM) || !eq32(accs[7].key(), &SYSTEM_PROGRAM) {
        return Err(ERR_ACCOUNTS);
    }
    if !eq32(pda.owner(), program_id) {
        return Err(ERR_OWNER);
    }
    let idx = ix[1];
    let idx_b = [idx];
    let (key, bump) = find_pda(&[SEED, &idx_b], program_id);
    if !eq32(&key, pda.key()) {
        return Err(ERR_PDA);
    }
    let bump_b = [bump];
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
    invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(1, true, true)], data: &assign, seeds: &[&[SEED, &idx_b, &bump_b]] })?;
    // DelegateArgs: commit frequency ms, seeds Vec<Vec<u8>>, validator Option<Pubkey> (always MagicBlock's devnet rollup)
    let mut args = [0u8; 8 + 4 + 4 + (4 + 16) + (4 + 1) + 1 + 32];
    let mut o = 8;
    args[o..o + 4].copy_from_slice(&ix[2..6]);
    o += 4;
    args[o..o + 4].copy_from_slice(&2u32.to_le_bytes());
    o += 4;
    args[o..o + 4].copy_from_slice(&(SEED.len() as u32).to_le_bytes());
    o += 4;
    args[o..o + SEED.len()].copy_from_slice(SEED);
    o += SEED.len();
    args[o..o + 4].copy_from_slice(&1u32.to_le_bytes());
    o += 4;
    args[o] = idx;
    o += 1;
    args[o] = 1;
    o += 1;
    args[o..o + 32].copy_from_slice(&VALIDATOR);
    o += 32;
    invoke(
        accs,
        &Cpi {
            program: &DELEGATION_PROGRAM,
            metas: &[(0, true, true), (1, true, true), (2, false, false), (3, true, false), (4, true, false), (5, true, false), (7, false, false)],
            data: &args[..o],
            seeds: &[&[SEED, &idx_b, &bump_b]],
        },
    )?;
    unsafe {
        *payer.lamports += *buffer.lamports;
        *buffer.lamports = 0;
        *buffer.data_len = 0;
        *buffer.owner = SYSTEM_PROGRAM;
    }
    log("streaks: delegated");
    Ok(())
}

#[inline(never)]
fn ix_commit_undelegate(c: &Ctx) -> Result<(), u32> {
    // [caller s w, group w, magic program, magic context w]; data: tag, idx
    // The operator may hand a group back to Solana at any time. Anyone may once its reserve is low, so that it can be
    // refilled on Solana and delegated again without the operator.
    let accs = c.accs;
    if c.n < 4 || c.ix.len() != 2 {
        return Err(ERR_DATA);
    }
    if !accs[0].is_signer {
        return Err(ERR_OPERATOR);
    }
    if !eq32(accs[2].key(), &MAGIC_PROGRAM) || !eq32(accs[3].key(), &MAGIC_CONTEXT) {
        return Err(ERR_ACCOUNTS);
    }
    let idx_b = [c.ix[1]];
    let (key, _) = find_pda(&[SEED, &idx_b], c.program_id);
    if !eq32(&key, accs[1].key()) {
        return Err(ERR_PDA);
    }
    let low = accs[1].lamports() < rent_minimum(accs[1].len()) + LOW_RESERVE;
    if !low && !eq32(accs[0].key(), &OPERATOR) {
        return Err(ERR_OPERATOR);
    }
    invoke(accs, &Cpi { program: &MAGIC_PROGRAM, metas: &[(0, accs[0].is_writable, true), (3, true, false), (1, true, false)], data: &[2, 0, 0, 0], seeds: &[] })?;
    log("streaks: commit+undelegate scheduled");
    Ok(())
}

#[inline(never)]
fn ix_undelegate_callback(c: &Ctx) -> Result<(), u32> {
    // [group w, buffer s, payer s w, system]; data: disc, borsh Vec<Vec<u8>> seeds
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
    if a.len() < 4 + 4 + 8 + 4 + 1 || rd32(a, 0) != 2 {
        return Err(ERR_DATA);
    }
    let l0 = rd32(a, 4) as usize;
    if a.len() != 8 + l0 + 4 + 1 || rd32(a, 8 + l0) != 1 || &a[8..8 + l0] != SEED {
        return Err(ERR_DATA);
    }
    let idx_b = [a[a.len() - 1]];
    let (key, bump) = find_pda(&[SEED, &idx_b], program_id);
    if !eq32(&key, pda.key()) {
        return Err(ERR_PDA);
    }
    let bump_b = [bump];
    let space = buffer.len();
    let mut data = [0u8; 52];
    data[4..12].copy_from_slice(&rent_minimum(space).to_le_bytes());
    data[12..20].copy_from_slice(&(space as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(2, true, true), (0, true, true)], data: &data, seeds: &[&[SEED, &idx_b, &bump_b]] })?;
    pda.data().copy_from_slice(buffer.data());
    log("streaks: returned to Solana");
    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    let mut accs: [Acc; MAX_ACCOUNTS] = [Acc::empty(); MAX_ACCOUNTS];
    let (n, ix, program_id) = parse(input, &mut accs);
    if ix.is_empty() {
        return ERR_TAG as u64;
    }
    let c = Ctx { accs: &accs, n, ix, program_id };
    let r = if ix.len() >= 8 && ix[0..8] == EXTERNAL_UNDELEGATE {
        ix_undelegate_callback(&c)
    } else {
        match ix[0] {
            0 => ix_init(&c),
            1 => ix_delegate(&c),
            2 => ix_check_in(&c),
            3 => ix_set_habit(&c),
            4 => ix_save(&c),
            5 => ix_forget(&c),
            8 => ix_commit_undelegate(&c),
            _ => Err(ERR_TAG),
        }
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
