//! Host stand-ins for the runtime services the program uses, so the real `entrypoint` can be run by the test harness.
//! LOCAL TEST SUPPORT ONLY. Nothing here talks to a chain. Program derived addresses are a fake (sha256 based, no curve
//! check), and cross-program calls are recorded and, for the system program, applied to the in-memory accounts.
//! A pass here is "PASSED LOCALLY", never live verification.
#![allow(dead_code)]
use crate::{Acc, Cpi, ERR_CPI, SYSTEM_PROGRAM};
use std::cell::RefCell;

#[derive(Clone, Debug)]
pub(crate) struct Recorded {
    pub program: [u8; 32],
    pub data: Vec<u8>,
    /// (account key, writable, signer)
    pub metas: Vec<([u8; 32], bool, bool)>,
    pub seeds: Vec<Vec<Vec<u8>>>,
}

struct State {
    logs: Vec<String>,
    slot: u64,
    unix: i64,
    program: [u8; 32],
    cpis: Vec<Recorded>,
    fail_program: Option<[u8; 32]>,
    per_byte: u64,
    threshold_bits: u64,
}

impl State {
    fn new() -> State {
        State {
            logs: Vec::new(),
            slot: 1_000,
            unix: 1_800_000_000,
            program: [0u8; 32],
            cpis: Vec::new(),
            fail_program: None,
            per_byte: 3480,
            threshold_bits: 0x4000_0000_0000_0000, // 2.0
        }
    }
}

thread_local! {
    static HOST: RefCell<State> = RefCell::new(State::new());
}

// ---------------------------------------------------------------- controls used by the tests
pub(crate) fn reset() {
    HOST.with(|h| *h.borrow_mut() = State::new());
}
pub(crate) fn set_program(id: [u8; 32]) {
    HOST.with(|h| h.borrow_mut().program = id);
}
pub(crate) fn set_clock(slot: u64, unix: i64) {
    HOST.with(|h| {
        let mut h = h.borrow_mut();
        h.slot = slot;
        h.unix = unix;
    });
}
pub(crate) fn advance_secs(s: i64) {
    HOST.with(|h| {
        let mut h = h.borrow_mut();
        h.unix += s;
        h.slot += (s as u64) * 2;
    });
}
pub(crate) fn clear_cpis() {
    HOST.with(|h| h.borrow_mut().cpis.clear());
}
pub(crate) fn cpis() -> Vec<Recorded> {
    HOST.with(|h| h.borrow().cpis.clone())
}
pub(crate) fn logs() -> Vec<String> {
    HOST.with(|h| h.borrow().logs.clone())
}
pub(crate) fn fail_cpi_to(p: Option<[u8; 32]>) {
    HOST.with(|h| h.borrow_mut().fail_program = p);
}
pub(crate) fn set_rent(per_byte: u64, threshold_bits: u64) {
    HOST.with(|h| {
        let mut h = h.borrow_mut();
        h.per_byte = per_byte;
        h.threshold_bits = threshold_bits;
    });
}

// ---------------------------------------------------------------- the runtime surface lib.rs expects
pub(crate) fn log(msg: &str) {
    HOST.with(|h| h.borrow_mut().logs.push(msg.to_string()));
}

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be,
    0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa,
    0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85,
    0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f,
    0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub(crate) fn sha256(parts: &[&[u8]], out: &mut [u8; 32]) {
    let mut msg: Vec<u8> = Vec::new();
    for p in parts {
        msg.extend_from_slice(p);
    }
    let bit_len = (msg.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v[7] = v[6];
            v[6] = v[5];
            v[5] = v[4];
            v[4] = v[3].wrapping_add(t1);
            v[3] = v[2];
            v[2] = v[1];
            v[1] = v[0];
            v[0] = t1.wrapping_add(t2);
        }
        for i in 0..8 {
            h[i] = h[i].wrapping_add(v[i]);
        }
    }
    for i in 0..8 {
        out[i * 4..i * 4 + 4].copy_from_slice(&h[i].to_be_bytes());
    }
}

/// FAKE program address: sha256(seeds || program id || "pda"). Always reports bump 255. It matches what `invoke` accepts as
/// a PDA signature, so tests are self-consistent. It is NOT the real derivation.
pub(crate) fn find_pda(seeds: &[&[u8]], program_id: &[u8; 32]) -> ([u8; 32], u8) {
    (fake_pda(seeds, program_id), 255)
}

fn fake_pda(seeds: &[&[u8]], program_id: &[u8; 32]) -> [u8; 32] {
    let mut parts: Vec<&[u8]> = seeds.to_vec();
    parts.push(program_id);
    parts.push(b"pda");
    let mut out = [0u8; 32];
    sha256(&parts, &mut out);
    out
}

