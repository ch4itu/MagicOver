//! Ludo Freeflow: four colours on one board and no turns, as a MagicBlock Ephemeral Rollup program on Solana devnet.
//!
//! Tables are created once on Solana (the operator pays the rent) and delegated to MagicBlock's devnet rollup
//! (validator MAS1Dt9q...). Players never hold SOL: every action is a zero-fee rollup transaction.
//!
//! Seats belong to Smaran keys. Create and Join are signed by the player's Smaran key (approved once in Smaran's
//! sign-in window) and name a session key: a throwaway key the game page makes for this match. The seat then
//! accepts Start, Roll, Move and Leave signed by either key, so play needs no further approvals and the Smaran key
//! never enters the game page. Rebind, signed by the Smaran key, replaces the session key (another device, or a
//! reloaded page that lost it); the old session key stops working at once.
//!
//! Freeflow: each seat rolls and moves on its own, whenever it has no die to play and no roll outstanding. Nobody
//! waits for a turn. The rollup orders transactions on a table one after another, so two actions never interleave;
//! each is checked against the state it actually meets.
//!
//! Dice come only from MagicBlock VRF. Roll asks the VRF program for randomness bound to (table, match, seat, the
//! seat's action number); the die is applied only when the VRF program's identity signs the callback for exactly that
//! outstanding request. A seat's pending roll never blocks another seat. If no callback arrives within VRF_RETRY
//! seconds, the seat may roll again; the new request replaces the old one, whose late callback is then refused.
//!
//! Every Roll and Move carries the seat's next action number (seq). A repeated or replayed action finds the number
//! already used and is refused, so a retry after a timeout, or the same player on two devices, never applies twice.
//! Move also names the token's current square: if a capture sent that token home in the meantime, the move is refused
//! instead of silently becoming a different move.
//!
//! Rules (the Freeflow page): tokens at -1 (base, stored as 255), 0-50 on the shared ring from the colour's start
//! (0, 13, 26, 39) round to its home lane's entrance, 51-55 the colour's home lane, 56 home. A six enters a token;
//! finishing needs the exact count. Two or more tokens of one colour on a ring square block every other colour from
//! passing or landing there, except on the safe squares, which every colour shares. Landing
//! alone on an opponent's single token off the safe squares (0, 8, 13, 21, 26, 34, 39, 47) sends it back to base.
//! A third six in a row is forfeited. A six or a capture gives a bonus roll (shown only: with no turns, every seat may
//! always roll again once its die is played). First colour with all four tokens home wins.
//!
//! Table PDA ["lf-table", idx], idx < NTABLES, TABLE_SIZE bytes. Instructions (first data byte):
//!   0 InitTable   base [operator s w, table w, system]                                      idx, lamports u64
//!   1 Delegate    base [operator s w, table w, this program, buffer w, record w, metadata w, delegation, system]
//!                                                                                            idx, commit_ms u32
//!   2 Create      ER   [smaran key s, table w]                     idx, session 32, name_len, name (up to 15 bytes)
//!   3 Join        ER   [smaran key s, table w]                     idx, match u32, session 32, name_len, name
//!   4 Start       ER   [player s, table w]                                  idx, match u32
//!   5 Roll        ER   [player s w, table w, identity, vrf program, queue w, system, slot hashes]
//!                                                                            idx, match u32, seat, seq u32
//!   6 Move        ER   [player s, table w]                                  idx, match u32, seat, seq u32, token, from
//!                      token 255 = pass, allowed only when the waiting die has no legal move
//!   7 Leave       ER   [player s, table w]                                  idx, match u32, seat
//!   8 CommitUndelegate ER [operator s w, table w, magic program, magic context w]           idx
//!   9 Rebind      ER   [smaran key s, table w]                              idx, match u32, seat, session 32
//!   Start, Roll, Move and Leave: [player s, ...] is the seat's Smaran key or its session key.
//!  "LFVRFCB1" VRF callback [vrf identity s, table w]               randomness 32, idx, match u32, seat, seq u32
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
// Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz
const VRF_PROGRAM: [u8; 32] = [
    7, 100, 104, 119, 200, 241, 253, 43, 158, 185, 243, 92, 234, 235, 93, 25, 63, 162, 158, 196, 92,
    7, 181, 101, 242, 27, 167, 177, 39, 172, 247, 127,
];
// 5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc (VRF oracle queue on the rollup)
const VRF_QUEUE: [u8; 32] = [
    69, 185, 193, 160, 226, 118, 43, 231, 240, 17, 186, 10, 117, 95, 214, 26, 143, 3, 61, 185, 132,
    75, 237, 89, 242, 59, 5, 128, 136, 40, 202, 167,
];
// SysvarS1otHashes111111111111111111111111111
const SLOT_HASHES: [u8; 32] = [
    6, 167, 213, 23, 25, 47, 10, 175, 198, 242, 101, 227, 251, 119, 204, 122, 218, 130, 197, 41, 208,
    190, 59, 19, 110, 45, 0, 85, 32, 0, 0, 0,
];
// MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57: MagicBlock's devnet rollup (devnet-as.magicblock.app)
const VALIDATOR: [u8; 32] = [
    5, 42, 71, 171, 131, 122, 182, 12, 61, 216, 159, 196, 200, 74, 97, 158, 42, 140, 82, 47, 63, 129,
    136, 254, 226, 67, 138, 88, 169, 118, 23, 178,
];
// HDbeKypeJHXMM5Xs2RHKj8XGQgJeP4jnQ1Jiugf8mXPn: the devnet operator (creates, delegates and returns tables)
const OPERATOR: [u8; 32] = [
    240, 245, 33, 212, 39, 39, 71, 159, 130, 37, 245, 248, 117, 196, 199, 238, 238, 219, 127, 236, 6,
    3, 251, 190, 108, 86, 112, 232, 229, 213, 0, 49,
];
const EXTERNAL_UNDELEGATE: [u8; 8] = [196, 28, 41, 206, 48, 37, 51, 167];
const VRF_CALLBACK: [u8; 8] = *b"LFVRFCB1";

