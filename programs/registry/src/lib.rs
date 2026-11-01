//! Sabha Vault Registry for Solana + MagicBlock Ephemeral Rollups.
//!
//! Name-and-password onboarding: the browser keeps a random Ed25519 key,
//! encrypts it (Argon2id + XChaCha20-Poly1305, SCK1 envelope) and stores the
//! envelope here, indexed by a hash of the player name. Registry shards are
//! delegated to a MagicBlock ER, so players register and act with zero-fee ER
//! transactions signed by their own key. No wallet, no SOL.
//!
//! Instructions (first data byte):
//!   0 InitShard        [payer s w, shard w, system]                     data: shard u8, lamports u64
//!   1 Register         [owner s, shard w]                               data: name_hash 32, pow_nonce u64, envelope 212
//!   2 Touch            [owner s, shard w]                               data: name_hash 32
//!   3 Delegate         [payer s w, shard w, this program, buffer w, delegation_record w,
//!                       delegation_metadata w, delegation program, system]  data: shard u8, has_validator u8, validator 32
//!   4 Commit           [payer s, shard w, magic program, magic context w]
//!   5 CommitUndelegate [payer s, shard w, magic program, magic context w]
//!   196.. (8-byte MagicBlock undelegation callback) [shard w, buffer s, payer s w, system]
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
    fn sol_get_rent_sysvar(addr: *mut u8) -> u64;
    fn abort() -> !;
}

fn log(msg: &str) {
    unsafe { sol_log_(msg.as_ptr(), msg.len() as u64) }
}

// ---------------------------------------------------------------- constants
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
const EXTERNAL_UNDELEGATE: [u8; 8] = [196, 28, 41, 206, 48, 37, 51, 167];

const SHARD_SEED: &[u8] = b"sabha-reg";
const NSHARDS: u8 = 4;
const HEADER: usize = 64;
const ENTRY: usize = 288;
const SLOTS: usize = 35;
const SHARD_SIZE: usize = HEADER + SLOTS * ENTRY; // 10,144 bytes
const ENVELOPE: usize = 212;
const POW_BITS: u32 = 16;
const MAGIC: &[u8; 8] = b"SABREG01";
const POW_DOMAIN: &[u8] = b"SABHA-SOL-POW-V1";

const MAX_PERMITTED_DATA_INCREASE: usize = 10_240;
const MAX_ACCOUNTS: usize = 10;

// Custom program errors: a nonzero return value n (< 2^32) is ProgramError::Custom(n).
fn custom(code: u32) -> u64 {
    code as u64
}
const ERR_ACCOUNTS: u32 = 1;
const ERR_SIGNER: u32 = 2;
const ERR_OWNER: u32 = 3;
const ERR_DATA: u32 = 4;
const ERR_SHARD: u32 = 5;
const ERR_POW: u32 = 6;
const ERR_EXISTS: u32 = 7;
const ERR_FULL: u32 = 8;
const ERR_NOT_FOUND: u32 = 9;
const ERR_NOT_OWNER: u32 = 10;
const ERR_PDA: u32 = 11;
const ERR_CPI: u32 = 12;
const ERR_TAG: u32 = 13;

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
fn find_pda(seeds: &[&[u8]], program_id: &[u8; 32]) -> ([u8; 32], u8) {
    let mut sb: [SolBytes; 4] = [
        SolBytes { addr: core::ptr::null(), len: 0 },
        SolBytes { addr: core::ptr::null(), len: 0 },
        SolBytes { addr: core::ptr::null(), len: 0 },
        SolBytes { addr: core::ptr::null(), len: 0 },
    ];
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
    let mut sb: [SolBytes; 6] = [
        SolBytes { addr: core::ptr::null(), len: 0 },
        SolBytes { addr: core::ptr::null(), len: 0 },
        SolBytes { addr: core::ptr::null(), len: 0 },
        SolBytes { addr: core::ptr::null(), len: 0 },
        SolBytes { addr: core::ptr::null(), len: 0 },
        SolBytes { addr: core::ptr::null(), len: 0 },
    ];
    for (i, p) in parts.iter().enumerate() {
        sb[i] = SolBytes { addr: p.as_ptr(), len: p.len() as u64 };
    }
    unsafe {
        sol_sha256(sb.as_ptr(), parts.len() as u64, out.as_mut_ptr());
    }
}

