//! Sabha Super Over: a real-time cricket duel on a MagicBlock Private Ephemeral Rollup (TEE).
//!
//! Tables are created once on Solana devnet (the operator pays rent) and delegated to the
//! MagicBlock TEE validator. Players never hold SOL: every action is a zero-fee ER transaction
//! signed by the player's own key from Sabha name+password onboarding.
//!
//! Each table has two PDAs:
//!   state ["so-table", idx]  public match state
//!   ball  ["so-ball", idx]   private: hidden part of the live delivery plus a rolling secret.
//!                            Its ephemeral permission admits only the current bowler.
//!
//! Instructions (first data byte):
//!   0 InitTable   base [payer s w, state w, ball w, system]            idx, lam_state u64, lam_ball u64
//!   1 Delegate    base [payer s w, pda w, this program, buffer w, record w, metadata w, delegation, system]
//!                                                                        idx, which, has_validator, validator 32, commit_ms u32
//!   2 SetupPrivacy ER  [signer s, ball w, permission w, vault w, magic, permission program]   idx, secret 32
//!   3 CreateMatch  ER  [player s, state w, ball w, permission w, vault w, magic, permission program]  idx, mode, name_len, name
//!   4 JoinMatch    ER  same accounts as CreateMatch                    idx, match_id u32, name_len, name
//!   5 Bowl         ER  [bowler s, state w, ball w]                     idx, seq u32, pace, line, length
//!   6 Shot         ER  [batter s, state w, ball w, permission w, vault w, magic, permission program]  idx, seq u32, shot (5 = leave)
//!   7 Advance      ER  same accounts as Shot, any signer               idx
//!   8 AiNextBall   ER  [batter s w, state w, ball w, identity, vrf program, queue w, system, slot hashes]  idx, seq u32
//!   9 Concede      ER  same accounts as Shot                           idx
//!  10 CommitUndelegate ER [payer s w, pda w, magic program, magic context w]   idx, which
//!  11 Arm          ER  [bowler s, ball w]                              idx, seq u32, variation
//!                      Touches only the private account, so the TEE hides it from the batter.
//!  "SOVRFCB1" VRF callback [vrf identity s, state w, ball w]          randomness 32, idx, seq u32
//!  MagicBlock undelegate callback (8-byte discriminator)
#![cfg_attr(target_arch = "bpf", no_std)]
use solana_compiler_builtins as _;

#[cfg(target_arch = "bpf")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { abort() }
}

// ---------------------------------------------------------------- syscalls
#[repr(C)]
struct SolBytes {
    addr: *const u8,
    len: u64,
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
    fn sol_log_64_(a: u64, b: u64, c: u64, d: u64, e: u64);
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
    fn abort() -> !;
}

fn log(msg: &str) {
    unsafe { sol_log_(msg.as_ptr(), msg.len() as u64) }
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
// ACLseoPoyC3cBqoUtkbjZ4aDrkurZW86v19pXz2XQnp1
const PERMISSION_PROGRAM: [u8; 32] = [
    136, 161, 10, 196, 33, 152, 1, 214, 246, 106, 29, 60, 6, 152, 192, 102, 169, 175, 212, 217,
    180, 252, 231, 71, 151, 141, 209, 5, 168, 212, 103, 82,
];
// MagicVau1t999999999999999999999999999999999
const EPHEMERAL_VAULT: [u8; 32] = [
    5, 69, 180, 36, 224, 197, 24, 97, 240, 41, 76, 112, 66, 34, 84, 78, 202, 127, 133, 79, 194,
    135, 136, 166, 123, 118, 113, 80, 62, 224, 143, 184,
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
const EXTERNAL_UNDELEGATE: [u8; 8] = [196, 28, 41, 206, 48, 37, 51, 167];
const VRF_CALLBACK: [u8; 8] = *b"SOVRFCB1";

// ---------------------------------------------------------------- layout
const TABLE_SEED: &[u8] = b"so-table";
const BALL_SEED: &[u8] = b"so-ball";
const IDENTITY_SEED: &[u8] = b"identity";
const NTABLES: u8 = 16;
const STATE_SIZE: usize = 384;
const BALL_SIZE: usize = 96;
const STATE_MAGIC: &[u8; 8] = b"SOVER001";
const BALL_MAGIC: &[u8; 8] = b"SOBALL01";

// state offsets
const S_IDX: usize = 8;
const S_PHASE: usize = 9;
const S_MODE: usize = 10;
const S_INN: usize = 11;
const S_SO: usize = 12; // super over number
const S_TIMEOUTS: usize = 13;
const S_WINNER: usize = 14; // 0 none, 1 A, 2 B, 3 tie
const S_A: usize = 16;
const S_B: usize = 48;
const S_MATCH: usize = 80; // u32
const S_SEQ: usize = 84; // u32, next delivery number
const S_DEADLINE: usize = 88; // u64 slot
const S_RELEASE: usize = 96; // u64 slot
const S_PACE: usize = 104;
const S_LINE: usize = 105;
const S_LENGTH: usize = 106;
const S_ARMED: usize = 107;
const S_RUNS: usize = 112; // u16 x2
const S_WKTS: usize = 116; // u8 x2
const S_BALLS: usize = 118; // u8 x2
const S_TARGET: usize = 120; // u16
const S_BATFIRST: usize = 122; // 0: A bats first in this super over, 1: B
const S_LAST: usize = 124; // shot, grade, result, variation
const S_LAST_DELTA: usize = 128; // i32 slots
const S_LAST_SEQ: usize = 132; // u32
const S_LAST_WKIND: usize = 136;
const S_LAST_BALL: usize = 137; // pace, line, length
const S_LAST_SLOT: usize = 140; // u32 low bits of resolve slot
const S_HISTORY: usize = 144; // 2 x 6 x 8 bytes
const S_FOURS: usize = 240; // u8 x2
const S_SIXES: usize = 242; // u8 x2
const S_CREATED: usize = 248; // u64 slot
const S_NAME_A: usize = 256; // len u8 + up to 23 bytes, as claimed by the player
const S_NAME_B: usize = 280;
const NAME_MAX: usize = 23;

// ball offsets
const B_IDX: usize = 8;
const B_ARMED: usize = 9;
const B_VARIATION: usize = 10;
const B_SEQ: usize = 12;
const B_SECRET: usize = 16;
const B_READER: usize = 48;
// B_ARMED values
const ARM_EMPTY: u8 = 0;
const ARM_SET: u8 = 1;
const ARM_LIVE: u8 = 2;

// phases
const P_FREE: u8 = 0;
const P_WAITING: u8 = 1;
const P_BOWLING: u8 = 2;
const P_LIVE: u8 = 3;
const P_AI_NEED: u8 = 4;
const P_AI_REQUESTED: u8 = 5;
const P_FINISHED: u8 = 6;

const MODE_PVP: u8 = 0;
const MODE_AI: u8 = 1;

// timing in ER slots (10 ms on MagicBlock devnet)
const RUNUP: u64 = 230; // 2.3 s from Bowl to release: room for a full run-up on screen
const AI_RUNUP: u64 = 240;
const FLIGHT: [u64; 3] = [120, 95, 75]; // spin, medium, fast
const SLOWER: u64 = 18;
const W_PERFECT: i64 = 6;
const W_GOOD: i64 = 13;
const W_EDGE: i64 = 24;
const LIVE_GRACE: u64 = 40; // network slack for a late shot after the last moment it could count (was 150)
const BOWL_CLOCK: u64 = 2500;
const WAIT_CLOCK: u64 = 90_000;
const AI_IDLE: u64 = 30_000;
const VRF_TIMEOUT: u64 = 400;
const FINISH_HOLD: u64 = 1500;
const ABANDON: u64 = 6000;
const MAX_SUPER_OVERS: u8 = 4;

// member flags: TX_LOGS | TX_BALANCES | TX_MESSAGE | ACCOUNT_SIGNATURES
const READER_FLAGS: u8 = 2 | 4 | 8 | 16;

const MAX_PERMITTED_DATA_INCREASE: usize = 10_240;
const MAX_ACCOUNTS: usize = 12;

// errors
const ERR_ACCOUNTS: u32 = 1;
const ERR_SIGNER: u32 = 2;
const ERR_OWNER: u32 = 3;
const ERR_DATA: u32 = 4;
const ERR_PHASE: u32 = 5;
const ERR_TURN: u32 = 6;
const ERR_SEQ: u32 = 7;
const ERR_EARLY: u32 = 8;
const ERR_PDA: u32 = 9;
const ERR_CPI: u32 = 10;
const ERR_TAG: u32 = 11;
const ERR_RANGE: u32 = 12;
const ERR_IDENTITY: u32 = 13;
const ERR_BUSY: u32 = 14;
const ERR_NOT_ARMED: u32 = 15;

// ---------------------------------------------------------------- input parsing
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
    fn data(&self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.data, self.len()) }
    }
    fn info(&self) -> SolAccountInfo {
        SolAccountInfo {
            key: self.key,
            lamports: self.lamports,
            data_len: self.len() as u64,
            data: self.data,
            owner: self.owner,
            rent_epoch: self.rent_epoch,
            is_signer: self.is_signer,
            is_writable: self.is_writable,
            executable: self.executable,
        }
    }
}