// ---------------------------------------------------------------- layout
const TABLE_SEED: &[u8] = b"lf-table";
const IDENTITY_SEED: &[u8] = b"identity";
const NTABLES: u8 = 8;
const TABLE_SIZE: usize = 512;
const MAGIC: &[u8; 8] = b"LUDOFF01";

const T_IDX: usize = 8;
const T_PHASE: usize = 9;
const T_SEATED: usize = 10; // bit per seated colour
const T_WINNER: usize = 11; // 255: none yet
const T_MATCH: usize = 12; // u32, increases with every new match on this table
const T_CREATED: usize = 16; // i64 unix seconds
const T_ACTIVE: usize = 24; // i64 unix seconds of the last change
const T_EVENTS: usize = 32; // u32 events written this match (the ring keeps the last RING)
const T_SEATS: usize = 40;
const SEAT: usize = 104;
const K_KEY: usize = 0; // the Smaran key that owns the seat
const K_SESSION: usize = 32; // the session key that may play for it
const K_NAME: usize = 64; // length + up to 15 bytes, as claimed by the player
const K_PIECES: usize = 80; // 4 tokens: 255 base, 0-50 ring, 51-55 lane, 56 home
const K_WAIT: usize = 84; // die waiting to be played, 0 none
const K_PENDING: usize = 85; // 1: a VRF roll is outstanding
const K_SIXES: usize = 86; // sixes in a row
const K_BONUS: usize = 87; // 1: the last die or capture earned a bonus roll (display)
const K_LAST: usize = 88; // last die rolled, 0 none
const K_CAPTURES: usize = 89; // opponents' tokens this seat sent home
const K_FLAGS: usize = 90; // 1 seated, 2 left the match
const K_HOME: usize = 91; // tokens home
const K_SEQ: usize = 92; // u32 next action number
const K_REQ: usize = 96; // u32 action number of the outstanding roll
const K_REQ_AT: usize = 100; // u32 unix seconds of the outstanding roll
const T_RING: usize = T_SEATS + 4 * SEAT;
const EVENT: usize = 8;
const RING: usize = 7;
const NAME_MAX: usize = 15;

const PH_FREE: u8 = 0;
const PH_LOBBY: u8 = 1;
const PH_PLAYING: u8 = 2;
const PH_FINISHED: u8 = 3;

// events: kind, seat, then up to six bytes
const E_JOIN: u8 = 1;
const E_START: u8 = 2;
const E_ROLL: u8 = 3; // die
const E_MOVE: u8 = 4; // token, from, to, die
const E_CAPTURE: u8 = 5; // token, square, victim seat, victim token
const E_HOME: u8 = 6; // token
const E_FORFEIT: u8 = 7; // die (the third six)
const E_NOMOVE: u8 = 8; // die
const E_WIN: u8 = 9;
const E_LEAVE: u8 = 10;
const E_PASS: u8 = 11; // die

const BASE_POS: u8 = 255;
const LANE: u8 = 51;
const HOME_POS: u8 = 56;
const STARTS: [u8; 4] = [0, 13, 26, 39];

const VRF_RETRY: i64 = 8; // seconds before a seat may replace a roll whose callback never came
const FINISH_HOLD: i64 = 20; // seconds a finished match stays on the table before a new one may start
const ABANDON: i64 = 900; // seconds without any action after which a lobby or match may be replaced

const MAX_PERMITTED_DATA_INCREASE: usize = 10_240;
const MAX_ACCOUNTS: usize = 10;

