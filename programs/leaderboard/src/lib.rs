//! Magic Over result register. Records verified completed friend matches, once per table/match.
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
const TEE_VALIDATOR: [u8;32] = [5, 61, 71, 26, 133, 158, 115, 46, 104, 11, 201, 88, 248, 65, 7, 43, 143, 63, 188, 25, 115, 155, 230, 151, 196, 198, 129, 18, 111, 140, 31, 116];
const ERR_SPONSOR_LOW: u32 = 17;
const BOARD_SEED: &[u8] = b"mo-board";
const TABLE_SEED: &[u8] = BOARD_SEED;
const BALL_SEED: &[u8] = b"unused";
const EXTERNAL_UNDELEGATE: [u8; 8] = [196, 28, 41, 206, 48, 37, 51, 167];
const BOARD_SIZE: usize = 10208;
const HEADER: usize = 128;
const ROW: usize = 72;
const CAPACITY: usize = 140;
const BOARD_MAGIC: &[u8; 8] = b"MOBOARD1";
// Fixed sponsor and game identities for this Devnet deployment.
const OPERATOR: [u8; 32] = [240, 245, 33, 212, 39, 39, 71, 159, 130, 37, 245, 248, 117, 196, 199, 238, 238, 219, 127, 236, 6, 3, 251, 190, 108, 86, 112, 232, 229, 213, 0, 49];
const GAME: [u8; 32] = [251, 179, 49, 100, 143, 53, 186, 149, 42, 91, 247, 113, 222, 82, 52, 122, 187, 41, 20, 155, 65, 117, 249, 17, 90, 26, 19, 193, 242, 82, 135, 190];
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