pub(crate) fn clock() -> (u64, i64) {
    HOST.with(|h| {
        let h = h.borrow();
        (h.slot, h.unix)
    })
}

pub(crate) fn rent_per_byte_and_threshold() -> (u64, u64) {
    HOST.with(|h| {
        let h = h.borrow();
        (h.per_byte, h.threshold_bits)
    })
}

/// A cross-program call. Enforces what the real runtime would: callee privileges cannot exceed the caller's, and a signer
/// that did not sign the outer transaction must be a PDA of the calling program named by the supplied seeds.
pub(crate) fn invoke(accs: &[Acc], cpi: &Cpi) -> Result<(), u32> {
    HOST.with(|h| {
        let mut h = h.borrow_mut();
        let rec = Recorded {
            program: *cpi.program,
            data: cpi.data.to_vec(),
            metas: cpi.metas.iter().map(|(i, w, s)| (*accs[*i].key(), *w, *s)).collect(),
            seeds: cpi.seeds.iter().map(|ss| ss.iter().map(|s| s.to_vec()).collect()).collect(),
        };
        h.cpis.push(rec);
        if h.fail_program == Some(*cpi.program) {
            return Err(ERR_CPI);
        }
        for (idx, writable, signer) in cpi.metas.iter() {
            let a = &accs[*idx];
            if *writable && !a.is_writable {
                return Err(ERR_CPI);
            }
            if *signer && !a.is_signer {
                // must be a PDA of the calling program: some seed set, with a trailing bump of 255, derives the key
                let ok = cpi.seeds.iter().any(|ss| {
                    if ss.is_empty() || ss[ss.len() - 1] != [255u8] {
                        return false;
                    }
                    fake_pda(&ss[..ss.len() - 1], &h.program) == *a.key()
                });
                if !ok {
                    return Err(ERR_CPI);
                }
            }
        }
        let p = *cpi.program;
        if p == SYSTEM_PROGRAM {
            let d = cpi.data;
            if d.len() >= 4 && d[..4] == [0, 0, 0, 0] && d.len() == 52 {
                // CreateAccount: lamports, space, owner
                let lamports = u64::from_le_bytes(d[4..12].try_into().unwrap());
                let space = u64::from_le_bytes(d[12..20].try_into().unwrap());
                let (from, to) = (&accs[cpi.metas[0].0], &accs[cpi.metas[1].0]);
                if to.len() != 0 || !to.owner().iter().all(|b| *b == 0) || to.lamports() > 0 {
                    return Err(ERR_CPI); // already in use
                }
                if space > 10_240 || from.lamports() < lamports {
                    return Err(ERR_CPI);
                }
                unsafe {
                    *from.lamports -= lamports;
                    *to.lamports += lamports;
                    *to.data_len = space;
                    *to.owner = d[20..52].try_into().unwrap();
                    core::ptr::write_bytes(to.data, 0, space as usize);
                }
                return Ok(());
            }
            if d.len() == 12 && d[..4] == [2, 0, 0, 0] {
                let (from, to) = (&accs[cpi.metas[0].0], &accs[cpi.metas[1].0]);
                let amount = u64::from_le_bytes(d[4..12].try_into().unwrap());
                let balance = to.lamports().checked_add(amount).ok_or(ERR_CPI)?;
                if from.owner() != &SYSTEM_PROGRAM || from.len() != 0 || from.lamports() < amount {
                    return Err(ERR_CPI);
                }
                unsafe {
                    *from.lamports -= amount;
                    *to.lamports = balance;
                }
                return Ok(());
            }
            if d.len() == 12 && d[..4] == [8, 0, 0, 0] {
                let account = &accs[cpi.metas[0].0];
                let len = u64::from_le_bytes(d[4..12].try_into().unwrap());
                if account.len() != 0 || account.owner() != &SYSTEM_PROGRAM || len > 10_240 {
                    return Err(ERR_CPI);
                }
                unsafe {
                    *account.data_len = len;
                    core::ptr::write_bytes(account.data, 0, len as usize);
                }
                return Ok(());
            }
            if d.len() == 36 && d[..4] == [1, 0, 0, 0] {
                // Assign
                let a = &accs[cpi.metas[0].0];
                unsafe {
                    *a.owner = d[4..36].try_into().unwrap();
                }
                return Ok(());
            }
            return Err(ERR_CPI);
        }
        // Magic program, delegation program and VRF program calls are recorded only
        if p == crate::MAGIC_PROGRAM || p == crate::DELEGATION_PROGRAM || p == crate::VRF_PROGRAM {
            return Ok(());
        }
        Err(ERR_CPI)
    })
}