// ---------------------------------------------------------------- errors
const ERR_ACCOUNTS: u32 = 1;
const ERR_SIGNER: u32 = 2;
const ERR_OWNER: u32 = 3;
const ERR_DATA: u32 = 4;
const ERR_PHASE: u32 = 5;
const ERR_SEAT: u32 = 6; // not this signer's seat, or the seat is not in play
const ERR_SEQ: u32 = 7; // action number already used or ahead
const ERR_BUSY: u32 = 8; // a die is waiting to be played, or a roll is outstanding
const ERR_PDA: u32 = 9;
const ERR_CPI: u32 = 10;
const ERR_TAG: u32 = 11;
const ERR_RANGE: u32 = 12;
const ERR_IDENTITY: u32 = 13; // callback not signed by the VRF program's identity
const ERR_MATCH: u32 = 14; // another match is on this table now
const ERR_ILLEGAL: u32 = 15;
const ERR_STALE: u32 = 16; // the token is no longer on the named square
const ERR_FULL: u32 = 17;
const ERR_SEATED: u32 = 18; // this key already holds a seat in this match
const ERR_NO_DIE: u32 = 19;
const ERR_PLAYERS: u32 = 20; // fewer than two seated
const ERR_TABLE_BUSY: u32 = 21; // a match is still in progress on this table
const ERR_OPERATOR: u32 = 22;
const ERR_REQUEST: u32 = 23; // the callback is not for the outstanding roll
const ERR_OWNER_ONLY: u32 = 24; // only the seat's Smaran key may do this

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
        sol_try_find_program_address(sb.as_ptr(), seeds.len() as u64, program_id.as_ptr(), out.as_mut_ptr(), &mut bump);
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

