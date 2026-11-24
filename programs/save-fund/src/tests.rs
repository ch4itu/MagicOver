//! Host harness for the Smaran save fund. LOCAL TESTS ONLY (classification: PASSED LOCALLY).
//!
//! The real `entrypoint` runs on accounts serialized in the loader's input format; `host.rs` stands in for the runtime
//! (fake PDA derivation, recorded cross-program calls, a settable clock and rent). Every successful call is also checked
//! the way the runtime would check it: lamports are conserved, and only accounts this program owns, or system accounts
//! that signed, may lose lamports.
use super::*;
use crate::host;

const PROG: [u8; 32] = [0x77; 32];
const SYS: [u8; 32] = [0u8; 32];
const DELEG: [u8; 32] = [0xde; 32];
const RICH: u64 = 10_000_000_000;
const T0: i64 = 20_000 * 86_400 + 12 * 3_600; // noon UTC

#[derive(Clone)]
struct A {
    key: [u8; 32],
    owner: [u8; 32],
    lamports: u64,
    data: Vec<u8>,
}
/// (account index, signer, writable)
type M = (usize, bool, bool);

fn call(w: &mut Vec<A>, metas: &[M], ix: &[u8]) -> u32 {
    host::set_program(PROG);
    host::clear_cpis();
    let snapshot = w.clone();
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(&(metas.len() as u64).to_le_bytes());
    let mut offs = Vec::new();
    for (i, s, wr) in metas {
        let a = &w[*i];
        offs.push(buf.len());
        buf.push(0xff);
        buf.push(*s as u8);
        buf.push(*wr as u8);
        buf.push(0);
        buf.extend_from_slice(&[0u8; 4]);
        buf.extend_from_slice(&a.key);
        buf.extend_from_slice(&a.owner);
        buf.extend_from_slice(&a.lamports.to_le_bytes());
        buf.extend_from_slice(&(a.data.len() as u64).to_le_bytes());
        buf.extend_from_slice(&a.data);
        buf.extend(std::iter::repeat(0u8).take(10_240));
        while buf.len() % 8 != 0 {
            buf.push(0);
        }
        buf.extend_from_slice(&0u64.to_le_bytes());
    }
    buf.extend_from_slice(&(ix.len() as u64).to_le_bytes());
    buf.extend_from_slice(ix);
    buf.extend_from_slice(&PROG);
    let mut backing = vec![0u64; (buf.len() + 7) / 8];
    let base = backing.as_mut_ptr() as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(buf.as_ptr(), base, buf.len());
    }
    let rc = unsafe { entrypoint(base) };
    if rc != 0 {
        *w = snapshot;
        return rc as u32;
    }
    for (k, (i, _, _)) in metas.iter().enumerate() {
        let o = offs[k];
        unsafe {
            let a = &mut w[*i];
            a.owner.copy_from_slice(core::slice::from_raw_parts(base.add(o + 40), 32));
            let mut l = [0u8; 8];
            l.copy_from_slice(core::slice::from_raw_parts(base.add(o + 72), 8));
            a.lamports = u64::from_le_bytes(l);
            l.copy_from_slice(core::slice::from_raw_parts(base.add(o + 80), 8));
            let len = u64::from_le_bytes(l) as usize;
            a.data = core::slice::from_raw_parts(base.add(o + 88), len).to_vec();
        }
    }
    0
}


/// `call`, plus the runtime's lamport rules on success.
fn run(w: &mut Vec<A>, metas: &[M], ix: &[u8]) -> u32 {
    let before: Vec<A> = w.clone();
    let rc = call(w, metas, ix);
    if rc == 0 {
        let sum = |v: &Vec<A>| v.iter().map(|a| a.lamports as u128).sum::<u128>();
        assert_eq!(sum(&before), sum(w), "lamports conserved");
        for (i, a) in w.iter().enumerate() {
            if a.lamports < before[i].lamports {
                let signed_system = before[i].owner == SYS && metas.iter().any(|(j, s, _)| *j == i && *s);
                assert!(before[i].owner == PROG || signed_system, "only the fund or a signing payer may lose lamports");
            }
        }
    }
    rc
}

