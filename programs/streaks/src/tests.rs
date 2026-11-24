//! Host harness for Smaran Streaks. LOCAL TESTS ONLY (classification: PASSED LOCALLY).
//!
//! The real `entrypoint` runs on accounts serialized in the loader's input format; `host.rs` stands in for the runtime
//! (fake PDA derivation, recorded cross-program calls, a settable clock). Delegation, saves and the rollup clock are
//! verified live, not here.
use super::*;
use crate::host;

const PROG: [u8; 32] = [0x77; 32];
const SYS: [u8; 32] = [0u8; 32];
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

// account indices in the test world
const OPI: usize = 0;
const GROUP0: usize = 1; // groups at 1..=4
const SYSI: usize = 5;
const P: usize = 6;
const Q: usize = 7;
const MAGICP: usize = 8;
const MAGICC: usize = 9;
const VAULT: usize = 10;
const REG_BASE: usize = 11; // registry groups 0..3 at 11..=14
const STRANGER: usize = 15;

fn acct(key: [u8; 32], owner: [u8; 32], lamports: u64) -> A {
    A { key, owner, lamports, data: Vec::new() }
}
fn pk(n: u8) -> [u8; 32] {
    [n; 32]
}
fn name_hash(name: &str) -> [u8; 32] {
    let mut h = [0u8; 32];
    host::sha256(&[NAME_DOMAIN, name.as_bytes()], &mut h);
    h
}
/// The registry group index a name lands in at step 0.
fn reg_index(name: &str) -> usize {
    (name_hash(name)[0] % 4) as usize
}

/// Builds the world: operator, 4 initialised groups, two players with registered names, magic accounts.
fn world(reserve: u64) -> Vec<A> {
    host::reset();
    host::set_clock(1, T0);
    let mut w = vec![
        acct(OPERATOR, SYS, RICH),
        acct(host::find_pda(&[SEED, &[0]], &PROG).0, SYS, 0),
        acct(host::find_pda(&[SEED, &[1]], &PROG).0, SYS, 0),
        acct(host::find_pda(&[SEED, &[2]], &PROG).0, SYS, 0),
        acct(host::find_pda(&[SEED, &[3]], &PROG).0, SYS, 0),
        acct(SYSTEM_PROGRAM, SYS, 1),
        acct(pk(0x50), SYS, 0),
        acct(pk(0x51), SYS, 0),
        acct(MAGIC_PROGRAM, SYS, 1),
        acct(MAGIC_CONTEXT, MAGIC_PROGRAM, 1),
        acct(host::find_pda(&[b"magic-fee-vault", &VALIDATOR], &DELEGATION_PROGRAM).0, DELEGATION_PROGRAM, 1),
    ];
    for i in 0..4u8 {
        let mut a = acct(host::find_pda(&[REG_SEED, &[i]], &REGISTRY).0, REGISTRY, 1);
        a.data = vec![0u8; REG_SIZE];
        a.data[..8].copy_from_slice(REG_MAGIC);
        a.data[8] = i;
        w.push(a);
    }
    w.push(acct(pk(0x52), SYS, 0));
    let lamports = rent_minimum(GROUP_SIZE) + reserve;
    for i in 0..4u8 {
        let mut ix = vec![0u8, i];
        ix.extend_from_slice(&lamports.to_le_bytes());
        assert_eq!(call(&mut w, &[(OPI, true, true), (GROUP0 + i as usize, false, true), (SYSI, false, false)], &ix), 0);
    }
    register(&mut w, "alice", P);
    register(&mut w, "bob_2", Q);
    w
}

/// Puts `name` for player account `who` into its registry group, at the slot the registry would use.
fn register(w: &mut Vec<A>, name: &str, who: usize) {
    let h = name_hash(name);
    let owner = w[who].key;
    let d = &mut w[REG_BASE + reg_index(name)].data;
    let initial = (h[1] as usize | (h[2] as usize) << 8) % REG_SLOTS;
    for k in 0..REG_SLOTS {
        let e = REG_HEADER + ((initial + k) % REG_SLOTS) * REG_ENTRY;
        if d[e] == 0 {
            d[e] = 1;
            d[e + 8..e + 40].copy_from_slice(&h);
            d[e + 40..e + 72].copy_from_slice(&owner);
            return;
        }
    }
    panic!("registry group full");
}