unsafe fn read_u64(p: *const u8) -> u64 {
    core::ptr::read_unaligned(p as *const u64)
}

unsafe fn parse(
    input: *mut u8,
    accs: &mut [Acc; MAX_ACCOUNTS],
) -> (usize, &'static [u8], &'static [u8; 32]) {
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
            accs[i] = Acc {
                key,
                owner,
                lamports,
                data_len: data_len_ptr,
                data,
                rent_epoch,
                is_signer,
                is_writable,
                executable,
            };
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

// ---------------------------------------------------------------- helpers
const NULL_BYTES: SolBytes = SolBytes { addr: core::ptr::null(), len: 0 };

fn find_pda(seeds: &[&[u8]], program_id: &[u8; 32]) -> ([u8; 32], u8) {
    let mut sb: [SolBytes; 4] = [NULL_BYTES, NULL_BYTES, NULL_BYTES, NULL_BYTES];
    for (i, s) in seeds.iter().enumerate() {
        sb[i] = SolBytes { addr: s.as_ptr(), len: s.len() as u64 };
    }
    let mut out = [0u8; 32];
    let mut bump = 0u8;
    unsafe {
        sol_try_find_program_address(
            sb.as_ptr(),
            seeds.len() as u64,
            program_id.as_ptr(),
            out.as_mut_ptr(),
            &mut bump,
        );
    }
    (out, bump)
}

fn sha256(parts: &[&[u8]], out: &mut [u8; 32]) {
    let mut sb: [SolBytes; 4] = [NULL_BYTES, NULL_BYTES, NULL_BYTES, NULL_BYTES];
    for (i, p) in parts.iter().enumerate() {
        sb[i] = SolBytes { addr: p.as_ptr(), len: p.len() as u64 };
    }
    unsafe {
        sol_sha256(sb.as_ptr(), parts.len() as u64, out.as_mut_ptr());
    }
}

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

fn now_slot() -> u64 {
    let mut buf = [0u8; 40];
    unsafe {
        sol_get_clock_sysvar(buf.as_mut_ptr());
    }
    u64::from_le_bytes([buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]])
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

/// A cross-program call, passed by reference so no call needs more than five registers.
/// `metas` lists (account index into `accs`, writable, signer).
struct Cpi<'a> {
    program: &'a [u8; 32],
    metas: &'a [(usize, bool, bool)],
    data: &'a [u8],
    seeds: &'a [&'a [&'a [u8]]],
}

#[inline(never)]
fn invoke(accs: &[Acc], cpi: &Cpi) -> Result<(), u32> {
    const NULL_META: SolAccountMeta = SolAccountMeta {
        pubkey: core::ptr::null(),
        is_writable: false,
        is_signer: false,
    };
    let mut m: [SolAccountMeta; 8] = [NULL_META; 8];
    let mut infos: [SolAccountInfo; 8] = [accs[0].info(); 8];
    for (i, (idx, w, s)) in cpi.metas.iter().enumerate() {
        m[i] = SolAccountMeta { pubkey: accs[*idx].key, is_writable: *w, is_signer: *s };
        infos[i] = accs[*idx].info();
    }
    let mut seed_bytes: [[SolBytes; 4]; 2] = [
        [NULL_BYTES, NULL_BYTES, NULL_BYTES, NULL_BYTES],
        [NULL_BYTES, NULL_BYTES, NULL_BYTES, NULL_BYTES],
    ];
    let mut signers: [SolSignerSeeds; 2] = [
        SolSignerSeeds { addr: core::ptr::null(), len: 0 },
        SolSignerSeeds { addr: core::ptr::null(), len: 0 },
    ];
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
        sol_invoke_signed_c(
            &ix,
            infos.as_ptr(),
            cpi.metas.len() as u64,
            signers.as_ptr(),
            cpi.seeds.len() as u64,
        )
    };
    if rc != 0 {
        return Err(ERR_CPI);
    }
    Ok(())
}

fn rent_minimum(len: usize) -> u64 {
    let mut buf = [0u8; 24];
    unsafe {
        sol_get_rent_sysvar(buf.as_mut_ptr());
    }
    let per_byte = rd64(&buf, 0);
    let thr_bits = rd64(&buf, 8);
    let base = (128 + len as u64) * per_byte;
    if thr_bits == 0x3FF0_0000_0000_0000 {
        base
    } else {
        base * 2
    }
}

// ---------------------------------------------------------------- context
struct Ctx<'a> {
    accs: &'a [Acc],
    n: usize,
    ix: &'a [u8],
    program_id: &'a [u8; 32],
}