fn acct(key: [u8; 32], owner: [u8; 32], lamports: u64, len: usize) -> A {
    A { key, owner, lamports, data: vec![0u8; len] }
}

// world indices
const PAYER: usize = 0;
const FUND: usize = 1;
const SYSI: usize = 2;

fn fund_key() -> [u8; 32] {
    host::find_pda(&[FUND_SEED], &PROG).0
}
fn listed(entry: u8, a: u8, b: u8) -> [u8; 32] {
    listed_address(entry, a, b).unwrap().0
}

/// payer, an empty fund address, the system program
fn bare() -> Vec<A> {
    host::reset();
    host::set_clock(1, T0);
    vec![acct([0x50; 32], SYS, RICH, 0), acct(fund_key(), SYS, 0, 0), acct(SYS, SYS, 1, 0)]
}
/// a created fund holding `extra` lamports above its rent
fn world(extra: u64) -> Vec<A> {
    let mut w = bare();
    assert_eq!(run(&mut w, &[(PAYER, true, true), (FUND, false, true), (SYSI, false, false)], &[0]), 0);
    w[FUND].lamports += extra;
    w[PAYER].lamports -= extra;
    w
}
/// adds a listed account owned by `owner` holding rent plus `above`; returns its index
fn add_target(w: &mut Vec<A>, entry: u8, a: u8, b: u8, owner: [u8; 32], len: usize, above: u64) -> usize {
    w.push(acct(listed(entry, a, b), owner, rent_minimum(len) + above, len));
    w.len() - 1
}
fn refill(w: &mut Vec<A>, t: usize, entry: u8, a: u8, b: u8) -> u32 {
    run(w, &[(FUND, false, true), (t, false, true)], &[1, entry, a, b])
}

#[test]
fn the_fund_is_created_once_at_its_own_address() {
    let mut w = bare();
    let m = [(PAYER, true, true), (FUND, false, true), (SYSI, false, false)];
    let mut wrong = w.clone();
    wrong[FUND].key = [0x51; 32];
    assert_eq!(run(&mut wrong, &m, &[0]), ERR_PDA);
    let unsigned = [(PAYER, false, true), (FUND, false, true), (SYSI, false, false)];
    assert_eq!(run(&mut w, &unsigned, &[0]), ERR_SIGNER);
    assert_eq!(run(&mut w, &m, &[0]), 0);
    assert_eq!(w[FUND].owner, PROG);
    assert_eq!(w[FUND].lamports, rent_minimum(FUND_SIZE));
    assert_eq!(&w[FUND].data[..8], MAGIC);
    assert_eq!(w[FUND].data.len(), FUND_SIZE);
    assert_eq!(run(&mut w, &m, &[0]), ERR_EXISTS);
}

#[test]
fn sol_sent_before_the_fund_exists_is_kept() {
    // someone sends SOL to the address first: creating the fund still works and keeps it
    for early in [1u64, rent_minimum(FUND_SIZE) - 1, rent_minimum(FUND_SIZE) + 5_000_000] {
        let mut w = bare();
        w[FUND].lamports = early;
        w[PAYER].lamports -= early;
        let m = [(PAYER, true, true), (FUND, false, true), (SYSI, false, false)];
        assert_eq!(run(&mut w, &m, &[0]), 0);
        assert_eq!(w[FUND].owner, PROG);
        assert_eq!(w[FUND].lamports, early.max(rent_minimum(FUND_SIZE)));
        assert_eq!(&w[FUND].data[..8], MAGIC);
    }
}