fn now_unix() -> i64 {
    let mut buf = [0u8; 40];
    unsafe {
        sol_get_clock_sysvar(buf.as_mut_ptr());
    }
    i64::from_le_bytes([buf[32], buf[33], buf[34], buf[35], buf[36], buf[37], buf[38], buf[39]])
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
    const NULL_META: SolAccountMeta = SolAccountMeta { pubkey: core::ptr::null(), is_writable: false, is_signer: false };
    let mut m: [SolAccountMeta; 8] = [NULL_META; 8];
    let mut infos: [SolAccountInfo; 8] = [accs[0].info(); 8];
    for (i, (idx, w, s)) in cpi.metas.iter().enumerate() {
        m[i] = SolAccountMeta { pubkey: accs[*idx].key, is_writable: *w, is_signer: *s };
        infos[i] = accs[*idx].info();
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
    let rc = unsafe {
        sol_invoke_signed_c(&ix, infos.as_ptr(), cpi.metas.len() as u64, signers.as_ptr(), cpi.seeds.len().min(1) as u64)
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

// ---------------------------------------------------------------- rules (pure: no syscalls, tested on the host)
fn seat_at(seat: usize) -> usize {
    T_SEATS + seat * SEAT
}
fn piece(t: &[u8], seat: usize, i: usize) -> u8 {
    t[seat_at(seat) + K_PIECES + i]
}
fn ring_square(seat: usize, v: u8) -> u8 {
    ((STARTS[seat] as u16 + v as u16) % 52) as u8
}
fn is_safe(r: u8) -> bool {
    matches!(r, 0 | 8 | 13 | 21 | 26 | 34 | 39 | 47)
}
fn in_play(t: &[u8], seat: usize) -> bool {
    t[seat_at(seat) + K_FLAGS] & 3 == 1
}
/// Tokens of `seat` on ring square `r` (lane and home squares are private to their colour).
fn block_count(t: &[u8], seat: usize, r: u8) -> u8 {
    let mut n = 0;
    for i in 0..4 {
        let v = piece(t, seat, i);
        if v < LANE && ring_square(seat, v) == r {
            n += 1;
        }
    }
    n
}
/// The page's legal(): a six enters from base; the exact count finishes; no passing or landing on another colour's
/// block of two or more, except on a safe square.
fn legal(t: &[u8], seat: usize, i: usize, d: u8) -> bool {
    let v = piece(t, seat, i);
    if v == BASE_POS {
        return d == 6;
    }
    if v >= HOME_POS || v + d > HOME_POS {
        return false;
    }
    for step in 1..=d {
        let n = v + step;
        if n >= LANE {
            break;
        }
        let r = ring_square(seat, n);
        if is_safe(r) {
            continue;
        }
        for q in 0..4 {
            if q != seat && block_count(t, q, r) >= 2 {
                return false;
            }
        }
    }
    true
}
fn any_legal(t: &[u8], seat: usize, d: u8) -> bool {
    (0..4).any(|i| legal(t, seat, i, d))
}
fn event(t: &mut [u8], e: [u8; EVENT]) {
    let n = rd32(t, T_EVENTS);
    let at = T_RING + (n as usize % RING) * EVENT;
    t[at..at + EVENT].copy_from_slice(&e);
    wr32(t, T_EVENTS, n.wrapping_add(1));
}
/// Unbiased die from VRF output: the first byte below 252 (= 42 x 6), mod 6, plus 1.
fn die_from(rnd: &[u8]) -> u8 {
    for b in rnd.iter() {
        if *b < 252 {
            return b % 6 + 1;
        }
    }
    rnd[0] % 6 + 1 // all 32 bytes >= 252: probability (4/256)^32
}
/// The page's roll() once the die is known.
fn apply_roll(t: &mut [u8], seat: usize, d: u8) {
    let o = seat_at(seat);
    t[o + K_LAST] = d;
    t[o + K_PENDING] = 0;
    if d == 6 {
        t[o + K_SIXES] += 1;
        if t[o + K_SIXES] >= 3 {
            t[o + K_SIXES] = 0;
            t[o + K_BONUS] = 0;
            event(t, [E_FORFEIT, seat as u8, d, 0, 0, 0, 0, 0]);
            return;
        }
    } else {
        t[o + K_SIXES] = 0;
    }
    t[o + K_BONUS] = 0;
    if any_legal(t, seat, d) {
        t[o + K_WAIT] = d;
        event(t, [E_ROLL, seat as u8, d, 0, 0, 0, 0, 0]);
    } else {
        t[o + K_WAIT] = 0;
        if d == 6 {
            t[o + K_BONUS] = 1;
        }
        event(t, [E_NOMOVE, seat as u8, d, 0, 0, 0, 0, 0]);
    }
}
/// The page's move() for a legal token: enter or advance, capture single opponents off the safe squares, finish,
/// win, bonus. Returns true when the move won the match.
fn apply_move(t: &mut [u8], seat: usize, i: usize, d: u8) -> bool {
    let o = seat_at(seat);
    let v = piece(t, seat, i);
    let n = if v == BASE_POS { 0 } else { v + d };
    t[o + K_PIECES + i] = n;
    t[o + K_WAIT] = 0;
    event(t, [E_MOVE, seat as u8, i as u8, v, n, d, 0, 0]);
    let mut captured = false;
    if n < LANE {
        let r = ring_square(seat, n);
        if !is_safe(r) {
            for q in 0..4 {
                if q == seat || block_count(t, q, r) >= 2 {
                    continue;
                }
                for j in 0..4 {
                    let w = piece(t, q, j);
                    if w < LANE && ring_square(q, w) == r {
                        t[seat_at(q) + K_PIECES + j] = BASE_POS;
                        captured = true;
                        t[o + K_CAPTURES] = t[o + K_CAPTURES].saturating_add(1);
                        event(t, [E_CAPTURE, seat as u8, i as u8, r, q as u8, j as u8, 0, 0]);
                    }
                }
            }
        }
    }
    if n == HOME_POS {
        t[o + K_HOME] += 1;
        event(t, [E_HOME, seat as u8, i as u8, 0, 0, 0, 0, 0]);
    }
    let won = (0..4).all(|j| piece(t, seat, j) == HOME_POS);
    if won {
        t[T_PHASE] = PH_FINISHED;
        t[T_WINNER] = seat as u8;
        event(t, [E_WIN, seat as u8, 0, 0, 0, 0, 0, 0]);
    }
    t[o + K_BONUS] = if !won && (d == 6 || captured) { 1 } else { 0 };
    won
}
/// Clears the table for a new match: every seat empty, every token at base.
fn reset_table(t: &mut [u8]) {
    for x in t[T_PHASE..].iter_mut() {
        *x = 0;
    }
    t[T_WINNER] = 255;
    for s in 0..4 {
        for i in 0..4 {
            t[seat_at(s) + K_PIECES + i] = BASE_POS;
        }
    }
}
fn reclaimable(t: &[u8], now: i64) -> bool {
    let idle = now.saturating_sub(rd64(t, T_ACTIVE) as i64);
    match t[T_PHASE] {
        PH_FREE => true,
        PH_FINISHED => idle >= FINISH_HOLD,
        _ => idle >= ABANDON,
    }
}
/// The seat whose Smaran key is `key`.
fn seat_of(t: &[u8], key: &[u8]) -> Option<usize> {
    (0..4).find(|&s| t[seat_at(s) + K_FLAGS] & 1 != 0 && eq32(&t[seat_at(s) + K_KEY..seat_at(s) + K_KEY + 32], key))
}
/// True when `key` is the seat's Smaran key or its session key.
fn plays_for(t: &[u8], seat: usize, key: &[u8]) -> bool {
    let o = seat_at(seat);
    eq32(&t[o + K_KEY..o + K_KEY + 32], key) || eq32(&t[o + K_SESSION..o + K_SESSION + 32], key)
}
/// `who` is the Smaran key followed by the session key (64 bytes).
fn take_seat(t: &mut [u8], seat: usize, who: &[u8], name: &[u8]) {
    let o = seat_at(seat);
    t[o + K_KEY..o + K_KEY + 64].copy_from_slice(&who[..64]);
    t[o + K_NAME] = name.len() as u8;
    t[o + K_NAME + 1..o + K_NAME + 1 + name.len()].copy_from_slice(name);
    t[o + K_FLAGS] = 1;
    t[T_SEATED] |= 1 << seat;
    event(t, [E_JOIN, seat as u8, 0, 0, 0, 0, 0, 0]);
}
/// A player leaves. In the lobby the seat is freed; during a match the colour is out (its tokens leave the board)
/// and, if only one colour is left in play, that colour wins.
fn leave_seat(t: &mut [u8], seat: usize) {
    let o = seat_at(seat);
    if t[T_PHASE] == PH_LOBBY {
        for x in t[o..o + SEAT].iter_mut() {
            *x = 0;
        }
        for i in 0..4 {
            t[o + K_PIECES + i] = BASE_POS;
        }
        t[T_SEATED] &= !(1 << seat);
        event(t, [E_LEAVE, seat as u8, 0, 0, 0, 0, 0, 0]);
        if t[T_SEATED] == 0 {
            t[T_PHASE] = PH_FREE;
        }
        return;
    }
    t[o + K_FLAGS] |= 2;
    t[o + K_WAIT] = 0;
    t[o + K_PENDING] = 0;
    for i in 0..4 {
        if t[o + K_PIECES + i] != HOME_POS {
            t[o + K_PIECES + i] = BASE_POS;
        }
    }
    event(t, [E_LEAVE, seat as u8, 0, 0, 0, 0, 0, 0]);
    let left: [usize; 4] = [0, 1, 2, 3];
    let remaining = left.iter().filter(|&&s| in_play(t, s)).count();
    if t[T_PHASE] == PH_PLAYING && remaining == 1 {
        let w = left.iter().find(|&&s| in_play(t, s)).copied().unwrap_or(0);
        t[T_PHASE] = PH_FINISHED;
        t[T_WINNER] = w as u8;
        event(t, [E_WIN, w as u8, 0, 0, 0, 0, 0, 0]);
    }
}

// ---------------------------------------------------------------- context
struct Ctx<'a> {
    accs: &'a [Acc],
    n: usize,
    ix: &'a [u8],
    program_id: &'a [u8; 32],
}

/// accs[1] must be the table PDA for `idx`, owned by this program and initialised.
fn table(c: &Ctx, idx: u8) -> Result<&'static mut [u8], u32> {
    if c.n < 2 || idx >= NTABLES {
        return Err(ERR_ACCOUNTS);
    }
    let idx_b = [idx];
    let (pda, _) = find_pda(&[TABLE_SEED, &idx_b], c.program_id);
    let a = &c.accs[1];
    if !eq32(&pda, a.key()) {
        return Err(ERR_PDA);
    }
    if !eq32(a.owner(), c.program_id) || !a.is_writable {
        return Err(ERR_OWNER);
    }
    let t = unsafe { core::slice::from_raw_parts_mut(a.data, a.len()) };
    if t.len() != TABLE_SIZE || &t[0..8] != MAGIC || t[T_IDX] != idx {
        return Err(ERR_DATA);
    }
    Ok(t)
}

/// The signer's own seat in the current match. `at` is where the 4-byte match id starts in the instruction data;
/// the seat byte follows it.
fn own_seat(c: &Ctx, t: &[u8], at: usize) -> Result<usize, u32> {
    if !c.accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    if rd32(c.ix, at) != rd32(t, T_MATCH) {
        return Err(ERR_MATCH);
    }
    let seat = c.ix[at + 4] as usize;
    if seat >= 4 || t[seat_at(seat) + K_FLAGS] & 1 == 0 || !plays_for(t, seat, c.accs[0].key()) {
        return Err(ERR_SEAT);
    }
    Ok(seat)
}

fn name_of(ix: &[u8], at: usize) -> Result<&[u8], u32> {
    let len = ix[at] as usize;
    if len > NAME_MAX || ix.len() != at + 1 + len {
        return Err(ERR_DATA);
    }
    Ok(&ix[at + 1..at + 1 + len])
}

// ---------------------------------------------------------------- instructions
#[inline(never)]
fn ix_init_table(c: &Ctx) -> Result<(), u32> {
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 3 || ix.len() != 10 {
        return Err(ERR_DATA);
    }
    if !accs[0].is_signer || !eq32(accs[0].key(), &OPERATOR) {
        return Err(ERR_OPERATOR);
    }
    let idx = ix[1];
    if idx >= NTABLES {
        return Err(ERR_RANGE);
    }
    let idx_b = [idx];
    let (pda, bump) = find_pda(&[TABLE_SEED, &idx_b], program_id);
    if !eq32(&pda, accs[1].key()) || !eq32(accs[2].key(), &SYSTEM_PROGRAM) {
        return Err(ERR_PDA);
    }
    let bump_b = [bump];
    let mut data = [0u8; 52];
    data[4..12].copy_from_slice(&ix[2..10]);
    data[12..20].copy_from_slice(&(TABLE_SIZE as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(0, true, true), (1, true, true)], data: &data, seeds: &[&[TABLE_SEED, &idx_b, &bump_b]] })?;
    let t = accs[1].data();
    t[0..8].copy_from_slice(MAGIC);
    t[T_IDX] = idx;
    reset_table(t);
    log("ludo: table ready");
    Ok(())
}