struct Ctx<'a> { accs: &'a [Acc], n: usize, ix: &'a [u8], program_id: &'a [u8; 32] }
fn board(c: &Ctx) -> Result<u8, u32> {
    if c.n < 2 || !c.accs[1].is_writable { return Err(ERR_ACCOUNTS); }
    if !eq32(c.accs[1].owner(),c.program_id) { return Err(ERR_OWNER); }
    let d = c.accs[1].data();
    if d.len()!=BOARD_SIZE || &d[..8]!=BOARD_MAGIC || d[8]!=1 || d[9]>1 || rd16(d,10) as usize > CAPACITY { return Err(ERR_DATA); }
    let (key,bump) = find_pda(&[BOARD_SEED, &[d[9]]], c.program_id);
    if !eq32(&key,c.accs[1].key()) { return Err(ERR_PDA); }
    Ok(bump)
}
fn authority(c: &Ctx) -> Result<(),u32> {
    board(c)?;
    if !c.accs[0].is_signer || !eq32(c.accs[0].key(),&c.accs[1].data()[16..48]) { return Err(ERR_SIGNER); }
    Ok(())
}
fn init(c: &Ctx) -> Result<(),u32> {
    if c.n<3 || c.ix.len()!=10 || c.ix[1]>1 || !c.accs[0].is_signer { return Err(ERR_DATA); }
    if !eq32(c.accs[0].key(), &OPERATOR) { return Err(ERR_SIGNER); }
    let (key,bump)=find_pda(&[BOARD_SEED,&[c.ix[1]]],c.program_id);
    if !eq32(&key,c.accs[1].key()) || !eq32(c.accs[2].key(),&SYSTEM_PROGRAM) { return Err(ERR_PDA); }
    let mut data=[0u8;52]; data[4..12].copy_from_slice(&c.ix[2..10]);
    data[12..20].copy_from_slice(&(BOARD_SIZE as u64).to_le_bytes());data[20..52].copy_from_slice(c.program_id);
    invoke(c.accs,&Cpi{program:&SYSTEM_PROGRAM,metas:&[(0,true,true),(1,true,true)],data:&data,seeds:&[&[BOARD_SEED,&[c.ix[1]],&[bump]]]})?;
    let d=c.accs[1].data(); d[..8].copy_from_slice(BOARD_MAGIC);d[8]=1;d[9]=c.ix[1];d[16..48].copy_from_slice(c.accs[0].key());
    Ok(())
}
// Pure state transition: validate first, reserve both rows, then update exactly once.
fn apply_result(d: &mut [u8], s: &[u8], idx: usize, expected: u32) -> Result<(),u32> {
    if d.len()!=BOARD_SIZE || &d[..8]!=BOARD_MAGIC || d[8]!=1 || idx>=8 { return Err(ERR_DATA); }
    if s.len()!=384 || &s[..8]!=b"SOVER001" || s[8] as usize != idx || s[9]!=6 || s[10]!=0 || !(1..=3).contains(&s[14]) { return Err(ERR_PHASE); }
    let mid=rd32(s,80);
    if mid!=expected || mid==0 { return Err(ERR_SEQ); }
    if is_zero32(&s[16..48]) || is_zero32(&s[48..80]) || eq32(&s[16..48],&s[48..80]) { return Err(ERR_TURN); }
    if mid<=rd32(d,48+idx*4) { return Ok(()); }
    let count=rd16(d,10) as usize;
    if count>CAPACITY { return Err(ERR_DATA); }
    let mut rows=[count;2]; let mut added=0;
    for side in 0..2 {
        let key=&s[16+side*32..48+side*32];
        if let Some(row)=(0..count).find(|r|eq32(&d[HEADER+r*ROW..HEADER+r*ROW+32],key)) { rows[side]=row; }
        else { rows[side]=count+added;added+=1; }
    }
    if count+added>CAPACITY { return Err(ERR_RANGE); }
    if rd32(d,12)==u32::MAX {return Err(ERR_RANGE);}
    for side in 0..2 { let o=HEADER+rows[side]*ROW;let stat=if s[14]==3{64}else if s[14] as usize==side+1{56}else{60};if rd32(d,o+stat)==u32::MAX{return Err(ERR_RANGE);} }
    for side in 0..2 {
        let o=HEADER+rows[side]*ROW;
        if rows[side]>=count { d[o..o+32].copy_from_slice(&s[16+side*32..48+side*32]); }
        let name=256+side*24; let len=(s[name] as usize).min(23);
        d[o+32..o+56].fill(0);d[o+32]=len as u8;d[o+33..o+33+len].copy_from_slice(&s[name+1..name+1+len]);
        let stat=if s[14]==3 {64} else if s[14] as usize==side+1 {56} else {60};
        wr32(d,o+stat,rd32(d,o+stat).checked_add(1).ok_or(ERR_RANGE)?);
    }
    wr16(d,10,(count+added) as u16);wr32(d,12,rd32(d,12).checked_add(1).ok_or(ERR_RANGE)?);wr32(d,48+idx*4,mid);
    Ok(())
}
// Only a new, validated result can spend the board's bounded save reserve.
fn schedule_snapshot(c: &Ctx, context: usize, vault: usize) -> Result<(),u32> {
    let bump=board(c)?;
    let (fee_vault,_)=find_pda(&[b"magic-fee-vault",&TEE_VALIDATOR],&DELEGATION_PROGRAM);
    if !eq32(c.accs[vault].key(),&fee_vault) || !eq32(c.accs[context].key(),&MAGIC_CONTEXT) { return Err(ERR_ACCOUNTS); }
    if unsafe{*c.accs[1].lamports}<rent_minimum(BOARD_SIZE)+100_000 { return Err(ERR_SPONSOR_LOW); }
    let board_id=[c.accs[1].data()[9]];
    invoke(c.accs,&Cpi{program:&MAGIC_PROGRAM,metas:&[(1,true,true),(context,true,false),(vault,true,false),(1,true,false)],data:&[1,0,0,0],seeds:&[&[BOARD_SEED,&board_id,&[bump]]]})
}
fn record(c: &Ctx) -> Result<(),u32> {
    if c.n<6 || c.ix.len()!=6 || !c.accs[0].is_signer {return Err(ERR_DATA);}
    board(c)?;let idx=c.ix[1];
    if !eq32(c.accs[3].key(),&MAGIC_PROGRAM) {return Err(ERR_ACCOUNTS);}
    if idx>=8 || !eq32(c.accs[2].owner(),&GAME) {return Err(ERR_OWNER);}
    let (key,_)=find_pda(&[b"so-table",&[idx]],&GAME);
    if !eq32(c.accs[2].key(),&key) {return Err(ERR_PDA);}
    let before=rd32(c.accs[1].data(),12);
    apply_result(c.accs[1].data(),c.accs[2].data(),idx as usize,rd32(c.ix,2))?;
    if rd32(c.accs[1].data(),12)>before {schedule_snapshot(c,4,5)?;}
    Ok(())
}
#[inline(never)]
fn ix_delegate(c: &Ctx) -> Result<(), u32> {
    // [payer, pda, this program, buffer, record, metadata, delegation program, system]
    authority(c)?;
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
    if idx > 1 || ix[2] != 0 { return Err(ERR_DATA); }
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

fn commit(c: &Ctx) -> Result<(),u32> {
    authority(c)?;
    if c.n<5 || c.ix.len()!=2 || c.ix[1]!=c.accs[1].data()[9] || !eq32(c.accs[2].key(),&MAGIC_PROGRAM) {return Err(ERR_ACCOUNTS);}
    schedule_snapshot(c,3,4)
}
#[cfg(not(test))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn entrypoint(input:*mut u8)->u64 {
    let mut accs=[Acc::empty();MAX_ACCOUNTS];let (n,ix,program_id)=parse(input,&mut accs);
    let c=Ctx{accs:&accs,n,ix,program_id};
    let r=if ix.len()>=8 && ix[..8]==EXTERNAL_UNDELEGATE {ix_undelegate_callback(&c)}
    else {match ix.first(){Some(0)=>init(&c),Some(1)=>ix_delegate(&c),Some(2)=>record(&c),Some(3)=>commit(&c),_=>Err(ERR_TAG)}};
    match r {Ok(())=>0,Err(e)=>e as u64}
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->(Vec<u8>,Vec<u8>) {
        let mut d=vec![0;BOARD_SIZE];d[..8].copy_from_slice(BOARD_MAGIC);d[8]=1;
        let mut s=vec![0;384];s[..8].copy_from_slice(b"SOVER001");s[9]=6;s[14]=1;s[16..48].fill(1);s[48..80].fill(2);wr32(&mut s,80,1);(d,s)
    }
    #[test]fn repeat_is_idempotent(){let(mut d,s)=fixture();apply_result(&mut d,&s,0,1).unwrap();let old=d.clone();apply_result(&mut d,&s,0,1).unwrap();assert_eq!(d,old);assert_eq!(rd32(&d,12),1);assert_eq!(rd32(&d,HEADER+56),1);assert_eq!(rd32(&d,HEADER+ROW+60),1);}
    #[test]fn rematch_and_reversed_sides(){let(mut d,mut s)=fixture();apply_result(&mut d,&s,0,1).unwrap();wr32(&mut s,80,2);s[14]=2;apply_result(&mut d,&s,0,2).unwrap();assert_eq!(rd16(&d,10),2);assert_eq!(rd32(&d,HEADER+56),1);assert_eq!(rd32(&d,HEADER+60),1);assert_eq!(rd32(&d,HEADER+ROW+56),1);}
    #[test]fn draws(){let(mut d,mut s)=fixture();s[14]=3;apply_result(&mut d,&s,0,1).unwrap();assert_eq!(rd32(&d,HEADER+64),1);assert_eq!(rd32(&d,HEADER+ROW+64),1);}
    #[test]fn invalid_states_do_not_change_totals(){for(case,value)in[(9,2),(10,1),(14,0),(8,8)]{let(mut d,mut s)=fixture();let old=d.clone();s[case]=value;assert!(apply_result(&mut d,&s,0,1).is_err());assert_eq!(d,old);}}
    #[test]fn reject_self_and_missing_opponent(){for v in [0,1]{let(mut d,mut s)=fixture();s[48..80].fill(v);assert!(apply_result(&mut d,&s,0,1).is_err());assert_eq!(rd32(&d,12),0);}}
    #[test]fn reject_stale_and_wrong_ids(){let(mut d,s)=fixture();assert!(apply_result(&mut d,&s,0,2).is_err());assert!(apply_result(&mut d,&s,1,1).is_err());}
    #[test]fn capacity_checks_both_players_before_writing(){let(mut d,s)=fixture();wr16(&mut d,10,139);let old=d.clone();assert!(apply_result(&mut d,&s,0,1).is_err());assert_eq!(d,old);}
    #[test]fn malformed_board_and_short_state(){let(mut d,s)=fixture();d[8]=2;assert!(apply_result(&mut d,&s,0,1).is_err());let(mut d,_)=fixture();assert!(apply_result(&mut d,&s[..300],0,1).is_err());}
    #[test]fn independent_tables_deduplicate_separately(){let(mut d,mut s)=fixture();apply_result(&mut d,&s,0,1).unwrap();s[8]=1;apply_result(&mut d,&s,1,1).unwrap();assert_eq!(rd32(&d,12),2);assert_eq!(rd16(&d,10),2);}
    #[test]fn older_match_is_ignored_after_a_newer_result(){let(mut d,mut s)=fixture();wr32(&mut s,80,5);apply_result(&mut d,&s,0,5).unwrap();let old=d.clone();wr32(&mut s,80,4);apply_result(&mut d,&s,0,4).unwrap();assert_eq!(d,old);}
    #[test]fn full_board_still_accepts_known_players(){let(mut d,mut s)=fixture();apply_result(&mut d,&s,0,1).unwrap();wr16(&mut d,10,140);wr32(&mut s,80,2);apply_result(&mut d,&s,0,2).unwrap();assert_eq!(rd32(&d,12),2);}
    #[test]fn replacing_table_does_not_erase_totals(){let(mut d,mut s)=fixture();apply_result(&mut d,&s,0,1).unwrap();s[16..48].fill(3);s[48..80].fill(4);wr32(&mut s,80,2);apply_result(&mut d,&s,0,2).unwrap();assert_eq!(rd16(&d,10),4);assert_eq!(rd32(&d,HEADER+56),1);}
    #[test]fn match_overflow_rejected_before_changes(){let(mut d,s)=fixture();wr32(&mut d,12,u32::MAX);let old=d.clone();assert!(apply_result(&mut d,&s,0,1).is_err());assert_eq!(d,old);}
    #[test]fn player_overflow_rejected_before_changes(){let(mut d,mut s)=fixture();apply_result(&mut d,&s,0,1).unwrap();wr32(&mut d,HEADER+56,u32::MAX);wr32(&mut s,80,2);let old=d.clone();assert!(apply_result(&mut d,&s,0,2).is_err());assert_eq!(d,old);}
    #[test]fn names_never_determine_identity(){let(mut d,mut s)=fixture();s[256]=4;s[257..261].copy_from_slice(b"same");s[280]=4;s[281..285].copy_from_slice(b"same");apply_result(&mut d,&s,0,1).unwrap();assert_eq!(rd16(&d,10),2);}

}