fn leading_zero_bits(h: &[u8; 32]) -> u32 {
    let mut bits = 0u32;
    for b in h.iter() {
        if *b == 0 {
            bits += 8;
        } else {
            bits += b.leading_zeros();
            break;
        }
    }
    bits
}

fn eq32(a: &[u8], b: &[u8]) -> bool {
    let mut d = 0u8;
    for i in 0..32 {
        d |= a[i] ^ b[i];
    }
    d == 0
}

/// A cross-program call. `metas` lists (account index into `accs`, writable, signer).
/// Passed by reference: upstream LLVM BPF spills arguments beyond five registers above r10,
/// which faults on SVM runtimes that keep stack frame gaps (the MagicBlock ER does).
struct Cpi<'a> {
    program: &'a [u8; 32],
    metas: &'a [(usize, bool, bool)],
    data: &'a [u8],
    seeds: &'a [&'a [&'a [u8]]],
}

#[inline(never)]
fn invoke(accs: &[Acc], cpi: &Cpi) -> Result<(), u32> {
    let (program, metas, data, signer_seeds) = (cpi.program, cpi.metas, cpi.data, cpi.seeds);
    const NULL_META: SolAccountMeta = SolAccountMeta {
        pubkey: core::ptr::null(),
        is_writable: false,
        is_signer: false,
    };
    let mut m: [SolAccountMeta; 8] = [NULL_META; 8];
    let mut infos: [SolAccountInfo; 8] = [accs[0].info(); 8];
    for (i, (idx, w, s)) in metas.iter().enumerate() {
        m[i] = SolAccountMeta { pubkey: accs[*idx].key, is_writable: *w, is_signer: *s };
        infos[i] = accs[*idx].info();
    }
    // flatten signer seeds (max 2 signers x 4 seeds)
    let mut seed_bytes: [[SolBytes; 4]; 2] = [
        [
            SolBytes { addr: core::ptr::null(), len: 0 },
            SolBytes { addr: core::ptr::null(), len: 0 },
            SolBytes { addr: core::ptr::null(), len: 0 },
            SolBytes { addr: core::ptr::null(), len: 0 },
        ],
        [
            SolBytes { addr: core::ptr::null(), len: 0 },
            SolBytes { addr: core::ptr::null(), len: 0 },
            SolBytes { addr: core::ptr::null(), len: 0 },
            SolBytes { addr: core::ptr::null(), len: 0 },
        ],
    ];
    let mut signers: [SolSignerSeeds; 2] = [
        SolSignerSeeds { addr: core::ptr::null(), len: 0 },
        SolSignerSeeds { addr: core::ptr::null(), len: 0 },
    ];
    for (si, seeds) in signer_seeds.iter().enumerate() {
        for (j, s) in seeds.iter().enumerate() {
            seed_bytes[si][j] = SolBytes { addr: s.as_ptr(), len: s.len() as u64 };
        }
        signers[si] = SolSignerSeeds { addr: seed_bytes[si].as_ptr(), len: seeds.len() as u64 };
    }
    let ix = SolInstruction {
        program_id: program,
        accounts: m.as_ptr(),
        account_len: metas.len() as u64,
        data: data.as_ptr(),
        data_len: data.len() as u64,
    };
    let rc = unsafe {
        sol_invoke_signed_c(
            &ix,
            infos.as_ptr(),
            metas.len() as u64,
            signers.as_ptr(),
            signer_seeds.len() as u64,
        )
    };
    if rc != 0 {
        return Err(ERR_CPI);
    }
    Ok(())
}