fn checkin_ix(slot: u8, kind: u8, offset: i16, name: &str) -> Vec<u8> {
    let mut ix = vec![2u8, slot, kind];
    ix.extend_from_slice(&offset.to_le_bytes());
    ix.push(name.len() as u8);
    ix.extend_from_slice(name.as_bytes());
    ix
}
fn group_metas() -> Vec<M> {
    (0..4).map(|i| (GROUP0 + i, false, true)).collect()
}
fn checkin_with(w: &mut Vec<A>, who: usize, reg: usize, ix: &[u8], save: bool) -> u32 {
    let mut m: Vec<M> = vec![(who, true, false)];
    m.extend(group_metas());
    m.push((reg, false, false));
    if save {
        m.extend([(MAGICP, false, false), (MAGICC, false, true), (VAULT, false, true)]);
    }
    call(w, &m, ix)
}
fn checkin(w: &mut Vec<A>, who: usize, name: &str, slot: u8, kind: u8, offset: i16) -> u32 {
    let reg = REG_BASE + reg_index(name);
    checkin_with(w, who, reg, &checkin_ix(slot, kind, offset, name), false)
}
fn simple(w: &mut Vec<A>, who: usize, ix: &[u8]) -> u32 {
    let mut m: Vec<M> = vec![(who, true, false)];
    m.extend(group_metas());
    call(w, &m, ix)
}

/// (group, record offset) of a player's record.
fn record(w: &[A], who: usize) -> Option<(usize, usize)> {
    for g in 0..4 {
        let d = &w[GROUP0 + g].data;
        for r in 0..RECS {
            let o = HEADER + r * REC;
            if d[o + R_FLAGS] & 1 == 1 && d[o..o + 32] == w[who].key {
                return Some((g, o));
            }
        }
    }
    None
}
/// (kind, streak, best, total, last) of one habit.
fn habit(w: &[A], who: usize, slot: usize) -> (u8, u16, u16, u32, u32) {
    let (g, o) = record(w, who).expect("record");
    let d = &w[GROUP0 + g].data;
    let h = o + R_HABITS + slot * HABIT;
    (d[h], rd16(d, h + A_STREAK), rd16(d, h + A_BEST), rd32(d, h + A_TOTAL), rd32(d, h + A_LAST))
}
fn magic_calls() -> usize {
    host::cpis().iter().filter(|c| c.program == MAGIC_PROGRAM).count()
}
fn at(t: i64) {
    host::set_clock(1, t);
}

#[test]
fn day_numbers_follow_the_time_zone() {
    assert_eq!(day_of(T0, 0), 20_000);
    assert_eq!(day_of(20_000 * 86_400 - 1, 0), 19_999);
    // 18:30 UTC is already the next day in India (UTC+5:30), still the same day in New York (UTC-5)
    let t = 20_000 * 86_400 + 18 * 3_600 + 30 * 60;
    assert_eq!(day_of(t, 330), 20_001);
    assert_eq!(day_of(t, -300), 20_000);
    assert_eq!(day_of(0, -720), -1);
}

#[test]
fn offsets_and_names_are_checked() {
    assert!(valid_offset(0) && valid_offset(330) && valid_offset(-720) && valid_offset(840) && valid_offset(345));
    assert!(!valid_offset(841) && !valid_offset(-735) && !valid_offset(7));
    assert!(valid_name(b"chaitu") && valid_name(b"bob_2") && valid_name(b"a-b") && valid_name(b"9lives"));
    assert!(!valid_name(b"ab") && !valid_name(b"Chaitu") && !valid_name(b"_x1") && !valid_name(b"a b") && !valid_name(&[b'a'; 33]));
}