/// Checks that accs[1] is the table state PDA and accs[2] the ball PDA for `idx`, both owned
/// by this program and initialised. Returns (state bump, ball bump).
fn check_table(c: &Ctx, idx: u8) -> Result<(u8, u8), u32> {
    if c.n < 3 || idx >= NTABLES {
        return Err(ERR_ACCOUNTS);
    }
    let (state, ball) = (&c.accs[1], &c.accs[2]);
    let idx_b = [idx];
    let (sp, sb) = find_pda(&[TABLE_SEED, &idx_b], c.program_id);
    let (bp, bb) = find_pda(&[BALL_SEED, &idx_b], c.program_id);
    if !eq32(&sp, state.key()) || !eq32(&bp, ball.key()) {
        return Err(ERR_PDA);
    }
    if !eq32(state.owner(), c.program_id) || !eq32(ball.owner(), c.program_id) {
        return Err(ERR_OWNER);
    }
    if !state.is_writable || !ball.is_writable {
        return Err(ERR_ACCOUNTS);
    }
    let (s, b) = (state.data(), ball.data());
    if s.len() != STATE_SIZE || b.len() != BALL_SIZE || &s[0..8] != STATE_MAGIC || &b[0..8] != BALL_MAGIC {
        return Err(ERR_DATA);
    }
    Ok((sb, bb))
}

/// Accounts for the ephemeral-permission CPI, as indices into the instruction's accounts.
struct Privacy {
    ball: usize,
    permission: usize,
    vault: usize,
    magic: usize,
    idx: u8,
    bump: u8,
}

fn privacy_accounts_ok(c: &Ctx, p: &Privacy) -> Result<(), u32> {
    if c.n <= p.magic + 1 {
        return Err(ERR_ACCOUNTS);
    }
    let (perm, _) = find_pda(&[b"permission:", c.accs[p.ball].key()], &PERMISSION_PROGRAM);
    if !eq32(&perm, c.accs[p.permission].key())
        || !eq32(c.accs[p.vault].key(), &EPHEMERAL_VAULT)
        || !eq32(c.accs[p.magic].key(), &MAGIC_PROGRAM)
        || !eq32(c.accs[p.magic + 1].key(), &PERMISSION_PROGRAM)
    {
        return Err(ERR_ACCOUNTS);
    }
    Ok(())
}