fn rent_minimum(len: usize) -> u64 {
    // Rent sysvar: lamports_per_byte_year u64, exemption_threshold f64, burn_percent u8
    let mut buf = [0u8; 24];
    unsafe {
        sol_get_rent_sysvar(buf.as_mut_ptr());
    }
    let per_byte = u64::from_le_bytes([buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]]);
    let thr_bits = u64::from_le_bytes([buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15]]);
    let base = (128 + len as u64) * per_byte;
    if thr_bits == 0x3FF0_0000_0000_0000 {
        base // threshold 1.0
    } else {
        base * 2 // threshold 2.0 (or unknown: overpay, never underpay)
    }
}

// ---------------------------------------------------------------- registry logic
/// Instruction context, passed by reference for the same register-limit reason as `Cpi`.
struct Ctx<'a> {
    accs: &'a [Acc],
    n: usize,
    ix: &'a [u8],
    program_id: &'a [u8; 32],
}

fn shard_ok(shard: &Acc, program_id: &[u8; 32]) -> Result<(), u32> {
    if !shard.is_writable {
        return Err(ERR_ACCOUNTS);
    }
    if !eq32(shard.owner(), program_id) {
        return Err(ERR_OWNER);
    }
    let d = shard.data();
    if d.len() != SHARD_SIZE || &d[0..8] != MAGIC {
        return Err(ERR_SHARD);
    }
    Ok(())
}

/// Linear probe from the name's home slot. Returns FOUND|slot, FREE|slot, or 0 when the shard is full.
const FOUND: u32 = 0x100;
const FREE: u32 = 0x200;
fn probe(d: &[u8], name_hash: &[u8]) -> u32 {
    let start = (u16::from_le_bytes([name_hash[1], name_hash[2]]) as usize) % SLOTS;
    for k in 0..SLOTS {
        let s = (start + k) % SLOTS;
        let e = HEADER + s * ENTRY;
        if d[e] == 0 {
            // entries are never deleted, so the first empty slot ends the chain
            return FREE | s as u32;
        }
        if eq32(&d[e + 8..e + 40], name_hash) {
            return FOUND | s as u32;
        }
    }
    0
}