#[inline(never)]
fn ix_delegate(c: &Ctx) -> Result<(), u32> {
    // [operator, table, this program, buffer, record, metadata, delegation program, system]
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 8 || ix.len() != 6 {
        return Err(ERR_ACCOUNTS);
    }
    let (payer, pda, buffer) = (&accs[0], &accs[1], &accs[3]);
    if !payer.is_signer || !eq32(payer.key(), &OPERATOR) {
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
    let (key, bump) = find_pda(&[TABLE_SEED, &idx_b], program_id);
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
    invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(1, true, true)], data: &assign, seeds: &[&[TABLE_SEED, &idx_b, &bump_b]] })?;
    // DelegateArgs: commit frequency ms, seeds Vec<Vec<u8>>, validator Option<Pubkey> (always MagicBlock's devnet rollup)
    let mut args = [0u8; 8 + 4 + 4 + (4 + 8) + (4 + 1) + 1 + 32];
    let mut o = 8;
    args[o..o + 4].copy_from_slice(&ix[2..6]);
    o += 4;
    args[o..o + 4].copy_from_slice(&2u32.to_le_bytes());
    o += 4;
    args[o..o + 4].copy_from_slice(&(TABLE_SEED.len() as u32).to_le_bytes());
    o += 4;
    args[o..o + TABLE_SEED.len()].copy_from_slice(TABLE_SEED);
    o += TABLE_SEED.len();
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
            seeds: &[&[TABLE_SEED, &idx_b, &bump_b]],
        },
    )?;
    unsafe {
        *payer.lamports += *buffer.lamports;
        *buffer.lamports = 0;
        *buffer.data_len = 0;
        *buffer.owner = SYSTEM_PROGRAM;
    }
    log("ludo: delegated");
    Ok(())
}

