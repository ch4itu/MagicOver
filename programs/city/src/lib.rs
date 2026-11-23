//! SmaranCity shared city: who is walking where, as a MagicBlock Ephemeral Rollup program on Solana devnet.
//!
//! One room PDA ["city-room", 0] (42 slots) is created once on Solana (the operator pays the rent) and delegated to
//! MagicBlock's devnet rollup (validator MAS1Dt9q...). Players never hold SOL: every update is a zero-fee rollup
//! transaction signed by the player's own Smaran key, so a slot always belongs to the key that signed it.
//! A page shows the name a slot claims only after checking it against the Smaran registry.
//!
//! Instructions (first data byte):
//!   0 InitRoom    base [operator s w, room w, system]                                  idx, lamports u64
//!   1 Delegate    base [operator s w, room w, this program, buffer w, record w, metadata w, delegation, system]
//!                                                                                       idx, commit_ms u32
//!   2 Move        ER   [player s, room w]             idx, x u32, y u32 (tiles x 256), facing, flags, name_len, name
//!   3 Leave       ER   [player s, room w]                                               idx
//!   8 CommitUndelegate ER [operator s w, room w, magic program, magic context w]        idx
//!  MagicBlock undelegate callback (8-byte discriminator)
//! A slot is the signer's own, else a free one, else one not updated for STALE seconds.
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


// ---------------------------------------------------------------- layout
const ROOM_SEED: &[u8] = b"city-room";
const NROOMS: u8 = 1;
const SLOTS: usize = 42;
const SLOT: usize = 96;
const HEADER: usize = 16;
const ROOM_SIZE: usize = 4_112; // the room as created; 42 slots of 96 bytes fit after the header
const _: () = assert!(HEADER + SLOTS * SLOT <= ROOM_SIZE);
const MAGIC: &[u8; 8] = b"SMCITY01";
const H_IDX: usize = 8;
const H_VERSION: usize = 9;
const H_COUNT: usize = 12; // u32 updates applied (wraps)
const S_KEY: usize = 0; // the Smaran key that walks here
const S_X: usize = 32; // u32, tiles x 256
const S_Y: usize = 36; // u32, tiles x 256
const S_FACING: usize = 40; // 0-7
const S_FLAGS: usize = 41; // 1 present, 2 moving
const S_NAME: usize = 42; // length + up to 32 bytes, as claimed by the player (pages check it against the registry)
const S_SEEN: usize = 92; // u32 unix seconds of the last update
const NAME_MAX: usize = 32; // a Smaran name is 3 to 32 characters
const STALE: i64 = 90; // seconds without an update after which a slot may be taken by someone else

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


// ---------------------------------------------------------------- context
struct Ctx<'a> {
    accs: &'a [Acc],
    n: usize,
    ix: &'a [u8],
    program_id: &'a [u8; 32],
}

/// accs[1] must be the room PDA for `idx`, owned by this program, writable and initialised.
fn room(c: &Ctx, idx: u8) -> Result<&'static mut [u8], u32> {
    if c.n < 2 || idx >= NROOMS {
        return Err(ERR_ACCOUNTS);
    }
    let idx_b = [idx];
    let (pda, _) = find_pda(&[ROOM_SEED, &idx_b], c.program_id);
    let a = &c.accs[1];
    if !eq32(&pda, a.key()) {
        return Err(ERR_PDA);
    }
    if !eq32(a.owner(), c.program_id) || !a.is_writable {
        return Err(ERR_OWNER);
    }
    let t = unsafe { core::slice::from_raw_parts_mut(a.data, a.len()) };
    if t.len() != ROOM_SIZE || &t[0..8] != MAGIC || t[H_IDX] != idx {
        return Err(ERR_DATA);
    }
    Ok(t)
}