#[test]
fn a_low_listed_account_is_topped_up_to_one_hundred_saves() {
    let mut w = world(1_000_000_000);
    let t = add_target(&mut w, 3, 2, 0, PLAY, 10_240, 100_000);
    let fund_before = w[FUND].lamports;
    assert_eq!(refill(&mut w, t, 3, 2, 0), 0);
    assert_eq!(w[t].lamports, rent_minimum(10_240) + TOP_UP);
    let paid = TOP_UP - 100_000;
    assert_eq!(w[FUND].lamports, fund_before - paid);
    let d = &w[FUND].data;
    assert_eq!(rd64(d, F_DAY), (T0 / 86_400) as u64);
    assert_eq!(rd64(d, F_PAID_TODAY), paid);
    assert_eq!(rd64(d, F_PAID_TOTAL), paid);
    assert_eq!(rd32(d, F_REFILLS), 1);
    // now healthy: refused
    assert_eq!(refill(&mut w, t, 3, 2, 0), ERR_NOT_LOW);
}

#[test]
fn only_below_ten_paid_saves() {
    let mut w = world(1_000_000_000);
    let t = add_target(&mut w, 5, 1, 0, STREAKS, 10_176, LOW);
    assert_eq!(refill(&mut w, t, 5, 1, 0), ERR_NOT_LOW, "exactly ten saves left is not low");
    w[t].lamports -= 1;
    assert_eq!(refill(&mut w, t, 5, 1, 0), 0);
    assert_eq!(w[t].lamports, rent_minimum(10_176) + TOP_UP);
}

#[test]
fn only_the_listed_address_and_only_while_on_solana() {
    let mut w = world(1_000_000_000);
    let t = add_target(&mut w, 6, 0, 0, BOARD, 10_208, 0);
    // the entry and seeds must name this very account
    assert_eq!(refill(&mut w, t, 6, 1, 0), ERR_PDA);
    assert_eq!(refill(&mut w, t, 5, 0, 0), ERR_PDA);
    assert_eq!(refill(&mut w, t, 9, 0, 0), ERR_ENTRY);
    assert_eq!(refill(&mut w, t, 255, 0, 0), ERR_ENTRY);
    assert_eq!(refill(&mut w, t, 6, 0, 1), ERR_DATA, "a one-seed entry takes no second index");
    // delegated to a rollup (owned by the delegation program), or never created: refused
    w[t].owner = DELEG;
    assert_eq!(refill(&mut w, t, 6, 0, 0), ERR_OWNER);
    w[t].owner = SYS;
    assert_eq!(refill(&mut w, t, 6, 0, 0), ERR_OWNER);
    // an account at the address but owned by another listed program: refused
    w[t].owner = PLAY;
    assert_eq!(refill(&mut w, t, 6, 0, 0), ERR_OWNER);
    w[t].owner = BOARD;
    assert_eq!(refill(&mut w, t, 6, 0, 0), 0);
}

#[test]
fn every_list_entry_derives_its_programs_address() {
    let cases: [(u8, u8, u8, &[u8; 32], &[&[u8]]); 9] = [
        (0, 3, 0, &REGISTRY, &[b"sabha-reg", &[3]]),
        (1, 2, 7, &REGISTRY, &[b"sabha-reg", &[2], &[7]]),
        (2, 0, 0, &PLAY, &[b"play-reg"]),
        (3, 1, 0, &PLAY, &[b"play-ach", &[1]]),
        (4, 3, 0, &PLAY, &[b"play-rel", &[3]]),
        (5, 2, 0, &STREAKS, &[b"smaran-streaks", &[2]]),
        (6, 1, 0, &BOARD, &[b"mo-board", &[1]]),
        (7, 4, 0, &ETERNAL, &[b"eb-land", &[4]]),
        (8, 1, 0, &ETERNAL, &[b"eb-book", &[1]]),
    ];
    for (entry, a, b, program, seeds) in cases {
        let (key, p) = listed_address(entry, a, b).unwrap();
        assert_eq!(p, program);
        assert_eq!(key, host::find_pda(seeds, program).0, "entry {}", entry);
    }
    assert_eq!(listed_address(2, 1, 0), Err(ERR_DATA), "the register has no index");
}