#[inline(never)]
fn ix_create(c: &Ctx) -> Result<(), u32> {
    // [smaran key s, table w]; idx, session 32, name_len, name
    if c.ix.len() < 35 {
        return Err(ERR_DATA);
    }
    let t = table(c, c.ix[1])?;
    if !c.accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    let name = name_of(c.ix, 34)?;
    let now = now_unix();
    if !reclaimable(t, now) {
        return Err(ERR_TABLE_BUSY);
    }
    let next = rd32(t, T_MATCH).wrapping_add(1);
    reset_table(t);
    wr32(t, T_MATCH, next);
    wr64(t, T_CREATED, now as u64);
    wr64(t, T_ACTIVE, now as u64);
    t[T_PHASE] = PH_LOBBY;
    let mut who = [0u8; 64];
    who[..32].copy_from_slice(c.accs[0].key());
    who[32..].copy_from_slice(&c.ix[2..34]);
    take_seat(t, 0, &who, name);
    log("ludo: match created");
    Ok(())
}

#[inline(never)]
fn ix_join(c: &Ctx) -> Result<(), u32> {
    // [smaran key s, table w]; idx, match u32, session 32, name_len, name
    if c.ix.len() < 39 {
        return Err(ERR_DATA);
    }
    let t = table(c, c.ix[1])?;
    if !c.accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    let name = name_of(c.ix, 38)?;
    if t[T_PHASE] != PH_LOBBY {
        return Err(ERR_PHASE);
    }
    if rd32(c.ix, 2) != rd32(t, T_MATCH) {
        return Err(ERR_MATCH);
    }
    if seat_of(t, c.accs[0].key()).is_some() {
        return Err(ERR_SEATED);
    }
    let seat = (0..4).find(|&s| t[seat_at(s) + K_FLAGS] == 0).ok_or(ERR_FULL)?;
    let mut who = [0u8; 64];
    who[..32].copy_from_slice(c.accs[0].key());
    who[32..].copy_from_slice(&c.ix[6..38]);
    take_seat(t, seat, &who, name);
    wr64(t, T_ACTIVE, now_unix() as u64);
    if t[T_SEATED] == 15 {
        t[T_PHASE] = PH_PLAYING;
        event(t, [E_START, seat as u8, 0, 0, 0, 0, 0, 0]);
    }
    log("ludo: joined");
    Ok(())
}

#[inline(never)]
fn ix_start(c: &Ctx) -> Result<(), u32> {
    // [player s, table w]; idx, match u32
    if c.ix.len() != 6 {
        return Err(ERR_DATA);
    }
    let t = table(c, c.ix[1])?;
    if !c.accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    if t[T_PHASE] != PH_LOBBY {
        return Err(ERR_PHASE);
    }
    if rd32(c.ix, 2) != rd32(t, T_MATCH) {
        return Err(ERR_MATCH);
    }
    let seat = (0..4).find(|&s| t[seat_at(s) + K_FLAGS] & 1 != 0 && plays_for(t, s, c.accs[0].key())).ok_or(ERR_SEAT)?;
    if t[T_SEATED].count_ones() < 2 {
        return Err(ERR_PLAYERS);
    }
    t[T_PHASE] = PH_PLAYING;
    wr64(t, T_ACTIVE, now_unix() as u64);
    event(t, [E_START, seat as u8, 0, 0, 0, 0, 0, 0]);
    log("ludo: started");
    Ok(())
}