/// Pure slot choice (tested on the host): the signer's own slot, else a free one, else the stalest stale one.
fn choose_slot(t: &[u8], who: &[u8], now: i64) -> Option<usize> {
    let mut free: Option<usize> = None;
    let mut stale: Option<(usize, i64)> = None;
    for i in 0..SLOTS {
        let o = HEADER + i * SLOT;
        let present = t[o + S_FLAGS] & 1 == 1;
        if present && eq32(&t[o + S_KEY..o + S_KEY + 32], who) {
            return Some(i);
        }
        if !present {
            if free.is_none() {
                free = Some(i);
            }
            continue;
        }
        let seen = rd32(t, o + S_SEEN) as i64;
        if now - seen > STALE && stale.map_or(true, |(_, s)| seen < s) {
            stale = Some((i, seen));
        }
    }
    free.or(stale.map(|(i, _)| i))
}

fn write_slot(t: &mut [u8], i: usize, who: &[u8], x: u32, y: u32, facing: u8, moving: bool, name: &[u8], now: i64) {
    let o = HEADER + i * SLOT;
    t[o + S_KEY..o + S_KEY + 32].copy_from_slice(who);
    wr32(t, o + S_X, x);
    wr32(t, o + S_Y, y);
    t[o + S_FACING] = facing;
    t[o + S_FLAGS] = 1 | if moving { 2 } else { 0 };
    for b in t[o + S_NAME..o + S_NAME + 1 + NAME_MAX].iter_mut() {
        *b = 0;
    }
    t[o + S_NAME] = name.len() as u8;
    t[o + S_NAME + 1..o + S_NAME + 1 + name.len()].copy_from_slice(name);
    wr32(t, o + S_SEEN, now as u32);
    let n = rd32(t, H_COUNT).wrapping_add(1);
    wr32(t, H_COUNT, n);
}

// ---------------------------------------------------------------- instructions
#[inline(never)]
fn ix_init_room(c: &Ctx) -> Result<(), u32> {
    // [operator s w, room w, system]; data: tag, idx, lamports u64
    let (accs, ix, program_id) = (c.accs, c.ix, c.program_id);
    if c.n < 3 || ix.len() != 10 {
        return Err(ERR_DATA);
    }
    if !accs[0].is_signer || !eq32(accs[0].key(), &OPERATOR) {
        return Err(ERR_OPERATOR);
    }
    let idx = ix[1];
    if idx >= NROOMS {
        return Err(ERR_RANGE);
    }
    let idx_b = [idx];
    let (pda, bump) = find_pda(&[ROOM_SEED, &idx_b], program_id);
    if !eq32(&pda, accs[1].key()) || !eq32(accs[2].key(), &SYSTEM_PROGRAM) {
        return Err(ERR_PDA);
    }
    let bump_b = [bump];
    let mut data = [0u8; 52];
    data[4..12].copy_from_slice(&ix[2..10]);
    data[12..20].copy_from_slice(&(ROOM_SIZE as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(0, true, true), (1, true, true)], data: &data, seeds: &[&[ROOM_SEED, &idx_b, &bump_b]] })?;
    let t = accs[1].data();
    t[0..8].copy_from_slice(MAGIC);
    t[H_IDX] = idx;
    t[H_VERSION] = 1;
    log("city: room ready");
    Ok(())
}

#[inline(never)]
fn ix_move(c: &Ctx) -> Result<(), u32> {
    // [player s, room w]; data: tag, idx, x u32, y u32, facing, flags, name_len, name
    let ix = c.ix;
    if ix.len() < 13 {
        return Err(ERR_DATA);
    }
    if !c.accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    let t = room(c, ix[1])?;
    let len = ix[12] as usize;
    if len > NAME_MAX || ix.len() != 13 + len || ix[10] > 7 {
        return Err(ERR_DATA);
    }
    let now = now_unix();
    let who = c.accs[0].key();
    let i = choose_slot(t, who, now).ok_or(ERR_FULL)?;
    write_slot(t, i, who, rd32(ix, 2), rd32(ix, 6), ix[10], ix[11] & 2 == 2, &ix[13..13 + len], now);
    Ok(())
}