#[test]
fn at_most_a_tenth_of_a_sol_a_day() {
    let mut w = world(5_000_000_000);
    let per = TOP_UP; // each target is at rent exactly
    let n = (DAILY_CAP / per) as u8; // 10
    let mut ts = Vec::new();
    for i in 0..n + 1 {
        ts.push(add_target(&mut w, 1, 1, i, REGISTRY, 10_144, 0));
    }
    for i in 0..n {
        assert_eq!(refill(&mut w, ts[i as usize], 1, 1, i), 0);
    }
    assert_eq!(rd64(&w[FUND].data, F_PAID_TODAY), DAILY_CAP);
    assert_eq!(refill(&mut w, ts[n as usize], 1, 1, n), ERR_DAILY_CAP);
    // the next UTC day starts again
    host::set_clock(2, (T0 / 86_400 + 1) * 86_400);
    assert_eq!(refill(&mut w, ts[n as usize], 1, 1, n), 0);
    assert_eq!(rd64(&w[FUND].data, F_PAID_TODAY), per);
    assert_eq!(rd64(&w[FUND].data, F_PAID_TOTAL), DAILY_CAP + per);
    assert_eq!(rd32(&w[FUND].data, F_REFILLS), n as u32 + 1);
}

#[test]
fn the_fund_keeps_its_own_rent() {
    let mut w = world(TOP_UP - 1);
    let t = add_target(&mut w, 7, 0, 0, ETERNAL, 5_380, 0);
    assert_eq!(refill(&mut w, t, 7, 0, 0), ERR_FUND_LOW);
    w[FUND].lamports += 1;
    w[PAYER].lamports -= 1;
    assert_eq!(refill(&mut w, t, 7, 0, 0), 0);
    assert_eq!(w[FUND].lamports, rent_minimum(FUND_SIZE));
}

#[test]
fn only_the_real_fund_pays() {
    let mut w = world(1_000_000_000);
    let t = add_target(&mut w, 8, 0, 0, ETERNAL, 9_104, 0);
    // another account owned by this program, or the fund's address held by someone else
    let mut fake = w[FUND].clone();
    fake.key = [0x52; 32];
    w.push(fake);
    let f = w.len() - 1;
    assert_eq!(run(&mut w, &[(f, false, true), (t, false, true)], &[1, 8, 0, 0]), ERR_PDA);
    let mut stolen = w.clone();
    stolen[FUND].owner = SYS;
    assert_eq!(run(&mut stolen, &[(FUND, false, true), (t, false, true)], &[1, 8, 0, 0]), ERR_OWNER);
    // both accounts must be writable
    assert_eq!(run(&mut w, &[(FUND, false, false), (t, false, true)], &[1, 8, 0, 0]), ERR_ACCOUNTS);
    assert_eq!(run(&mut w, &[(FUND, false, true), (t, false, false)], &[1, 8, 0, 0]), ERR_ACCOUNTS);
    assert_eq!(run(&mut w, &[(FUND, false, true), (t, false, true)], &[1, 8, 0]), ERR_ACCOUNTS);
}

#[test]
fn there_is_no_other_way_out() {
    let mut w = world(1_000_000_000);
    let t = add_target(&mut w, 2, 0, 0, PLAY, 512, 0);
    for tag in 2..=255u8 {
        assert_eq!(run(&mut w, &[(FUND, false, true), (t, false, true), (PAYER, true, true)], &[tag, 2, 0, 0]), ERR_TAG);
    }
    assert_eq!(run(&mut w, &[(FUND, false, true)], &[]), ERR_TAG);
    // a refill never pays the caller or anyone but the named account
    let payer_before = w[PAYER].lamports;
    assert_eq!(refill(&mut w, t, 2, 0, 0), 0);
    assert_eq!(w[PAYER].lamports, payer_before);
    assert!(host::cpis().is_empty(), "a refill makes no cross-program call");
}