#[inline(never)]
fn ix_init_shard(c: &Ctx) -> Result<(), u32> {
    let (accs, n, ix, program_id) = (c.accs, c.n, c.ix, c.program_id);
    if n < 3 || ix.len() != 10 {
        return Err(ERR_ACCOUNTS);
    }
    let (payer, shard) = (&accs[0], &accs[1]);
    if !payer.is_signer {
        return Err(ERR_SIGNER);
    }
    let idx = ix[1];
    if idx >= NSHARDS {
        return Err(ERR_DATA);
    }
    let lamports = u64::from_le_bytes([ix[2], ix[3], ix[4], ix[5], ix[6], ix[7], ix[8], ix[9]]);
    let idx_b = [idx];
    let (pda, bump) = find_pda(&[SHARD_SEED, &idx_b], program_id);
    if !eq32(&pda, shard.key()) {
        return Err(ERR_PDA);
    }
    let bump_b = [bump];
    // system CreateAccount: u32 tag 0, lamports u64, space u64, owner [32]
    let mut data = [0u8; 52];
    data[4..12].copy_from_slice(&lamports.to_le_bytes());
    data[12..20].copy_from_slice(&(SHARD_SIZE as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(
        accs,
        &Cpi {
            program: &SYSTEM_PROGRAM,
            metas: &[(0, true, true), (1, true, true)],
            data: &data,
            seeds: &[&[SHARD_SEED, &idx_b, &bump_b]],
        },
    )?;
    // accs[1] memory now reflects the new size (runtime updates serialized input)
    let d = shard.data();
    if d.len() != SHARD_SIZE {
        return Err(ERR_SHARD);
    }
    d[0..8].copy_from_slice(MAGIC);
    d[8] = idx;
    d[9] = NSHARDS;
    d[10..12].copy_from_slice(&(SLOTS as u16).to_le_bytes());
    d[12..14].copy_from_slice(&(ENTRY as u16).to_le_bytes());
    d[14] = POW_BITS as u8;
    log("sabha: shard ready");
    Ok(())
}

#[inline(never)]
fn ix_register(c: &Ctx) -> Result<(), u32> {
    let (accs, n, ix, program_id) = (c.accs, c.n, c.ix, c.program_id);
    if n < 2 || ix.len() != 1 + 32 + 8 + ENVELOPE {
        return Err(ERR_DATA);
    }
    let (owner, shard) = (&accs[0], &accs[1]);
    if !owner.is_signer {
        return Err(ERR_SIGNER);
    }
    shard_ok(shard, program_id)?;
    let name_hash = &ix[1..33];
    let nonce = &ix[33..41];
    let envelope = &ix[41..41 + ENVELOPE];
    let d = shard.data();
    if name_hash[0] % NSHARDS != d[8] {
        return Err(ERR_SHARD);
    }
    // proof of work binds program, name and owner key
    let mut h = [0u8; 32];
    sha256(&[POW_DOMAIN, program_id, name_hash, owner.key(), nonce], &mut h);
    if leading_zero_bits(&h) < POW_BITS {
        return Err(ERR_POW);
    }
    let pr = probe(d, name_hash);
    if pr & FOUND != 0 {
        return Err(ERR_EXISTS);
    }
    if pr & FREE == 0 {
        return Err(ERR_FULL);
    }
    let s = (pr & 0xff) as usize;
    let e = HEADER + s * ENTRY;
    d[e] = 1; // used
    d[e + 1] = 1; // entry version
    d[e + 4..e + 8].copy_from_slice(&0u32.to_le_bytes());
    d[e + 8..e + 40].copy_from_slice(name_hash);
    d[e + 40..e + 72].copy_from_slice(owner.key());
    d[e + 72..e + 72 + ENVELOPE].copy_from_slice(envelope);
    let count = u32::from_le_bytes([d[16], d[17], d[18], d[19]]) + 1;
    d[16..20].copy_from_slice(&count.to_le_bytes());
    log("sabha: registered");
    unsafe { sol_log_64_(d[8] as u64, s as u64, count as u64, 0, 0) };
    Ok(())
}

#[inline(never)]
fn ix_touch(c: &Ctx) -> Result<(), u32> {
    let (accs, n, ix, program_id) = (c.accs, c.n, c.ix, c.program_id);
    if n < 2 || ix.len() != 33 {
        return Err(ERR_DATA);
    }
    let (owner, shard) = (&accs[0], &accs[1]);
    if !owner.is_signer {
        return Err(ERR_SIGNER);
    }
    shard_ok(shard, program_id)?;
    let d = shard.data();
    let pr = probe(d, &ix[1..33]);
    if pr & FOUND == 0 {
        return Err(ERR_NOT_FOUND);
    }
    let s = (pr & 0xff) as usize;
    let e = HEADER + s * ENTRY;
    if !eq32(&d[e + 40..e + 72], owner.key()) {
        return Err(ERR_NOT_OWNER);
    }
    let c = u32::from_le_bytes([d[e + 4], d[e + 5], d[e + 6], d[e + 7]]).wrapping_add(1);
    d[e + 4..e + 8].copy_from_slice(&c.to_le_bytes());
    log("sabha: touch signed by player");
    unsafe { sol_log_64_(s as u64, c as u64, 0, 0, 0) };
    Ok(())
}

#[inline(never)]
fn ix_delegate(c: &Ctx) -> Result<(), u32> {
    let (accs, n, ix, program_id) = (c.accs, c.n, c.ix, c.program_id);
    // [payer, shard, this program, buffer, record, metadata, delegation program, system]
    // data: [3, shard, has_validator, validator 32, commit_frequency_ms u32]
    if n < 8 || ix.len() != 39 {
        return Err(ERR_ACCOUNTS);
    }
    let (payer, shard, buffer) = (&accs[0], &accs[1], &accs[3]);
    if !payer.is_signer {
        return Err(ERR_SIGNER);
    }
    if !eq32(accs[2].key(), program_id) || !eq32(accs[6].key(), &DELEGATION_PROGRAM) {
        return Err(ERR_ACCOUNTS);
    }
    shard_ok(shard, program_id)?;
    let idx = ix[1];
    let idx_b = [idx];
    let (pda, bump) = find_pda(&[SHARD_SEED, &idx_b], program_id);
    if !eq32(&pda, shard.key()) {
        return Err(ERR_PDA);
    }
    let bump_b = [bump];
    let (buf_pda, buf_bump) = find_pda(&[b"buffer", shard.key()], program_id);
    if !eq32(&buf_pda, buffer.key()) {
        return Err(ERR_PDA);
    }
    let buf_bump_b = [buf_bump];
    let len = shard.len();

    // 1. create buffer PDA (0 lamports, closed again before the end of this tx)
    let mut data = [0u8; 52];
    data[12..20].copy_from_slice(&(len as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(
        accs,
        &Cpi {
            program: &SYSTEM_PROGRAM,
            metas: &[(0, true, true), (3, true, true)],
            data: &data,
            seeds: &[&[b"buffer", shard.key(), &buf_bump_b]],
        },
    )?;
    // 2. copy shard -> buffer, zero shard
    {
        let src = shard.data();
        let dst = buffer.data();
        dst.copy_from_slice(src);
        for b in src.iter_mut() {
            *b = 0;
        }
    }
    // 3. owner -> system (direct), then system Assign -> delegation program
    unsafe {
        *shard.owner = SYSTEM_PROGRAM;
    }
    let mut assign = [0u8; 36];
    assign[0] = 1; // SystemInstruction::Assign
    assign[4..36].copy_from_slice(&DELEGATION_PROGRAM);
    invoke(
        accs,
        &Cpi {
            program: &SYSTEM_PROGRAM,
            metas: &[(1, true, true)],
            data: &assign,
            seeds: &[&[SHARD_SEED, &idx_b, &bump_b]],
        },
    )?;
    // 4. CPI delegation program: Delegate (discriminator 0u64), args
    let mut args = [0u8; 8 + 4 + 4 + (4 + 9) + (4 + 1) + 1 + 32];
    let mut o = 8; // discriminator 0
    args[o..o + 4].copy_from_slice(&ix[35..39]); // commit_frequency_ms
    o += 4;
    args[o..o + 4].copy_from_slice(&2u32.to_le_bytes()); // seeds count
    o += 4;
    args[o..o + 4].copy_from_slice(&(SHARD_SEED.len() as u32).to_le_bytes());
    o += 4;
    args[o..o + SHARD_SEED.len()].copy_from_slice(SHARD_SEED);
    o += SHARD_SEED.len();
    args[o..o + 4].copy_from_slice(&1u32.to_le_bytes());
    o += 4;
    args[o] = idx;
    o += 1;
    if ix[2] == 1 {
        args[o] = 1;
        o += 1;
        args[o..o + 32].copy_from_slice(&ix[3..35]);
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
            seeds: &[&[SHARD_SEED, &idx_b, &bump_b]],
        },
    )?;
    // 5. close buffer back to payer
    unsafe {
        *payer.lamports += *buffer.lamports;
        *buffer.lamports = 0;
        *buffer.data_len = 0;
        *buffer.owner = SYSTEM_PROGRAM;
    }
    log("sabha: shard delegated");
    Ok(())
}

#[inline(never)]
fn ix_commit(c: &Ctx, undelegate: bool) -> Result<(), u32> {
    let (accs, n) = (c.accs, c.n);
    // [payer s, shard w, magic program, magic context w]
    if n < 4 {
        return Err(ERR_ACCOUNTS);
    }
    if !accs[0].is_signer {
        return Err(ERR_SIGNER);
    }
    if !eq32(accs[2].key(), &MAGIC_PROGRAM) || !eq32(accs[3].key(), &MAGIC_CONTEXT) {
        return Err(ERR_ACCOUNTS);
    }
    let data: [u8; 4] = if undelegate { [2, 0, 0, 0] } else { [1, 0, 0, 0] };
    invoke(
        accs,
        &Cpi {
            program: &MAGIC_PROGRAM,
            metas: &[(0, accs[0].is_writable, true), (3, true, false), (1, true, false)],
            data: &data,
            seeds: &[],
        },
    )?;
    log(if undelegate { "sabha: commit+undelegate scheduled" } else { "sabha: commit scheduled" });
    Ok(())
}

#[inline(never)]
fn ix_undelegate_callback(c: &Ctx) -> Result<(), u32> {
    let (accs, n, ix, program_id) = (c.accs, c.n, c.ix, c.program_id);
    // [shard w, buffer s, payer s w, system]
    if n < 4 {
        return Err(ERR_ACCOUNTS);
    }
    let (shard, buffer) = (&accs[0], &accs[1]);
    if !buffer.is_signer {
        return Err(ERR_SIGNER);
    }
    if !eq32(buffer.owner(), &DELEGATION_PROGRAM) {
        return Err(ERR_OWNER);
    }
    let (ub, _) = find_pda(&[b"undelegate-buffer", shard.key()], &DELEGATION_PROGRAM);
    if !eq32(&ub, buffer.key()) {
        return Err(ERR_PDA);
    }
    // args: borsh Vec<Vec<u8>>, we expect exactly [SHARD_SEED, [idx]]
    let a = &ix[8..];
    if a.len() != 4 + 4 + SHARD_SEED.len() + 4 + 1 {
        return Err(ERR_DATA);
    }
    if a[0..4] != 2u32.to_le_bytes() || &a[8..8 + SHARD_SEED.len()] != SHARD_SEED {
        return Err(ERR_DATA);
    }
    let idx = a[a.len() - 1];
    let idx_b = [idx];
    let (pda, bump) = find_pda(&[SHARD_SEED, &idx_b], program_id);
    if !eq32(&pda, shard.key()) {
        return Err(ERR_PDA);
    }
    let bump_b = [bump];
    let space = buffer.len();
    let lamports = rent_minimum(space);
    let mut data = [0u8; 52];
    data[4..12].copy_from_slice(&lamports.to_le_bytes());
    data[12..20].copy_from_slice(&(space as u64).to_le_bytes());
    data[20..52].copy_from_slice(program_id);
    invoke(
        accs,
        &Cpi {
            program: &SYSTEM_PROGRAM,
            metas: &[(2, true, true), (0, true, true)],
            data: &data,
            seeds: &[&[SHARD_SEED, &idx_b, &bump_b]],
        },
    )?;
    shard.data().copy_from_slice(buffer.data());
    log("sabha: shard returned to Solana");
    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    let mut accs: [Acc; MAX_ACCOUNTS] = [Acc::empty(); MAX_ACCOUNTS];
    let (n, ix, program_id) = parse(input, &mut accs);
    if ix.is_empty() {
        return custom(ERR_TAG);
    }
    let c = Ctx { accs: &accs, n, ix, program_id };
    let r = if ix.len() >= 8 && ix[0..8] == EXTERNAL_UNDELEGATE {
        ix_undelegate_callback(&c)
    } else {
        match ix[0] {
            0 => ix_init_shard(&c),
            1 => ix_register(&c),
            2 => ix_touch(&c),
            3 => ix_delegate(&c),
            4 => ix_commit(&c, false),
            5 => ix_commit(&c, true),
            _ => Err(ERR_TAG),
        }
    };
    match r {
        Ok(()) => 0,
        Err(code) => custom(code),
    }
}