/// Roll: asks MagicBlock VRF for this seat's die. Nothing about the die is decided here.
#[inline(never)]
fn ix_roll(c: &Ctx) -> Result<(), u32> {
    // [player s w, table w, identity, vrf program, queue w, system, slot hashes]; idx, match u32, seat, seq u32
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 7 || ix.len() != 11 {
        return Err(ERR_DATA);
    }
    let t = table(c, ix[1])?;
    if t[T_PHASE] != PH_PLAYING {
        return Err(ERR_PHASE);
    }
    let seat = own_seat(c, t, 2)?;
    if !in_play(t, seat) {
        return Err(ERR_SEAT);
    }
    let o = seat_at(seat);
    let seq = rd32(ix, 7);
    if seq != rd32(t, o + K_SEQ) {
        return Err(ERR_SEQ);
    }
    let now = now_unix();
    if t[o + K_WAIT] != 0 || (t[o + K_PENDING] != 0 && now.saturating_sub(rd32(t, o + K_REQ_AT) as i64) < VRF_RETRY) {
        return Err(ERR_BUSY);
    }
    let (identity, id_bump) = find_pda(&[IDENTITY_SEED], program_id);
    if !eq32(&identity, accs[2].key())
        || !eq32(accs[3].key(), &VRF_PROGRAM)
        || !eq32(accs[4].key(), &VRF_QUEUE)
        || !eq32(accs[5].key(), &SYSTEM_PROGRAM)
        || !eq32(accs[6].key(), &SLOT_HASHES)
    {
        return Err(ERR_ACCOUNTS);
    }
    wr32(t, o + K_SEQ, seq.wrapping_add(1));
    t[o + K_PENDING] = 1;
    wr32(t, o + K_REQ, seq);
    wr32(t, o + K_REQ_AT, now as u32);
    wr64(t, T_ACTIVE, now as u64);
    // RequestRandomness (scoped identity, regular priority): seed, callback program, callback discriminator,
    // callback accounts [table w], callback args [idx, match u32, seat, seq u32]
    let mut data = [0u8; 8 + 32 + 32 + 4 + 8 + 4 + 34 + 4 + 10];
    data[0] = 10;
    let mut seed = [0u8; 32];
    sha256(&[accs[1].key(), &ix[2..7], &ix[7..11]], &mut seed);
    let mut o2 = 8;
    data[o2..o2 + 32].copy_from_slice(&seed);
    o2 += 32;
    data[o2..o2 + 32].copy_from_slice(program_id);
    o2 += 32;
    data[o2..o2 + 4].copy_from_slice(&8u32.to_le_bytes());
    o2 += 4;
    data[o2..o2 + 8].copy_from_slice(&VRF_CALLBACK);
    o2 += 8;
    data[o2..o2 + 4].copy_from_slice(&1u32.to_le_bytes());
    o2 += 4;
    data[o2..o2 + 32].copy_from_slice(accs[1].key());
    data[o2 + 32] = 0;
    data[o2 + 33] = 1;
    o2 += 34;
    data[o2..o2 + 4].copy_from_slice(&10u32.to_le_bytes());
    o2 += 4;
    data[o2..o2 + 10].copy_from_slice(&ix[1..11]);
    o2 += 10;
    let id_bump_b = [id_bump];
    invoke(
        accs,
        &Cpi {
            program: &VRF_PROGRAM,
            metas: &[(0, true, true), (2, false, true), (4, true, false), (5, false, false), (6, false, false)],
            data: &data[..o2],
            seeds: &[&[IDENTITY_SEED, &id_bump_b]],
        },
    )?;
    log("ludo: roll requested");
    Ok(())
}

/// The VRF program's signed answer to one outstanding roll.
#[inline(never)]
fn ix_vrf_callback(c: &Ctx) -> Result<(), u32> {
    // [vrf identity s, table w]; disc 8, randomness 32, idx, match u32, seat, seq u32
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 2 || ix.len() != 8 + 32 + 10 {
        return Err(ERR_DATA);
    }
    let (expected, _) = find_pda(&[IDENTITY_SEED, program_id], &VRF_PROGRAM);
    if !accs[0].is_signer || !eq32(&expected, accs[0].key()) {
        return Err(ERR_IDENTITY);
    }
    let t = table(c, ix[40])?;
    if t[T_PHASE] != PH_PLAYING {
        return Err(ERR_PHASE);
    }
    if rd32(ix, 41) != rd32(t, T_MATCH) {
        return Err(ERR_MATCH);
    }
    let seat = ix[45] as usize;
    if seat >= 4 || !in_play(t, seat) {
        return Err(ERR_SEAT);
    }
    let o = seat_at(seat);
    if t[o + K_PENDING] == 0 || rd32(t, o + K_REQ) != rd32(ix, 46) {
        return Err(ERR_REQUEST);
    }
    apply_roll(t, seat, die_from(&ix[8..40]));
    wr64(t, T_ACTIVE, now_unix() as u64);
    log("ludo: rolled");
    Ok(())
}

#[inline(never)]
fn ix_move(c: &Ctx) -> Result<(), u32> {
    // [player s, table w]; idx, match u32, seat, seq u32, token (255 pass), from
    let ix = c.ix;
    if ix.len() != 13 {
        return Err(ERR_DATA);
    }
    let t = table(c, ix[1])?;
    if t[T_PHASE] != PH_PLAYING {
        return Err(ERR_PHASE);
    }
    let seat = own_seat(c, t, 2)?;
    if !in_play(t, seat) {
        return Err(ERR_SEAT);
    }
    let o = seat_at(seat);
    let seq = rd32(ix, 7);
    if seq != rd32(t, o + K_SEQ) {
        return Err(ERR_SEQ);
    }
    let d = t[o + K_WAIT];
    if d == 0 {
        return Err(ERR_NO_DIE);
    }
    let token = ix[11];
    if token == 255 {
        if any_legal(t, seat, d) {
            return Err(ERR_ILLEGAL);
        }
        t[o + K_WAIT] = 0;
        event(t, [E_PASS, seat as u8, d, 0, 0, 0, 0, 0]);
    } else {
        let i = token as usize;
        if i >= 4 {
            return Err(ERR_RANGE);
        }
        if piece(t, seat, i) != ix[12] {
            return Err(ERR_STALE);
        }
        if !legal(t, seat, i, d) {
            return Err(ERR_ILLEGAL);
        }
        apply_move(t, seat, i, d);
    }
    wr32(t, o + K_SEQ, seq.wrapping_add(1));
    wr64(t, T_ACTIVE, now_unix() as u64);
    log("ludo: moved");
    Ok(())
}