/// Sets who may read the private ball account: nobody, or exactly `reader`.
#[inline(never)]
fn set_reader(c: &Ctx, p: &Privacy, reader: Option<&[u8; 32]>) -> Result<(), u32> {
    privacy_accounts_ok(c, p)?;
    let mut data = [0u8; 8 + 1 + 33];
    data[0] = 7; // UpdateEphemeralPermission
    data[8] = 1; // is_private
    let mut len = 9;
    if let Some(r) = reader {
        data[9] = READER_FLAGS;
        data[10..42].copy_from_slice(r);
        len = 42;
    }
    let idx_b = [p.idx];
    let bump_b = [p.bump];
    invoke(
        c.accs,
        &Cpi {
            program: &PERMISSION_PROGRAM,
            metas: &[
                (p.ball, true, true),
                (p.ball, false, false),
                (p.ball, false, true),
                (p.permission, true, false),
                (p.vault, true, false),
                (p.magic, false, false),
            ],
            data: &data[..len],
            seeds: &[&[BALL_SEED, &idx_b, &bump_b]],
        },
    )?;
    let b = c.accs[p.ball].data();
    match reader {
        Some(r) => b[B_READER..B_READER + 32].copy_from_slice(r),
        None => {
            for x in b[B_READER..B_READER + 32].iter_mut() {
                *x = 0;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- cricket rules
// pace: 0 spin, 1 medium, 2 fast
// line: 0 outside off, 1 off stump, 2 middle, 3 leg stump
// length: 0 yorker, 1 full, 2 good, 3 short
// variation (hidden): 0 none, 1 moves in, 2 moves away, 3 slower ball
// shot: 0 defend, 1 drive, 2 cut, 3 pull, 4 loft, 255 no shot
// grade: 0 perfect, 1 good, 2 early, 3 late, 4 missed, 5 left alone
// result: 0..6 runs, 7 wicket
// wicket kind: 1 bowled, 2 lbw, 3 caught, 4 caught behind

// FIT[shot][length][line]
const FIT: [[[i8; 4]; 4]; 5] = [
    [[2, 2, 2, 2], [2, 2, 2, 2], [2, 2, 2, 2], [2, 2, 2, 2]],
    [[1, 1, 1, 1], [4, 4, 3, 2], [2, 2, 2, 1], [0, 0, 0, 0]],
    [[0, 0, 0, 0], [1, 0, 0, 0], [3, 2, 0, 0], [4, 3, 1, 0]],
    [[0, 0, 0, 0], [0, 0, 0, 0], [0, 1, 2, 3], [2, 3, 4, 4]],
    [[0, 0, 0, 0], [4, 4, 4, 4], [2, 2, 2, 2], [1, 1, 1, 1]],
];

struct Ball {
    pace: u8,
    line: u8,
    length: u8,
    variation: u8,
    shot: u8,
    grade: u8,
}

/// Where the ball ends up after its hidden movement: 0 outside off .. 3 leg, 4 down leg.
fn effective_line(line: u8, variation: u8) -> u8 {
    match variation {
        1 => line + 1,
        2 => {
            if line == 0 {
                0
            } else {
                line - 1
            }
        }
        _ => line,
    }
}

fn hits_stumps(b: &Ball) -> bool {
    let el = effective_line(b.line, b.variation);
    b.length != 3 && el >= 1 && el <= 3
}

/// Returns result | (wicket_kind << 8).
fn outcome(b: &Ball) -> u16 {
    if b.shot > 4 || b.grade >= 4 {
        // missed or left alone
        if hits_stumps(b) {
            let kind: u16 = if b.length == 0 || b.grade == 5 { 1 } else { 2 };
            return 7 | (kind << 8);
        }
        return 0;
    }
    let s = b.shot as usize;
    let mut fit = FIT[s][b.length as usize][b.line as usize] as i16;
    if s == 4 && b.pace == 0 && b.length <= 2 {
        fit += 1; // stepping out to spin
    }
    match b.variation {
        1 if s == 1 || s == 2 => fit -= 1,
        2 if s == 1 || s == 4 => fit -= 1,
        3 if s == 4 => fit -= 1,
        _ => {}
    }
    if fit < 0 {
        fit = 0;
    }
    if fit > 4 {
        fit = 4;
    }
    let q = fit - match b.grade {
        0 => 0,
        1 => 1,
        _ => 2,
    };
    let mistimed = b.grade >= 2;
    match s {
        0 => {
            if b.grade == 0 {
                1
            } else if mistimed && b.variation == 2 && b.line <= 1 {
                7 | (4 << 8)
            } else {
                0
            }
        }
        4 => match q {
            4 => 6,
            3 => 4,
            2 => 1,
            _ => 7 | (3 << 8),
        },
        _ => match q {
            4 => 4,
            3 => 2,
            2 => 1,
            1 => 0,
            _ => {
                if b.variation == 2 && b.line <= 1 && (s == 1 || s == 2) {
                    7 | (4 << 8)
                } else if s == 3 && b.length <= 1 && hits_stumps(b) {
                    7 | (2 << 8)
                } else if s == 2 && b.line >= 2 && hits_stumps(b) {
                    7 | (1 << 8)
                } else {
                    0
                }
            }
        },
    }
}

fn grade_for(delta: i64) -> u8 {
    let a = if delta < 0 { -delta } else { delta };
    if a <= W_PERFECT {
        0
    } else if a <= W_GOOD {
        1
    } else if a <= W_EDGE {
        if delta < 0 {
            2
        } else {
            3
        }
    } else {
        4
    }
}

// ---------------------------------------------------------------- match flow helpers
fn batting_side(s: &[u8]) -> u8 {
    // 0 = A, 1 = B
    s[S_BATFIRST] ^ s[S_INN]
}

fn side_key(s: &[u8], side: u8) -> [u8; 32] {
    let o = if side == 0 { S_A } else { S_B };
    let mut k = [0u8; 32];
    k.copy_from_slice(&s[o..o + 32]);
    k
}

fn reset_innings(s: &mut [u8]) {
    for o in S_RUNS..S_TARGET {
        s[o] = 0;
    }
    for o in S_HISTORY..S_FOURS + 4 {
        s[o] = 0;
    }
    s[S_INN] = 0;
}

/// After a ball or penalty: move the match on. Updates who may read the private ball.
#[inline(never)]
fn advance_match(c: &Ctx, p: &Privacy, slot: u64) -> Result<(), u32> {
    let s = c.accs[1].data();
    let inn = s[S_INN] as usize;
    let runs = rd16(s, S_RUNS + 2 * inn);
    let wkts = s[S_WKTS + inn];
    let balls = s[S_BALLS + inn];
    let target = rd16(s, S_TARGET);
    let chasing = s[S_MODE] == MODE_AI || inn == 1;
    let innings_over = balls >= 6 || wkts >= 2 || (chasing && target > 0 && runs >= target);
    if !innings_over {
        if s[S_MODE] == MODE_AI {
            s[S_PHASE] = P_AI_NEED;
            wr64(s, S_DEADLINE, slot + AI_IDLE);
        } else {
            s[S_PHASE] = P_BOWLING;
            wr64(s, S_DEADLINE, slot + BOWL_CLOCK);
        }
        return Ok(());
    }
    if s[S_MODE] == MODE_AI {
        s[S_WINNER] = if runs >= target {
            1
        } else if runs + 1 == target {
            3
        } else {
            2
        };
        s[S_PHASE] = P_FINISHED;
        wr64(s, S_DEADLINE, slot + FINISH_HOLD);
        log("so: chase over");
        return Ok(());
    }
    if inn == 0 {
        s[S_INN] = 1;
        wr16(s, S_TARGET, runs + 1);
        s[S_PHASE] = P_BOWLING;
        s[S_TIMEOUTS] = 0;
        wr64(s, S_DEADLINE, slot + BOWL_CLOCK);
        let bowler = side_key(s, batting_side(s) ^ 1);
        log("so: innings break");
        return set_reader(c, p, Some(&bowler));
    }
    let first = rd16(s, S_RUNS);
    let bat_second = batting_side(s);
    if runs >= target {
        s[S_WINNER] = bat_second + 1;
    } else if runs == first {
        if s[S_SO] + 1 < MAX_SUPER_OVERS {
            // tied: another Super Over, the side that batted second now bats first
            s[S_SO] += 1;
            s[S_BATFIRST] = bat_second;
            reset_innings(s);
            s[S_PHASE] = P_BOWLING;
            s[S_TIMEOUTS] = 0;
            wr64(s, S_DEADLINE, slot + BOWL_CLOCK);
            let bowler = side_key(s, batting_side(s) ^ 1);
            log("so: tied, another super over");
            return set_reader(c, p, Some(&bowler));
        }
        s[S_WINNER] = 3;
    } else {
        s[S_WINNER] = (bat_second ^ 1) + 1;
    }
    s[S_PHASE] = P_FINISHED;
    wr64(s, S_DEADLINE, slot + FINISH_HOLD);
    log("so: match over");
    set_reader(c, p, None)
}

/// Scores the live ball. `shot` is 255 when the batter never played.
#[inline(never)]
fn resolve(c: &Ctx, p: &Privacy, shot: u8, slot: u64) -> Result<(), u32> {
    let s = c.accs[1].data();
    let b = c.accs[2].data();
    let pace = s[S_PACE];
    let variation = b[B_VARIATION];
    let ideal = rd64(s, S_RELEASE) + FLIGHT[pace as usize] + if variation == 3 { SLOWER } else { 0 };
    let delta = slot as i64 - ideal as i64;
    let grade = if shot == 255 { 5 } else { grade_for(delta) };
    let ball = Ball { pace, line: s[S_LINE], length: s[S_LENGTH], variation, shot, grade };
    let r = outcome(&ball);
    let result = (r & 0xff) as u8;
    let wkind = (r >> 8) as u8;
    let inn = s[S_INN] as usize;
    let n = s[S_BALLS + inn] as usize;
    if n < 6 {
        let h = S_HISTORY + inn * 48 + n * 8;
        s[h] = pace;
        s[h + 1] = ball.line;
        s[h + 2] = ball.length;
        s[h + 3] = variation;
        s[h + 4] = shot;
        s[h + 5] = grade;
        s[h + 6] = result;
        s[h + 7] = wkind;
    }
    s[S_BALLS + inn] += 1;
    if result == 7 {
        s[S_WKTS + inn] += 1;
    } else {
        let runs = rd16(s, S_RUNS + 2 * inn) + result as u16;
        wr16(s, S_RUNS + 2 * inn, runs);
        if result == 4 {
            s[S_FOURS + inn] += 1;
        }
        if result == 6 {
            s[S_SIXES + inn] += 1;
        }
    }
    s[S_LAST] = shot;
    s[S_LAST + 1] = grade;
    s[S_LAST + 2] = result;
    s[S_LAST + 3] = variation;
    let d32 = if delta > 100_000 {
        100_000
    } else if delta < -100_000 {
        -100_000
    } else {
        delta
    } as i32;
    s[S_LAST_DELTA..S_LAST_DELTA + 4].copy_from_slice(&d32.to_le_bytes());
    let seq = rd32(s, S_SEQ);
    wr32(s, S_LAST_SEQ, seq);
    wr32(s, S_SEQ, seq + 1);
    s[S_LAST_WKIND] = wkind;
    s[S_LAST_BALL] = pace;
    s[S_LAST_BALL + 1] = ball.line;
    s[S_LAST_BALL + 2] = ball.length;
    wr32(s, S_LAST_SLOT, slot as u32);
    s[S_ARMED] = 0;
    b[B_ARMED] = ARM_EMPTY;
    b[B_VARIATION] = 0;
    wr32(b, B_SEQ, seq + 1);
    log("so: ball");
    unsafe { sol_log_64_(seq as u64, shot as u64, grade as u64, result as u64, wkind as u64) };
    advance_match(c, p, slot)
}

fn privacy_for(c: &Ctx, idx: u8, ball_bump: u8) -> Privacy {
    let _ = c;
    Privacy { ball: 2, permission: 3, vault: 4, magic: 5, idx, bump: ball_bump }
}

// ---------------------------------------------------------------- instructions
#[inline(never)]
fn ix_init_table(c: &Ctx) -> Result<(), u32> {
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 4 || ix.len() != 18 {
        return Err(ERR_DATA);
    }
    if !accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    let idx = ix[1];
    if idx >= NTABLES {
        return Err(ERR_RANGE);
    }
    let idx_b = [idx];
    let (sp, sbump) = find_pda(&[TABLE_SEED, &idx_b], program_id);
    let (bp, bbump) = find_pda(&[BALL_SEED, &idx_b], program_id);
    if !eq32(&sp, accs[1].key()) || !eq32(&bp, accs[2].key()) {
        return Err(ERR_PDA);
    }
    let sb = [sbump];
    let bb = [bbump];
    let mut data = [0u8; 52];
    data[4..12].copy_from_slice(&ix[2..10]);
    data[12..20].copy_from_slice(&(STATE_SIZE as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(
        accs,
        &Cpi {
            program: &SYSTEM_PROGRAM,
            metas: &[(0, true, true), (1, true, true)],
            data: &data,
            seeds: &[&[TABLE_SEED, &idx_b, &sb]],
        },
    )?;
    data[4..12].copy_from_slice(&ix[10..18]);
    data[12..20].copy_from_slice(&(BALL_SIZE as u64).to_le_bytes());
    invoke(
        accs,
        &Cpi {
            program: &SYSTEM_PROGRAM,
            metas: &[(0, true, true), (2, true, true)],
            data: &data,
            seeds: &[&[BALL_SEED, &idx_b, &bb]],
        },
    )?;
    let s = accs[1].data();
    s[0..8].copy_from_slice(STATE_MAGIC);
    s[S_IDX] = idx;
    let b = accs[2].data();
    b[0..8].copy_from_slice(BALL_MAGIC);
    b[B_IDX] = idx;
    log("so: table ready");
    Ok(())
}

#[inline(never)]
fn ix_delegate(c: &Ctx) -> Result<(), u32> {
    // [payer, pda, this program, buffer, record, metadata, delegation program, system]
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 8 || ix.len() != 40 {
        return Err(ERR_ACCOUNTS);
    }
    let (payer, pda, buffer) = (&accs[0], &accs[1], &accs[3]);
    if !payer.is_signer {
        return Err(ERR_SIGNER);
    }
    if !eq32(accs[2].key(), program_id) || !eq32(accs[6].key(), &DELEGATION_PROGRAM) {
        return Err(ERR_ACCOUNTS);
    }
    if !eq32(pda.owner(), program_id) {
        return Err(ERR_OWNER);
    }
    let idx = ix[1];
    let seed: &[u8] = if ix[2] == 0 { TABLE_SEED } else { BALL_SEED };
    let idx_b = [idx];
    let (key, bump) = find_pda(&[seed, &idx_b], program_id);
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
    invoke(
        accs,
        &Cpi {
            program: &SYSTEM_PROGRAM,
            metas: &[(0, true, true), (3, true, true)],
            data: &data,
            seeds: &[&[b"buffer", pda.key(), &buf_bump_b]],
        },
    )?;
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
    invoke(
        accs,
        &Cpi {
            program: &SYSTEM_PROGRAM,
            metas: &[(1, true, true)],
            data: &assign,
            seeds: &[&[seed, &idx_b, &bump_b]],
        },
    )?;
    let mut args = [0u8; 8 + 4 + 4 + (4 + 8) + (4 + 1) + 1 + 32];
    let mut o = 8;
    args[o..o + 4].copy_from_slice(&ix[36..40]);
    o += 4;
    args[o..o + 4].copy_from_slice(&2u32.to_le_bytes());
    o += 4;
    args[o..o + 4].copy_from_slice(&(seed.len() as u32).to_le_bytes());
    o += 4;
    args[o..o + seed.len()].copy_from_slice(seed);
    o += seed.len();
    args[o..o + 4].copy_from_slice(&1u32.to_le_bytes());
    o += 4;
    args[o] = idx;
    o += 1;
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
            metas: &[
                (0, true, true),
                (1, true, true),
                (2, false, false),
                (3, true, false),
                (4, true, false),
                (5, true, false),
                (7, false, false),
            ],
            data: &args[..o],
            seeds: &[&[seed, &idx_b, &bump_b]],
        },
    )?;
    unsafe {
        *payer.lamports += *buffer.lamports;
        *buffer.lamports = 0;
        *buffer.data_len = 0;
        *buffer.owner = SYSTEM_PROGRAM;
    }
    log("so: delegated");
    Ok(())
}

#[inline(never)]
fn ix_setup_privacy(c: &Ctx) -> Result<(), u32> {
    // [signer, ball, permission, vault, magic, permission program]
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 6 || ix.len() != 34 {
        return Err(ERR_DATA);
    }
    if !accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    let idx = ix[1];
    let idx_b = [idx];
    let (bp, bump) = find_pda(&[BALL_SEED, &idx_b], program_id);
    if !eq32(&bp, accs[1].key()) || !eq32(accs[1].owner(), program_id) {
        return Err(ERR_PDA);
    }
    let p = Privacy { ball: 1, permission: 2, vault: 3, magic: 4, idx, bump };
    privacy_accounts_ok(c, &p)?;
    if accs[2].len() == 0 {
        let mut data = [0u8; 9];
        data[0] = 6; // CreateEphemeralPermission
        data[8] = 1; // private, no members
        let bump_b = [bump];
        invoke(
            accs,
            &Cpi {
                program: &PERMISSION_PROGRAM,
                metas: &[(1, true, true), (1, false, true), (2, true, false), (3, true, false), (4, false, false)],
                data: &data,
                seeds: &[&[BALL_SEED, &idx_b, &bump_b]],
            },
        )?;
    }
    let b = accs[1].data();
    let mut fresh = [0u8; 32];
    let slot = now_slot().to_le_bytes();
    sha256(&[&b[B_SECRET..B_SECRET + 32], &ix[2..34], &slot], &mut fresh);
    b[B_SECRET..B_SECRET + 32].copy_from_slice(&fresh);
    log("so: privacy ready");
    Ok(())
}

/// Copies an optional claimed display name (clients verify it against the Sabha registry).
fn store_name(s: &mut [u8], at: usize, ix: &[u8], from: usize) -> Result<(), u32> {
    for x in s[at..at + 1 + NAME_MAX].iter_mut() {
        *x = 0;
    }
    if ix.len() <= from {
        return Ok(());
    }
    let len = ix[from] as usize;
    if len > NAME_MAX || ix.len() != from + 1 + len {
        return Err(ERR_DATA);
    }
    s[at] = len as u8;
    s[at + 1..at + 1 + len].copy_from_slice(&ix[from + 1..from + 1 + len]);
    Ok(())
}

fn reclaimable(s: &[u8], slot: u64) -> bool {
    let phase = s[S_PHASE];
    let deadline = rd64(s, S_DEADLINE);
    phase == P_FREE || (phase == P_FINISHED && slot > deadline) || slot > deadline + ABANDON
}

#[inline(never)]
fn ix_create_match(c: &Ctx) -> Result<(), u32> {
    let ix = c.ix;
    if ix.len() < 3 || c.n < 7 {
        return Err(ERR_DATA);
    }
    let player = &c.accs[0];
    if !player.is_signer {
        return Err(ERR_SIGNER);
    }
    let idx = ix[1];
    let mode = ix[2];
    if mode > MODE_AI {
        return Err(ERR_RANGE);
    }
    let (_, bb) = check_table(c, idx)?;
    let slot = now_slot();
    let s = c.accs[1].data();
    if !reclaimable(s, slot) {
        return Err(ERR_BUSY);
    }
    let match_id = rd32(s, S_MATCH) + 1;
    for o in S_PHASE..STATE_SIZE {
        if o < S_SEQ || o >= S_SEQ + 4 {
            s[o] = 0;
        }
    }
    wr32(s, S_MATCH, match_id);
    s[S_MODE] = mode;
    s[S_A..S_A + 32].copy_from_slice(player.key());
    store_name(s, S_NAME_A, ix, 3)?;
    wr64(s, S_CREATED, slot);
    let b = c.accs[2].data();
    b[B_ARMED] = ARM_EMPTY;
    b[B_VARIATION] = 0;
    wr32(b, B_SEQ, rd32(s, S_SEQ));
    if mode == MODE_PVP {
        s[S_PHASE] = P_WAITING;
        wr64(s, S_DEADLINE, slot + WAIT_CLOCK);
    } else {
        s[S_PHASE] = P_AI_NEED;
        wr64(s, S_DEADLINE, slot + AI_IDLE);
    }
    log("so: match created");
    unsafe { sol_log_64_(idx as u64, match_id as u64, mode as u64, 0, 0) };
    let p = privacy_for(c, idx, bb);
    set_reader(c, &p, None)
}

#[inline(never)]
fn ix_join_match(c: &Ctx) -> Result<(), u32> {
    let ix = c.ix;
    if ix.len() < 6 || c.n < 7 {
        return Err(ERR_DATA);
    }
    let player = &c.accs[0];
    if !player.is_signer {
        return Err(ERR_SIGNER);
    }
    let idx = ix[1];
    let (_, bb) = check_table(c, idx)?;
    let slot = now_slot();
    let s = c.accs[1].data();
    if s[S_PHASE] != P_WAITING || s[S_MODE] != MODE_PVP {
        return Err(ERR_PHASE);
    }
    if rd32(s, S_MATCH) != rd32(ix, 2) {
        return Err(ERR_SEQ);
    }
    if eq32(&s[S_A..S_A + 32], player.key()) {
        return Err(ERR_TURN);
    }
    s[S_B..S_B + 32].copy_from_slice(player.key());
    store_name(s, S_NAME_B, ix, 6)?;
    s[S_BATFIRST] = 0;
    s[S_INN] = 0;
    s[S_PHASE] = P_BOWLING;
    wr64(s, S_DEADLINE, slot + BOWL_CLOCK);
    let bowler = side_key(s, 1);
    log("so: opponent joined");
    let p = privacy_for(c, idx, bb);
    set_reader(c, &p, Some(&bowler))
}

#[inline(never)]
fn ix_bowl(c: &Ctx) -> Result<(), u32> {
    let ix = c.ix;
    if ix.len() != 9 {
        return Err(ERR_DATA);
    }
    let bowler = &c.accs[0];
    if !bowler.is_signer {
        return Err(ERR_SIGNER);
    }
    let idx = ix[1];
    check_table(c, idx)?;
    let (pace, line, length) = (ix[6], ix[7], ix[8]);
    if pace > 2 || line > 3 || length > 3 {
        return Err(ERR_RANGE);
    }
    let slot = now_slot();
    let s = c.accs[1].data();
    if s[S_PHASE] != P_BOWLING || s[S_MODE] != MODE_PVP {
        return Err(ERR_PHASE);
    }
    let bowler_key = side_key(s, batting_side(s) ^ 1);
    if !eq32(&bowler_key, bowler.key()) {
        return Err(ERR_TURN);
    }
    let seq = rd32(s, S_SEQ);
    if seq != rd32(ix, 2) {
        return Err(ERR_SEQ);
    }
    // the hidden variation was armed earlier in a transaction the batter cannot see
    let b = c.accs[2].data();
    if b[B_ARMED] != ARM_SET || rd32(b, B_SEQ) != seq {
        return Err(ERR_NOT_ARMED);
    }
    b[B_ARMED] = ARM_LIVE;
    let release = slot + RUNUP;
    s[S_PACE] = pace;
    s[S_LINE] = line;
    s[S_LENGTH] = length;
    s[S_ARMED] = 1;
    wr64(s, S_RELEASE, release);
    // deadline never depends on the hidden variation
    wr64(s, S_DEADLINE, release + FLIGHT[pace as usize] + SLOWER + W_EDGE as u64 + LIVE_GRACE);
    s[S_PHASE] = P_LIVE;
    s[S_TIMEOUTS] = 0;
    log("so: bowled");
    Ok(())
}

#[inline(never)]
fn ix_shot(c: &Ctx) -> Result<(), u32> {
    let ix = c.ix;
    if ix.len() != 7 || c.n < 7 {
        return Err(ERR_DATA);
    }
    let batter = &c.accs[0];
    if !batter.is_signer {
        return Err(ERR_SIGNER);
    }
    let idx = ix[1];
    let (_, bb) = check_table(c, idx)?;
    let shot = match ix[6] {
        5 => 255, // leave the ball
        x if x <= 4 => x,
        _ => return Err(ERR_RANGE),
    };
    let slot = now_slot();
    let s = c.accs[1].data();
    if s[S_PHASE] != P_LIVE {
        return Err(ERR_PHASE);
    }
    let batter_key = side_key(s, batting_side(s));
    if !eq32(&batter_key, batter.key()) {
        return Err(ERR_TURN);
    }
    if rd32(s, S_SEQ) != rd32(ix, 2) {
        return Err(ERR_SEQ);
    }
    if slot < rd64(s, S_RELEASE) {
        return Err(ERR_EARLY);
    }
    let p = privacy_for(c, idx, bb);
    privacy_accounts_ok(c, &p)?;
    resolve(c, &p, shot, slot)
}

#[inline(never)]
fn ix_advance(c: &Ctx) -> Result<(), u32> {
    let ix = c.ix;
    if ix.len() != 2 || c.n < 7 {
        return Err(ERR_DATA);
    }
    if !c.accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    let idx = ix[1];
    let (_, bb) = check_table(c, idx)?;
    let p = privacy_for(c, idx, bb);
    privacy_accounts_ok(c, &p)?;
    let slot = now_slot();
    let s = c.accs[1].data();
    let deadline = rd64(s, S_DEADLINE);
    if slot <= deadline {
        return Err(ERR_PHASE);
    }
    match s[S_PHASE] {
        P_LIVE => resolve(c, &p, 255, slot),
        P_BOWLING => {
            // bowler ran the clock down: one penalty run, and three in a row forfeits
            let inn = s[S_INN] as usize;
            let runs = rd16(s, S_RUNS + 2 * inn) + 1;
            wr16(s, S_RUNS + 2 * inn, runs);
            s[S_TIMEOUTS] += 1;
            if s[S_TIMEOUTS] >= 3 {
                s[S_WINNER] = batting_side(s) + 1;
                s[S_PHASE] = P_FINISHED;
                wr64(s, S_DEADLINE, slot + FINISH_HOLD);
                log("so: bowler forfeited");
                return set_reader(c, &p, None);
            }
            wr64(s, S_DEADLINE, slot + BOWL_CLOCK);
            log("so: bowler penalty");
            advance_match(c, &p, slot)
        }
        P_WAITING => {
            s[S_PHASE] = P_FREE;
            log("so: table freed");
            Ok(())
        }
        P_AI_REQUESTED => {
            s[S_PHASE] = P_AI_NEED;
            wr64(s, S_DEADLINE, slot + AI_IDLE);
            Ok(())
        }
        P_AI_NEED => {
            s[S_WINNER] = 2;
            s[S_PHASE] = P_FINISHED;
            wr64(s, S_DEADLINE, slot + FINISH_HOLD);
            Ok(())
        }
        _ => Err(ERR_PHASE),
    }
}

#[inline(never)]
fn ix_ai_next_ball(c: &Ctx) -> Result<(), u32> {
    // [batter s w, state w, ball w, identity, vrf program, queue w, system, slot hashes]
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if ix.len() != 6 || c.n < 8 {
        return Err(ERR_DATA);
    }
    let batter = &accs[0];
    if !batter.is_signer {
        return Err(ERR_SIGNER);
    }
    let idx = ix[1];
    check_table(c, idx)?;
    let (identity, id_bump) = find_pda(&[IDENTITY_SEED], program_id);
    if !eq32(&identity, accs[3].key())
        || !eq32(accs[4].key(), &VRF_PROGRAM)
        || !eq32(accs[5].key(), &VRF_QUEUE)
        || !eq32(accs[6].key(), &SYSTEM_PROGRAM)
        || !eq32(accs[7].key(), &SLOT_HASHES)
    {
        return Err(ERR_ACCOUNTS);
    }
    let slot = now_slot();
    let s = accs[1].data();
    if s[S_MODE] != MODE_AI {
        return Err(ERR_PHASE);
    }
    let phase = s[S_PHASE];
    if !(phase == P_AI_NEED || (phase == P_AI_REQUESTED && slot > rd64(s, S_DEADLINE))) {
        return Err(ERR_PHASE);
    }
    if !eq32(&s[S_A..S_A + 32], batter.key()) {
        return Err(ERR_TURN);
    }
    let seq = rd32(s, S_SEQ);
    if seq != rd32(ix, 2) {
        return Err(ERR_SEQ);
    }
    // RequestRandomness (scoped identity, regular priority)
    let mut data = [0u8; 8 + 32 + 32 + 4 + 8 + 4 + 2 * 34 + 4 + 5];
    data[0] = 10;
    let mut seed = [0u8; 32];
    let slot_b = slot.to_le_bytes();
    sha256(&[accs[1].key(), &ix[2..6], &slot_b], &mut seed);
    let mut o = 8;
    data[o..o + 32].copy_from_slice(&seed);
    o += 32;
    data[o..o + 32].copy_from_slice(program_id);
    o += 32;
    data[o..o + 4].copy_from_slice(&8u32.to_le_bytes());
    o += 4;
    data[o..o + 8].copy_from_slice(&VRF_CALLBACK);
    o += 8;
    data[o..o + 4].copy_from_slice(&2u32.to_le_bytes());
    o += 4;
    data[o..o + 32].copy_from_slice(accs[1].key());
    data[o + 32] = 0;
    data[o + 33] = 1;
    o += 34;
    data[o..o + 32].copy_from_slice(accs[2].key());
    data[o + 32] = 0;
    data[o + 33] = 1;
    o += 34;
    data[o..o + 4].copy_from_slice(&5u32.to_le_bytes());
    o += 4;
    data[o] = idx;
    data[o + 1..o + 5].copy_from_slice(&ix[2..6]);
    o += 5;
    let id_bump_b = [id_bump];
    invoke(
        accs,
        &Cpi {
            program: &VRF_PROGRAM,
            metas: &[(0, true, true), (3, false, true), (5, true, false), (6, false, false), (7, false, false)],
            data: &data[..o],
            seeds: &[&[IDENTITY_SEED, &id_bump_b]],
        },
    )?;
    s[S_PHASE] = P_AI_REQUESTED;
    wr64(s, S_DEADLINE, slot + VRF_TIMEOUT);
    log("so: ai ball requested");
    Ok(())
}

/// Picks from `weights` (summing to 256) using byte `r`.
fn pick(r: u8, weights: &[u16]) -> u8 {
    let mut acc: u16 = 0;
    for (i, w) in weights.iter().enumerate() {
        acc += *w;
        if (r as u16) < acc {
            return i as u8;
        }
    }
    (weights.len() - 1) as u8
}

#[inline(never)]
fn ix_vrf_callback(c: &Ctx) -> Result<(), u32> {
    // [vrf identity s, state w, ball w]; data: callback disc, randomness 32, idx, seq u32
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 3 || ix.len() < 8 + 32 + 5 {
        return Err(ERR_DATA);
    }
    let (expected, _) = find_pda(&[IDENTITY_SEED, program_id], &VRF_PROGRAM);
    if !accs[0].is_signer || !eq32(&expected, accs[0].key()) {
        return Err(ERR_IDENTITY);
    }
    let rnd = &ix[8..40];
    let idx = ix[40];
    check_table(c, idx)?;
    let slot = now_slot();
    let s = accs[1].data();
    if s[S_MODE] != MODE_AI || s[S_PHASE] != P_AI_REQUESTED {
        return Err(ERR_PHASE);
    }
    let seq = rd32(s, S_SEQ);
    if seq != rd32(ix, 41) {
        return Err(ERR_SEQ);
    }
    if rd16(s, S_TARGET) == 0 {
        wr16(s, S_TARGET, 8 + (rnd[4] % 11) as u16);
    }
    // visible part straight from the VRF output
    let pace = pick(rnd[0], &[64, 90, 102]);
    let line = pick(rnd[1], &[52, 90, 77, 37]);
    let length = pick(rnd[2], &[38, 77, 90, 51]);
    // hidden part also mixes in the private secret, so the VRF output alone does not reveal it
    let b = accs[2].data();
    let mut h = [0u8; 32];
    sha256(&[&b[B_SECRET..B_SECRET + 32], rnd], &mut h);
    let variation = pick(h[0], &[102, 51, 64, 39]);
    let mut next = [0u8; 32];
    sha256(&[&h, b"next"], &mut next);
    b[B_SECRET..B_SECRET + 32].copy_from_slice(&next);
    b[B_ARMED] = ARM_LIVE;
    b[B_VARIATION] = variation;
    wr32(b, B_SEQ, seq);
    let release = slot + AI_RUNUP;
    s[S_PACE] = pace;
    s[S_LINE] = line;
    s[S_LENGTH] = length;
    s[S_ARMED] = 1;
    wr64(s, S_RELEASE, release);
    wr64(s, S_DEADLINE, release + FLIGHT[pace as usize] + SLOWER + W_EDGE as u64 + LIVE_GRACE);
    s[S_PHASE] = P_LIVE;
    log("so: ai bowled");
    Ok(())
}

/// Stores the hidden variation for the bowler's next ball. Reads and writes only the private
/// ball account, so the TEE shows this transaction to the bowler alone.
#[inline(never)]
fn ix_arm(c: &Ctx) -> Result<(), u32> {
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 2 || ix.len() != 7 {
        return Err(ERR_DATA);
    }
    let bowler = &accs[0];
    if !bowler.is_signer {
        return Err(ERR_SIGNER);
    }
    let idx_b = [ix[1]];
    let (bp, _) = find_pda(&[BALL_SEED, &idx_b], program_id);
    if !eq32(&bp, accs[1].key()) || !eq32(accs[1].owner(), program_id) || !accs[1].is_writable {
        return Err(ERR_PDA);
    }
    let b = accs[1].data();
    if b.len() != BALL_SIZE || &b[0..8] != BALL_MAGIC {
        return Err(ERR_DATA);
    }
    if is_zero32(&b[B_READER..B_READER + 32]) || !eq32(&b[B_READER..B_READER + 32], bowler.key()) {
        return Err(ERR_TURN);
    }
    if b[B_ARMED] == ARM_LIVE {
        return Err(ERR_PHASE);
    }
    if rd32(b, B_SEQ) != rd32(ix, 2) {
        return Err(ERR_SEQ);
    }
    let variation = ix[6];
    if variation > 3 {
        return Err(ERR_RANGE);
    }
    b[B_ARMED] = ARM_SET;
    b[B_VARIATION] = variation;
    Ok(())
}

#[inline(never)]
fn ix_concede(c: &Ctx) -> Result<(), u32> {
    let ix = c.ix;
    if ix.len() != 2 || c.n < 7 {
        return Err(ERR_DATA);
    }
    let player = &c.accs[0];
    if !player.is_signer {
        return Err(ERR_SIGNER);
    }
    let idx = ix[1];
    let (_, bb) = check_table(c, idx)?;
    let slot = now_slot();
    let s = c.accs[1].data();
    let phase = s[S_PHASE];
    if phase == P_FREE || phase == P_FINISHED {
        return Err(ERR_PHASE);
    }
    let side = if eq32(&s[S_A..S_A + 32], player.key()) {
        0u8
    } else if s[S_MODE] == MODE_PVP && eq32(&s[S_B..S_B + 32], player.key()) {
        1u8
    } else {
        return Err(ERR_TURN);
    };
    if phase == P_WAITING {
        s[S_PHASE] = P_FREE;
        return Ok(());
    }
    s[S_WINNER] = (side ^ 1) + 1;
    s[S_PHASE] = P_FINISHED;
    s[S_ARMED] = 0;
    wr64(s, S_DEADLINE, slot + FINISH_HOLD);
    log("so: conceded");
    let p = privacy_for(c, idx, bb);
    set_reader(c, &p, None)
}

#[inline(never)]
fn ix_commit_undelegate(c: &Ctx) -> Result<(), u32> {
    // [payer s w, pda w, magic program, magic context w]
    let accs = c.accs;
    if c.n < 4 || c.ix.len() != 3 {
        return Err(ERR_DATA);
    }
    if !accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    if !eq32(accs[2].key(), &MAGIC_PROGRAM) || !eq32(accs[3].key(), &MAGIC_CONTEXT) {
        return Err(ERR_ACCOUNTS);
    }
    let seed: &[u8] = if c.ix[2] == 0 { TABLE_SEED } else { BALL_SEED };
    let idx_b = [c.ix[1]];
    let (key, _) = find_pda(&[seed, &idx_b], c.program_id);
    if !eq32(&key, accs[1].key()) {
        return Err(ERR_PDA);
    }
    invoke(
        accs,
        &Cpi {
            program: &MAGIC_PROGRAM,
            metas: &[(0, accs[0].is_writable, true), (3, true, false), (1, true, false)],
            data: &[2, 0, 0, 0],
            seeds: &[],
        },
    )?;
    log("so: commit+undelegate scheduled");
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
    if a.len() < 4 + 4 + 7 + 4 + 1 || rd32(a, 0) != 2 {
        return Err(ERR_DATA);
    }
    let l0 = rd32(a, 4) as usize;
    if a.len() != 8 + l0 + 4 + 1 || rd32(a, 8 + l0) != 1 {
        return Err(ERR_DATA);
    }
    let seed = &a[8..8 + l0];
    if seed != TABLE_SEED && seed != BALL_SEED {
        return Err(ERR_DATA);
    }
    let idx_b = [a[a.len() - 1]];
    let (key, bump) = find_pda(&[seed, &idx_b], program_id);
    if !eq32(&key, pda.key()) {
        return Err(ERR_PDA);
    }
    let bump_b = [bump];
    let space = buffer.len();
    let mut data = [0u8; 52];
    data[4..12].copy_from_slice(&rent_minimum(space).to_le_bytes());
    data[12..20].copy_from_slice(&(space as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(
        accs,
        &Cpi {
            program: &SYSTEM_PROGRAM,
            metas: &[(2, true, true), (0, true, true)],
            data: &data,
            seeds: &[&[seed, &idx_b, &bump_b]],
        },
    )?;
    pda.data().copy_from_slice(buffer.data());
    log("so: returned to Solana");
    Ok(())
}

#[cfg(not(test))]
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
    } else if ix.len() >= 8 && ix[0..8] == VRF_CALLBACK {
        ix_vrf_callback(&c)
    } else {
        match ix[0] {
            0 => ix_init_table(&c),
            1 => ix_delegate(&c),
            2 => ix_setup_privacy(&c),
            3 => ix_create_match(&c),
            4 => ix_join_match(&c),
            5 => ix_bowl(&c),
            6 => ix_shot(&c),
            7 => ix_advance(&c),
            8 => ix_ai_next_ball(&c),
            9 => ix_concede(&c),
            10 => ix_commit_undelegate(&c),
            11 => ix_arm(&c),
            _ => Err(ERR_TAG),
        }
    };
    match r {
        Ok(()) => 0,
        Err(code) => code as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn o(pace: u8, line: u8, length: u8, variation: u8, shot: u8, grade: u8) -> u16 {
        outcome(&Ball { pace, line, length, variation, shot, grade })
    }
    #[test]
    fn rules() {
        assert_eq!(o(2, 1, 1, 0, 1, 0), 4); // perfect drive, full on off stump
        assert_eq!(o(0, 2, 1, 0, 4, 0), 6); // lofted down the ground to spin
        assert_eq!(o(2, 2, 1, 0, 255, 5) & 0xff, 7); // left a full straight ball: bowled
        assert_eq!(o(2, 0, 2, 0, 255, 5), 0); // good leave outside off
        assert_eq!(o(2, 0, 2, 1, 255, 5) & 0xff, 7); // ...unless it swings back in
        assert_eq!(o(2, 1, 2, 2, 1, 2) >> 8, 4); // mistimed drive at an away-swinger: caught behind
        assert_eq!(o(1, 3, 3, 0, 3, 0), 4); // pull, short on leg
        assert_eq!(o(1, 2, 0, 0, 4, 1) & 0xff, 7); // lofting a yorker: caught
        assert_eq!(grade_for(0), 0);
        assert_eq!(grade_for(-10), 1);
        assert_eq!(grade_for(-20), 2);
        assert_eq!(grade_for(20), 3);
        assert_eq!(grade_for(40), 4);
    }
}