#[test]
fn a_first_check_in_makes_a_record_with_the_registry_name() {
    let mut w = world(1_000_000);
    assert_eq!(checkin(&mut w, P, "alice", 0, 3, 330), 0);
    let (g, o) = record(&w, P).unwrap();
    let d = &w[GROUP0 + g].data;
    assert_eq!(&d[o + R_NAME + 1..o + R_NAME + 1 + d[o + R_NAME] as usize], b"alice");
    assert_eq!(rd16(d, o + R_OFFSET) as i16, 330);
    assert_eq!(rd32(d, o + R_JOINED) as i64, T0);
    assert_eq!(rd16(d, H_USED), 1);
    assert_eq!(habit(&w, P, 0), (3, 1, 1, 1, T0 as u32));
}

#[test]
fn one_check_in_per_day_and_streaks_count_consecutive_days() {
    let mut w = world(1_000_000);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 0), 0);
    at(T0 + 3_600);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 0), ERR_TODAY);
    at(T0 + DAY);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 0), 0);
    at(T0 + 2 * DAY);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 0), 0);
    assert_eq!(habit(&w, P, 0).1, 3);
    // a missed day starts again at 1 and keeps the best
    at(T0 + 4 * DAY);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 0), 0);
    let (_, streak, best, total, _) = habit(&w, P, 0);
    assert_eq!((streak, best, total), (1, 3, 4));
}

#[test]
fn the_day_turns_at_local_midnight() {
    let mut w = world(1_000_000);
    // India: 23:00 and 00:30 local are an hour and a half apart but two days
    let late = 20_000 * 86_400 + 17 * 3_600 + 30 * 60; // 23:00 IST
    at(late);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 330), 0);
    at(late + 90 * 60);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 330), 0);
    assert_eq!(habit(&w, P, 0).1, 2);
    // the same two moments in UTC are one day
    at(late);
    assert_eq!(checkin(&mut w, Q, "bob_2", 0, 1, 0), 0);
    at(late + 90 * 60);
    assert_eq!(checkin(&mut w, Q, "bob_2", 0, 1, 0), ERR_TODAY);
}

#[test]
fn the_time_zone_is_fixed_at_the_start() {
    let mut w = world(1_000_000);
    let late = 20_000 * 86_400 + 23 * 3_600; // 23:00 UTC
    at(late);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 0), 0);
    // claiming a time zone 14 hours east now would make it the next day there: ignored, the record keeps UTC
    at(late + 60);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 840), ERR_TODAY);
    let (g, o) = record(&w, P).unwrap();
    assert_eq!(rd16(&w[GROUP0 + g].data, o + R_OFFSET) as i16, 0);
}

#[test]
fn a_bad_time_zone_is_refused_at_the_start() {
    let mut w = world(1_000_000);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 7), ERR_OFFSET);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 900), ERR_OFFSET);
    assert!(record(&w, P).is_none());
}

#[test]
fn only_the_names_owner_can_start_a_record() {
    let mut w = world(1_000_000);
    // Q signs with alice's name: the registry entry belongs to P
    assert_eq!(checkin_with(&mut w, Q, REG_BASE + reg_index("alice"), &checkin_ix(0, 1, 0, "alice"), false), ERR_NOT_REGISTERED);
    // an unregistered name
    let n = "nobody";
    assert_eq!(checkin_with(&mut w, STRANGER, REG_BASE + reg_index(n), &checkin_ix(0, 1, 0, n), false), ERR_NOT_REGISTERED);
    // the right name but the wrong registry group
    let wrong = REG_BASE + (reg_index("alice") + 1) % 4;
    assert_eq!(checkin_with(&mut w, P, wrong, &checkin_ix(0, 1, 0, "alice"), false), ERR_NOT_REGISTERED);
    // a fake registry account not owned by the registry program
    w[REG_BASE + reg_index("alice")].owner = PROG;
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 0), ERR_NOT_REGISTERED);
    assert!(record(&w, P).is_none() && record(&w, Q).is_none());
}