#[inline(never)]
fn ix_leave(c: &Ctx) -> Result<(), u32> {
    // [player s, room w]; data: tag, idx
    if c.ix.len() != 2 {
        return Err(ERR_DATA);
    }
    if !c.accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    let t = room(c, c.ix[1])?;
    let who = c.accs[0].key();
    for i in 0..SLOTS {
        let o = HEADER + i * SLOT;
        if t[o + S_FLAGS] & 1 == 1 && eq32(&t[o + S_KEY..o + S_KEY + 32], who) {
            for b in t[o..o + SLOT].iter_mut() {
                *b = 0;
            }
        }
    }
    let n = rd32(t, H_COUNT).wrapping_add(1);
    wr32(t, H_COUNT, n);
    Ok(())
}

#[inline(never)]
fn ix_delegate(c: &Ctx) -> Result<(), u32> {
    // [operator, room, this program, buffer, record, metadata, delegation program, system]
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
    let (key, bump) = find_pda(&[ROOM_SEED, &idx_b], program_id);
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
    invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(1, true, true)], data: &assign, seeds: &[&[ROOM_SEED, &idx_b, &bump_b]] })?;
    // DelegateArgs: commit frequency ms, seeds Vec<Vec<u8>>, validator Option<Pubkey> (always MagicBlock's devnet rollup)
    let mut args = [0u8; 8 + 4 + 4 + (4 + 16) + (4 + 1) + 1 + 32];
    let mut o = 8;
    args[o..o + 4].copy_from_slice(&ix[2..6]);
    o += 4;
    args[o..o + 4].copy_from_slice(&2u32.to_le_bytes());
    o += 4;
    args[o..o + 4].copy_from_slice(&(ROOM_SEED.len() as u32).to_le_bytes());
    o += 4;
    args[o..o + ROOM_SEED.len()].copy_from_slice(ROOM_SEED);
    o += ROOM_SEED.len();
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
            seeds: &[&[ROOM_SEED, &idx_b, &bump_b]],
        },
    )?;
    unsafe {
        *payer.lamports += *buffer.lamports;
        *buffer.lamports = 0;
        *buffer.data_len = 0;
        *buffer.owner = SYSTEM_PROGRAM;
    }
    log("city: delegated");
    Ok(())
}

#[inline(never)]
fn ix_commit_undelegate(c: &Ctx) -> Result<(), u32> {
    // [operator s w, room w, magic program, magic context w]
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
    let (key, _) = find_pda(&[ROOM_SEED, &idx_b], c.program_id);
    if !eq32(&key, accs[1].key()) {
        return Err(ERR_PDA);
    }
    invoke(accs, &Cpi { program: &MAGIC_PROGRAM, metas: &[(0, accs[0].is_writable, true), (3, true, false), (1, true, false)], data: &[2, 0, 0, 0], seeds: &[] })?;
    log("city: commit+undelegate scheduled");
    Ok(())
}

#[inline(never)]
fn ix_undelegate_callback(c: &Ctx) -> Result<(), u32> {
    // [room w, buffer s, payer s w, system]; data: disc, borsh Vec<Vec<u8>> seeds
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
    if a.len() != 8 + l0 + 4 + 1 || rd32(a, 8 + l0) != 1 || &a[8..8 + l0] != ROOM_SEED {
        return Err(ERR_DATA);
    }
    let idx_b = [a[a.len() - 1]];
    let (key, bump) = find_pda(&[ROOM_SEED, &idx_b], program_id);
    if !eq32(&key, pda.key()) {
        return Err(ERR_PDA);
    }
    let bump_b = [bump];
    let space = buffer.len();
    let mut data = [0u8; 52];
    data[4..12].copy_from_slice(&rent_minimum(space).to_le_bytes());
    data[12..20].copy_from_slice(&(space as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(accs, &Cpi { program: &SYSTEM_PROGRAM, metas: &[(2, true, true), (0, true, true)], data: &data, seeds: &[&[ROOM_SEED, &idx_b, &bump_b]] })?;
    pda.data().copy_from_slice(buffer.data());
    log("city: returned to Solana");
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
            0 => ix_init_room(&c),
            1 => ix_delegate(&c),
            2 => ix_move(&c),
            3 => ix_leave(&c),
            8 => ix_commit_undelegate(&c),
            _ => Err(ERR_TAG),
        }
    };
    match r {
        Ok(()) => 0,
        Err(code) => code as u64,
    }
}