#[inline(never)]
fn ix_leave(c: &Ctx) -> Result<(), u32> {
    // [player s, table w]; idx, match u32, seat
    if c.ix.len() != 7 {
        return Err(ERR_DATA);
    }
    let t = table(c, c.ix[1])?;
    let seat = own_seat(c, t, 2)?;
    if t[T_PHASE] == PH_FINISHED || t[T_PHASE] == PH_FREE {
        return Err(ERR_PHASE);
    }
    if t[seat_at(seat) + K_FLAGS] & 2 != 0 {
        return Err(ERR_SEAT);
    }
    leave_seat(t, seat);
    wr64(t, T_ACTIVE, now_unix() as u64);
    log("ludo: left");
    Ok(())
}

/// The seat's Smaran key names a new session key (a new device, or a page that lost its session key).
#[inline(never)]
fn ix_rebind(c: &Ctx) -> Result<(), u32> {
    // [smaran key s, table w]; idx, match u32, seat, session 32
    if c.ix.len() != 39 {
        return Err(ERR_DATA);
    }
    let t = table(c, c.ix[1])?;
    let seat = own_seat(c, t, 2)?;
    let o = seat_at(seat);
    if !eq32(&t[o + K_KEY..o + K_KEY + 32], c.accs[0].key()) {
        return Err(ERR_OWNER_ONLY);
    }
    if t[T_PHASE] == PH_FREE || t[o + K_FLAGS] & 2 != 0 {
        return Err(ERR_PHASE);
    }
    t[o + K_SESSION..o + K_SESSION + 32].copy_from_slice(&c.ix[7..39]);
    wr64(t, T_ACTIVE, now_unix() as u64);
    log("ludo: session key replaced");
    Ok(())
}

#[inline(never)]
fn ix_commit_undelegate(c: &Ctx) -> Result<(), u32> {
    // [operator s w, table w, magic program, magic context w]
    let accs = c.accs;
    if c.n < 4 || c.ix.len() != 2 {
        return Err(ERR_DATA);
    }
    if !accs[0].is_signer || !eq32(accs[0].key(), &OPERATOR) {
        return Err(ERR_OPERATOR);
    }
    if !eq32(accs[2].key(), &MAGIC_PROGRAM) || !eq32(accs[3].key(), &MAGIC_CONTEXT) {
        return Err(ERR_ACCOUNTS);
    }
    let idx_b = [c.ix[1]];
    let (key, _) = find_pda(&[TABLE_SEED, &idx_b], c.program_id);
    if !eq32(&key, accs[1].key()) {
        return Err(ERR_PDA);
    }
    invoke(accs, &Cpi { program: &MAGIC_PROGRAM, metas: &[(0, accs[0].is_writable, true), (3, true, false), (1, true, false)], data: &[2, 0, 0, 0], seeds: &[] })?;
    log("ludo: commit+undelegate scheduled");
    Ok(())
}

#[inline(never)]
fn ix_undelegate_callback(c: &Ctx) -> Result<(), u32> {
    // [table w, buffer s, payer s w, system]; data: disc, borsh Vec<Vec<u8>> seeds
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
    if a.len() != 8 + l0 + 4 + 1 || rd32(a, 8 + l0) != 1 || &a[8..8 + l0] != TABLE_SEED {
        return Err(ERR_DATA);
    }
    let idx_b = [a[a.len() - 1]];
    let (key, bump) = find_pda(&[TABLE_SEED, &idx_b], program_id);
    if !eq32(&key, pda.key()) {
        return Err(ERR_PDA);
    }
    let bump_b = [bump];
    let space = buffer.len();
    let mut data = [0u8; 52];
    data[4..12].copy_from_slice(&rent_minimum(space).to_le_bytes());
    data[12..20].copy_from_slice(&(space as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(2, true, true), (0, true, true)], data: &data, seeds: &[&[TABLE_SEED, &idx_b, &bump_b]] })?;
    pda.data().copy_from_slice(buffer.data());
    log("ludo: returned to Solana");
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
            2 => ix_create(&c),
            3 => ix_join(&c),
            4 => ix_start(&c),
            5 => ix_roll(&c),
            6 => ix_move(&c),
            7 => ix_leave(&c),
            8 => ix_commit_undelegate(&c),
            9 => ix_rebind(&c),
            _ => Err(ERR_TAG),
        }
    };
    match r {
        Ok(()) => 0,
        Err(code) => code as u64,
    }
}

#[cfg(test)]
mod tests;