#[test]
fn the_player_must_sign_and_groups_must_be_in_order() {
    let mut w = world(1_000_000);
    let reg = REG_BASE + reg_index("alice");
    let ix = checkin_ix(0, 1, 0, "alice");
    let mut m: Vec<M> = vec![(P, false, false)];
    m.extend(group_metas());
    m.push((reg, false, false));
    assert_eq!(call(&mut w, &m, &ix), ERR_SIGNER);
    let mut m: Vec<M> = vec![(P, true, false), (GROUP0 + 1, false, true), (GROUP0, false, true), (GROUP0 + 2, false, true), (GROUP0 + 3, false, true), (reg, false, false)];
    assert_eq!(call(&mut w, &m, &ix), ERR_PDA);
    m[1] = (GROUP0, false, false);
    m[2] = (GROUP0 + 1, false, true);
    assert_eq!(call(&mut w, &m, &ix), ERR_OWNER);
}

#[test]
fn habits_are_separate_and_a_slot_keeps_its_kind() {
    let mut w = world(1_000_000);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 0), 0);
    assert_eq!(checkin(&mut w, P, "alice", 1, 4, 0), 0); // a second habit, same day
    assert_eq!(checkin(&mut w, P, "alice", 0, 2, 0), ERR_HABIT); // slot 0 keeps kind 1
    assert_eq!(checkin(&mut w, P, "alice", 2, 1, 0), ERR_HABIT); // kind 1 already has a slot
    assert_eq!(checkin(&mut w, P, "alice", 3, 5, 0), ERR_DATA);
    assert_eq!(checkin(&mut w, P, "alice", 2, 17, 0), ERR_DATA);
    assert_eq!(checkin(&mut w, P, "alice", 2, 0, 0), ERR_DATA);
    assert_eq!((habit(&w, P, 0).0, habit(&w, P, 1).0, habit(&w, P, 2).0), (1, 4, 0));
}

#[test]
fn set_habit_starts_a_slot_again_and_can_remove_it() {
    let mut w = world(1_000_000);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 0), 0);
    at(T0 + DAY);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 0), 0);
    assert_eq!(simple(&mut w, P, &[3, 0, 6]), 0);
    assert_eq!(habit(&w, P, 0), (6, 0, 0, 0, 0));
    assert_eq!(checkin(&mut w, P, "alice", 0, 6, 0), 0);
    assert_eq!(habit(&w, P, 0).1, 1);
    assert_eq!(simple(&mut w, P, &[3, 1, 6]), ERR_HABIT); // kind 6 is in slot 0
    assert_eq!(simple(&mut w, P, &[3, 0, 0]), 0);
    assert_eq!(habit(&w, P, 0).0, 0);
    assert_eq!(simple(&mut w, Q, &[3, 0, 2]), ERR_NO_RECORD);
}

#[test]
fn forget_removes_only_the_signers_record() {
    let mut w = world(1_000_000);
    assert_eq!(checkin(&mut w, P, "alice", 0, 1, 0), 0);
    assert_eq!(checkin(&mut w, Q, "bob_2", 0, 1, 0), 0);
    assert_eq!(simple(&mut w, P, &[5]), 0);
    assert!(record(&w, P).is_none() && record(&w, Q).is_some());
    assert_eq!(rd16(&w[GROUP0].data, H_USED) + rd16(&w[GROUP0 + 1].data, H_USED), 1);
    assert_eq!(simple(&mut w, P, &[5]), ERR_NO_RECORD);
    // starting again later is a fresh record
    at(T0 + 3_600);
    assert_eq!(checkin(&mut w, P, "alice", 0, 2, 0), 0);
    assert_eq!(habit(&w, P, 0).1, 1);
}

#[test]
fn check_ins_save_the_group_at_most_once_an_hour() {
    let mut w = world(1_000_000);
    let reg = REG_BASE + reg_index("alice");
    assert_eq!(checkin_with(&mut w, P, reg, &checkin_ix(0, 1, 0, "alice"), true), 0);
    assert_eq!(magic_calls(), 1);
    let call0 = host::cpis().into_iter().find(|c| c.program == MAGIC_PROGRAM).unwrap();
    assert_eq!(call0.data, vec![1, 0, 0, 0]);
    assert_eq!(call0.metas[0], (w[GROUP0].key, true, true)); // the group pays and signs
    assert_eq!(rd32(&w[GROUP0].data, H_SAVED) as i64, T0);
    assert_eq!(w[GROUP0].data[H_UNSAVED], 0);
    // a second habit within the hour: no save, the change is marked unsaved
    at(T0 + 600);
    assert_eq!(checkin_with(&mut w, P, reg, &checkin_ix(1, 2, 0, "alice"), true), 0);
    assert_eq!(magic_calls(), 0);
    assert_eq!(w[GROUP0].data[H_UNSAVED], 1);
    // anyone may save it, but not within the hour
    let save = |w: &mut Vec<A>, who: usize| call(w, &[(who, true, false), (GROUP0, false, true), (MAGICP, false, false), (MAGICC, false, true), (VAULT, false, true)], &[4, 0]);
    assert_eq!(save(&mut w, Q), ERR_TOO_SOON);
    at(T0 + 3_600);
    assert_eq!(save(&mut w, Q), 0);
    assert_eq!(magic_calls(), 1);
    at(T0 + 9_000);
    assert_eq!(save(&mut w, Q), ERR_UNCHANGED);
    assert_eq!(save(&mut w, OPI), 0); // the operator may always save
}

#[test]
fn a_low_reserve_skips_the_save_but_keeps_the_check_in() {
    let mut w = world(0);
    let reg = REG_BASE + reg_index("alice");
    assert_eq!(checkin_with(&mut w, P, reg, &checkin_ix(0, 1, 0, "alice"), true), 0);
    assert_eq!(magic_calls(), 0);
    assert_eq!(habit(&w, P, 0).1, 1);
    at(T0 + 7_200);
    let r = call(&mut w, &[(Q, true, false), (GROUP0, false, true), (MAGICP, false, false), (MAGICC, false, true), (VAULT, false, true)], &[4, 0]);
    assert_eq!(r, ERR_RESERVE);
}

#[test]
fn a_wrong_fee_vault_is_refused() {
    let mut w = world(1_000_000);
    w[VAULT].key = pk(0x99);
    let reg = REG_BASE + reg_index("alice");
    assert_eq!(checkin_with(&mut w, P, reg, &checkin_ix(0, 1, 0, "alice"), true), ERR_ACCOUNTS);
    assert!(record(&w, P).is_none());
}

#[test]
fn groups_fill_in_order_and_report_full() {
    let mut w = world(1_000_000);
    let mut gs: Vec<Vec<u8>> = (0..4).map(|g| w[GROUP0 + g].data.clone()).collect();
    for t in gs.iter_mut() {
        for r in 0..RECS {
            t[HEADER + r * REC + R_FLAGS] = 1;
        }
        wr16(t, H_USED, RECS as u16);
    }
    {
        let [a, b, c, d] = &mut gs[..] else { unreachable!() };
        let arr: [&mut [u8]; 4] = [a, b, c, d];
        assert_eq!(free(&arr), None);
    }
    gs[2][HEADER + 5 * REC + R_FLAGS] = 0;
    wr16(&mut gs[2], H_USED, RECS as u16 - 1);
    let [a, b, c, d] = &mut gs[..] else { unreachable!() };
    let arr: [&mut [u8]; 4] = [a, b, c, d];
    assert_eq!(free(&arr), Some((2, 5)));
    let _ = &mut w;
}

#[test]
fn only_the_operator_creates_groups() {
    host::reset();
    host::set_clock(1, T0);
    let mut w = vec![acct(pk(0x60), SYS, RICH), acct(host::find_pda(&[SEED, &[0]], &PROG).0, SYS, 0), acct(SYSTEM_PROGRAM, SYS, 1)];
    let mut ix = vec![0u8, 0];
    ix.extend_from_slice(&(rent_minimum(GROUP_SIZE) + 1).to_le_bytes());
    assert_eq!(call(&mut w, &[(0, true, true), (1, false, true), (2, false, false)], &ix), ERR_OPERATOR);
}

#[test]
fn streaks_cap_instead_of_wrapping() {
    let mut h = [0u8; HABIT];
    wr16(&mut h, A_STREAK, u16::MAX);
    wr16(&mut h, A_BEST, u16::MAX);
    wr32(&mut h, A_LAST, (T0 - DAY) as u32);
    assert_eq!(check_in(&mut h, T0, 0), Ok(()));
    assert_eq!(rd16(&h, A_STREAK), u16::MAX);
}
