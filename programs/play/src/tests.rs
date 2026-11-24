//! Host harness for the PLAY program. LOCAL TESTS ONLY (classification: PASSED LOCALLY).
//!
//! The real `entrypoint` is called with accounts serialized in the loader's input format. The runtime is stood in for by
//! `host.rs`: fake PDA derivation, recorded cross-program calls (system create/assign applied to the accounts), a settable
//! clock. A failed call restores every account, as the real runtime does. Nothing here proves behaviour on a live
//! validator: delegation, commit, the VRF callback shape and rollup concurrency are PENDING LIVE VERIFICATION.
use super::*;
use crate::host;
use std::collections::HashMap;

const PROG: [u8; 32] = [0x77; 32];
const GAME: [u8; 32] = [0x66; 32];
const OP: [u8; 32] = [0x0e; 32];
const ATTACKER: [u8; 32] = [0x99; 32];
const SYS: [u8; 32] = [0u8; 32];
const RICH: u64 = 200_000_000;
const STD: [(u8, u8, u16); 4] = [(1, 0, 6000), (2, 1, 3000), (3, 2, 900), (4, 3, 100)];

#[derive(Clone)]
struct A {
    key: [u8; 32],
    owner: [u8; 32],
    lamports: u64,
    data: Vec<u8>,
    exec: bool,
}
/// (account index, signer, writable)
type M = (usize, bool, bool);

fn pk(n: u16) -> [u8; 32] {
    let mut k = [0xAA; 32];
    k[0] = n as u8;
    k[1] = (n >> 8) as u8;
    k
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

/// Serializes the accounts like the loader, runs `entrypoint`, copies results back. Restores everything on failure.
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
        buf.push(a.exec as u8);
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

fn ledger(kind: u8, idx: u8) -> Vec<u8> {
    let size = if kind == KIND_REG { REG_SIZE } else { SHARD_SIZE };
    let mut d = vec![0u8; size];
    d[..8].copy_from_slice(MAGIC_LEDGER);
    d[H_KIND] = kind;
    d[H_VERSION] = VERSION;
    d[H_IDX] = idx;
    d
}

struct Fx {
    w: Vec<A>,
    players: HashMap<u16, usize>,
    op: usize,
    reg: usize,
    ach: [usize; 4],
    rel: [usize; 4],
    magic: usize,
    ctx: usize,
    vault: usize,
    ident: usize,
    vrf_ident: usize,
    vrf: usize,
    queue: usize,
    sys: usize,
    slothash: usize,
    deleg: usize,
    prog: usize,
    tables: Vec<usize>,
}

impl Fx {
    fn add(&mut self, key: [u8; 32], owner: [u8; 32], lamports: u64, data: Vec<u8>, exec: bool) -> usize {
        self.w.push(A { key, owner, lamports, data, exec });
        self.w.len() - 1
    }

    fn build(initialised: bool) -> Fx {
        host::reset();
        let mut fx = Fx {
            w: Vec::new(),
            players: HashMap::new(),
            op: 0,
            reg: 0,
            ach: [0; 4],
            rel: [0; 4],
            magic: 0,
            ctx: 0,
            vault: 0,
            ident: 0,
            vrf_ident: 0,
            vrf: 0,
            queue: 0,
            sys: 0,
            slothash: 0,
            deleg: 0,
            prog: 0,
            tables: Vec::new(),
        };
        let (own, lam) = if initialised { (PROG, RICH) } else { (SYS, 0) };
        let body = |kind: u8, idx: u8| if initialised { ledger(kind, idx) } else { Vec::new() };
        fx.op = fx.add(OP, SYS, 10_000_000_000, vec![], false);
        fx.reg = fx.add(ledger_address(KIND_REG, 0, &PROG).0, own, lam, body(KIND_REG, 0), false);
        for i in 0..4u8 {
            fx.ach[i as usize] = fx.add(ledger_address(KIND_ACH, i, &PROG).0, own, lam, body(KIND_ACH, i), false);
        }
        for i in 0..4u8 {
            fx.rel[i as usize] = fx.add(ledger_address(KIND_REL, i, &PROG).0, own, lam, body(KIND_REL, i), false);
        }
        fx.magic = fx.add(MAGIC_PROGRAM, SYS, 1, vec![], true);
        fx.ctx = fx.add(MAGIC_CONTEXT, MAGIC_PROGRAM, 1, vec![], false);
        let vault = find_pda(&[b"magic-fee-vault", &VALIDATOR], &DELEGATION_PROGRAM).0;
        fx.vault = fx.add(vault, SYS, 1, vec![], false);
        fx.ident = fx.add(find_pda(&[IDENTITY_SEED], &PROG).0, SYS, 1, vec![], false);
        fx.vrf_ident = fx.add(find_pda(&[IDENTITY_SEED, &PROG], &VRF_PROGRAM).0, SYS, 1, vec![], false);
        fx.vrf = fx.add(VRF_PROGRAM, SYS, 1, vec![], true);
        fx.queue = fx.add(VRF_QUEUE, VRF_PROGRAM, 1, vec![], false);
        fx.sys = fx.add(SYSTEM_PROGRAM, SYS, 1, vec![], true);
        fx.slothash = fx.add(SLOT_HASHES, SYS, 1, vec![], false);
        fx.deleg = fx.add(DELEGATION_PROGRAM, SYS, 1, vec![], true);
        fx.prog = fx.add(PROG, SYS, 1, vec![], true);
        for i in 0..T_MAX_IDX {
            let mut d = vec![0u8; T_SIZE];
            d[..8].copy_from_slice(T_MAGIC);
            let t = fx.add(find_pda(&[T_SEED, &[i]], &GAME).0, GAME, 1_000_000, d, false);
            fx.tables.push(t);
        }
        fx
    }
    /// Initialised ledger accounts, nothing registered yet.
    fn new() -> Fx {
        Fx::build(true)
    }
    /// Uninitialised (empty, system owned) ledger accounts, for init and delegation tests.
    fn bare() -> Fx {
        Fx::build(false)
    }
    /// Game registered, odds set and frozen.
    fn ready() -> Fx {
        let mut fx = Fx::new();
        assert_eq!(fx.register(&GAME_MAGIC_OVER, &GAME, 1), 0);
        assert_eq!(fx.set_odds(&STD), 0);
        assert_eq!(fx.freeze(), 0);
        fx
    }
    fn player(&mut self, n: u16) -> usize {
        if let Some(i) = self.players.get(&n) {
            return *i;
        }
        let i = self.add(pk(n), SYS, 0, vec![], false);
        self.players.insert(n, i);
        i
    }

    // ---- instructions
    fn register(&mut self, id: &[u8; 16], prog: &[u8; 32], enabled: u8) -> u32 {
        let mut ix = vec![4];
        ix.extend_from_slice(id);
        ix.extend_from_slice(prog);
        ix.push(enabled);
        call(&mut self.w, &[(self.op, true, false), (self.reg, false, true)], &ix)
    }
    fn set_odds(&mut self, odds: &[(u8, u8, u16)]) -> u32 {
        let mut ix = vec![5, odds.len() as u8];
        for (k, r, wt) in odds {
            ix.push(*k);
            ix.push(*r);
            ix.extend_from_slice(&wt.to_le_bytes());
        }
        call(&mut self.w, &[(self.op, true, false), (self.reg, false, true)], &ix)
    }
    fn freeze(&mut self) -> u32 {
        call(&mut self.w, &[(self.op, true, false), (self.reg, false, true)], &[6])
    }
    fn claim_metas(&self, player: usize, tbl: usize) -> Vec<M> {
        vec![
            (player, true, false),
            (self.reg, false, false),
            (self.ach[0], false, true),
            (self.ach[1], false, true),
            (self.ach[2], false, true),
            (self.ach[3], false, true),
            (tbl, false, false),
            (self.magic, false, false),
            (self.ctx, false, true),
            (self.vault, false, true),
        ]
    }
    fn claim_ix(ach: u16, tidx: u8) -> Vec<u8> {
        let mut ix = vec![7];
        ix.extend_from_slice(&ach.to_le_bytes());
        ix.push(tidx);
        ix
    }
    fn claim(&mut self, player: usize, tidx: u8) -> u32 {
        let m = self.claim_metas(player, self.tables[tidx as usize]);
        call(&mut self.w, &m, &Fx::claim_ix(ACH_SIX_HITTER, tidx))
    }
    fn request_metas(&self, player: usize, shard: usize) -> Vec<M> {
        vec![
            (player, true, true),
            (self.ach[shard], false, true),
            (self.ident, false, false),
            (self.vrf, false, false),
            (self.queue, false, true),
            (self.sys, false, false),
            (self.slothash, false, false),
            (self.reg, false, false),
        ]
    }
    fn request(&mut self, player: usize, shard: usize, slot: u16) -> u32 {
        let m = self.request_metas(player, shard);
        let mut ix = vec![8];
        ix.extend_from_slice(&slot.to_le_bytes());
        call(&mut self.w, &m, &ix)
    }
    fn callback_ix(rnd: &[u8; 32], shard: u8, slot: u16, seq: u32) -> Vec<u8> {
        let mut ix = VRF_CALLBACK.to_vec();
        ix.extend_from_slice(rnd);
        ix.push(shard);
        ix.extend_from_slice(&slot.to_le_bytes());
        ix.extend_from_slice(&seq.to_le_bytes());
        ix
    }
    fn callback_metas(&self, signer: bool, ach_shard: usize) -> Vec<M> {
        vec![
            (self.vrf_ident, signer, false),
            (self.ach[ach_shard], false, true),
            (self.reg, false, false),
            (self.rel[0], false, true),
            (self.rel[1], false, true),
            (self.rel[2], false, true),
            (self.rel[3], false, true),
        ]
    }
    fn callback(&mut self, shard: u8, slot: u16, seq: u32, rnd: &[u8; 32]) -> u32 {
        let m = self.callback_metas(true, shard as usize);
        call(&mut self.w, &m, &Fx::callback_ix(rnd, shard, slot, seq))
    }
    fn transfer(&mut self, owner: usize, rshard: usize, slot: u16, new_owner: &[u8; 32]) -> u32 {
        let mut ix = vec![10];
        ix.extend_from_slice(&slot.to_le_bytes());
        ix.extend_from_slice(new_owner);
        let m = [(owner, true, false), (self.rel[rshard], false, true), (self.magic, false, false), (self.ctx, false, true), (self.vault, false, true)];
        call(&mut self.w, &m, &ix)
    }
    fn commit(&mut self, payer: usize, acct: usize) -> u32 {
        let m = [(payer, true, true), (acct, false, true), (self.magic, false, false), (self.ctx, false, true), (self.vault, false, true)];
        call(&mut self.w, &m, &[2])
    }

    // ---- fixtures and readers
    #[allow(clippy::too_many_arguments)]
    fn set_table(&mut self, tidx: u8, phase: u8, winner: u8, a: [u8; 32], b: [u8; 32], matchno: u32, totals_on: u8, sixes: (u16, u16)) {
        let d = &mut self.w[self.tables[tidx as usize]].data;
        d[..8].copy_from_slice(T_MAGIC);
        d[T_PHASE] = phase;
        d[T_WINNER] = winner;
        d[T_A..T_A + 32].copy_from_slice(&a);
        d[T_B..T_B + 32].copy_from_slice(&b);
        wr32(d, T_MATCH, matchno);
        d[T_TOTALS_ON] = totals_on;
        wr16(d, T_TOTALS + 4, sixes.0);
        wr16(d, T_TOTALS + 8 + 4, sixes.1);
    }
    /// Player `p` (side A) wins match `matchno` at table `tidx` with two sixes against the AI side.
    fn win(&mut self, p: u16, tidx: u8, matchno: u32) {
        self.set_table(tidx, T_FINISHED, 1, pk(p), [0; 32], matchno, 1, (2, 0));
    }
    fn ach_count(&self) -> usize {
        (0..4).map(|i| (0..ACH_PER).filter(|s| self.w[self.ach[i]].data[HEADER + s * ACH_SLOT] == 1).count()).sum()
    }
    fn ach_slot(&self, shard: usize, slot: usize) -> Vec<u8> {
        self.w[self.ach[shard]].data[HEADER + slot * ACH_SLOT..HEADER + (slot + 1) * ACH_SLOT].to_vec()
    }
    fn rel_slot(&self, shard: usize, slot: usize) -> Vec<u8> {
        self.w[self.rel[shard]].data[HEADER + slot * REL_SLOT..HEADER + (slot + 1) * REL_SLOT].to_vec()
    }
    fn find_ach(&self, p: u16) -> Option<(usize, usize)> {
        let key = pk(p);
        for i in 0..4 {
            for s in 0..ACH_PER {
                let o = HEADER + s * ACH_SLOT;
                let d = &self.w[self.ach[i]].data;
                if d[o] == 1 && d[o + A_PLAYER..o + A_PLAYER + 32] == key {
                    return Some((i, s));
                }
            }
        }
        None
    }
    fn find_relic(&self, id: &[u8; 32]) -> Option<(usize, usize)> {
        for i in 0..4 {
            for s in 0..REL_PER {
                let o = HEADER + s * REL_SLOT;
                let d = &self.w[self.rel[i]].data;
                if d[o] == 1 && d[o + L_ID..o + L_ID + 32] == *id {
                    return Some((i, s));
                }
            }
        }
        None
    }
    /// Full happy path for player `p`: win, claim, request, VRF callback with `rnd`. Returns where the relic landed.
    fn earn(&mut self, p: u16, tidx: u8, matchno: u32, rnd: &[u8; 32]) -> (usize, usize) {
        self.win(p, tidx, matchno);
        let pl = self.player(p);
        assert_eq!(self.claim(pl, tidx), 0, "claim");
        let (sh, sl) = self.find_ach(p).unwrap();
        assert_eq!(self.request(pl, sh, sl as u16), 0, "request");
        let seq = rd32(&self.ach_slot(sh, sl), A_REQ_SEQ);
        assert_eq!(self.callback(sh as u8, sl as u16, seq, rnd), 0, "callback");
        self.find_relic(rnd).expect("relic placed")
    }
}

/// 32 random bytes whose first u16 (little endian) is `x`; `tag` makes the rest distinct.
fn rnd_for(x: u16, tag: u8) -> [u8; 32] {
    let mut r = [tag; 32];
    r[..2].copy_from_slice(&x.to_le_bytes());
    r
}

fn reg_with(odds: &[(u8, u8, u16)]) -> Vec<u8> {
    let mut reg = ledger(KIND_REG, 0);
    reg[R_NKINDS] = odds.len() as u8;
    for (i, (k, r, w)) in odds.iter().enumerate() {
        let o = R_KINDS + i * KIND_ENTRY;
        reg[o] = *k;
        reg[o + 1] = *r;
        wr16(&mut reg, o + 2, *w);
    }
    reg
}

fn leak_acc(key: [u8; 32], signer: bool) -> Acc {
    let key = Box::leak(Box::new(key));
    let owner = Box::leak(Box::new([0u8; 32]));
    let lamports = Box::leak(Box::new(0u64));
    let data_len = Box::leak(Box::new(0u64));
    Acc { key, owner, lamports, data_len, data: core::ptr::null_mut(), rent_epoch: 0, is_signer: signer, is_writable: false, executable: false }
}

// ================================================================ harness self-checks
#[test]
fn host_sha256_matches_known_vectors() {
    let mut o = [0u8; 32];
    host::sha256(&[b"abc"], &mut o);
    assert_eq!(hex(&o), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    host::sha256(&[], &mut o);
    assert_eq!(hex(&o), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    host::sha256(&[b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"], &mut o);
    assert_eq!(hex(&o), "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
    let mut p = [0u8; 32];
    host::sha256(&[b"a", b"bc"], &mut p);
    host::sha256(&[b"abc"], &mut o);
    assert_eq!(p, o);
}

#[test]
fn harness_round_trips_account_state() {
    let mut fx = Fx::new();
    let before = fx.w[fx.reg].data.clone();
    // a successful no-op style call must leave the registry readable and unchanged except what it edits
    assert_eq!(fx.register(&GAME_MAGIC_OVER, &GAME, 1), 0);
    let after = &fx.w[fx.reg].data;
    assert_eq!(before.len(), after.len());
    assert_eq!(after[R_NGAMES], 1);
    assert_eq!(&after[R_GAMES..R_GAMES + 16], &GAME_MAGIC_OVER);
}

// ================================================================ AT7: the draw
#[test]
fn draw_is_exactly_uniform_over_the_accepted_range() {
    let reg = reg_with(&STD);
    let mut counts = [0u32; 5];
    let mut per_residue = vec![0u32; 10_000];
    for x in 0..DRAW_LIMIT as u32 {
        let r = rnd_for(x as u16, 0);
        let (k, _) = draw(&reg, &r).unwrap();
        counts[k as usize] += 1;
        per_residue[(x % 10_000) as usize] += 1;
    }
    assert!(per_residue.iter().all(|c| *c == 6), "every residue has exactly six sources");
    assert_eq!(&counts[1..], &[36_000, 18_000, 5_400, 600]);
}

#[test]
fn draw_boundaries_and_rarity() {
    let reg = reg_with(&STD);
    for (v, kind, rarity) in [(0u16, 1u8, 0u8), (5999, 1, 0), (6000, 2, 1), (8999, 2, 1), (9000, 3, 2), (9899, 3, 2), (9900, 4, 3), (9999, 4, 3)] {
        assert_eq!(draw(&reg, &rnd_for(v, 0)), Ok((kind, rarity)), "v={}", v);
    }
    // 10_000 + 5_999 is the same residue as 5_999
    assert_eq!(draw(&reg, &rnd_for(15_999, 0)), Ok((1, 0)));
}

#[test]
fn draw_skips_biased_values_and_fails_closed() {
    let reg = reg_with(&STD);
    // first u16 is 65_535 (rejected), second is 9_950 (kind 4)
    let mut r = [0u8; 32];
    r[..2].copy_from_slice(&65_535u16.to_le_bytes());
    r[2..4].copy_from_slice(&9_950u16.to_le_bytes());
    assert_eq!(draw(&reg, &r), Ok((4, 3)));
    // all sixteen values at or above the limit: no draw
    let mut hi = [0u8; 32];
    for i in 0..16 {
        hi[i * 2..i * 2 + 2].copy_from_slice(&60_000u16.to_le_bytes());
    }
    assert_eq!(draw(&reg, &hi), Err(ERR_DRAW));
    // no odds configured
    assert_eq!(draw(&reg_with(&[]), &rnd_for(5, 0)), Err(ERR_ODDS));
}

// ================================================================ operator and registry
#[test]
fn operator_unconfigured_refuses_everything() {
    let zero = [0u8; 32];
    assert_eq!(operator_with(&zero, &leak_acc(OP, true)), Err(ERR_NOT_CONFIGURED));
    assert_eq!(operator_with(&zero, &leak_acc(zero, true)), Err(ERR_NOT_CONFIGURED));
    assert_eq!(operator_with(&OP, &leak_acc(OP, true)), Ok(()));
    assert_eq!(operator_with(&OP, &leak_acc(OP, false)), Err(ERR_SIGNER));
    assert_eq!(operator_with(&OP, &leak_acc(ATTACKER, true)), Err(ERR_SIGNER));
}

#[test]
fn registry_requires_the_operator() {
    let mut fx = Fx::new();
    let p = fx.player(1);
    let mut ix = vec![4];
    ix.extend_from_slice(&GAME_MAGIC_OVER);
    ix.extend_from_slice(&GAME);
    ix.push(1);
    // a different signer in the operator position
    assert_eq!(call(&mut fx.w, &[(p, true, false), (fx.reg, false, true)], &ix), ERR_SIGNER);
    // the operator key present but unsigned
    assert_eq!(call(&mut fx.w, &[(fx.op, false, false), (fx.reg, false, true)], &ix), ERR_SIGNER);
    assert_eq!(fx.w[fx.reg].data[R_NGAMES], 0);
}

#[test]
fn registry_game_rules() {
    let mut fx = Fx::new();
    assert_eq!(fx.register(&GAME_MAGIC_OVER, &GAME, 1), 0);
    assert_eq!(fx.w[fx.reg].data[R_NGAMES], 1);
    // switching off and on is allowed, changing the program is not
    assert_eq!(fx.register(&GAME_MAGIC_OVER, &GAME, 0), 0);
    assert_eq!(fx.w[fx.reg].data[R_GAMES + 48], 0);
    assert_eq!(fx.register(&GAME_MAGIC_OVER, &ATTACKER, 1), ERR_PROGRAM_FIXED);
    assert_eq!(fx.w[fx.reg].data[R_GAMES + 16..R_GAMES + 48], GAME);
    assert_eq!(fx.register(&GAME_MAGIC_OVER, &GAME, 2), ERR_DATA);
    assert_eq!(fx.register(b"another-game\0\0\0\0", &[0; 32], 1), ERR_DATA);
    for i in 1..MAX_GAMES as u8 {
        let mut id = [0u8; 16];
        id[0] = i;
        assert_eq!(fx.register(&id, &GAME, 1), 0);
    }
    assert_eq!(fx.register(b"ninth\0\0\0\0\0\0\0\0\0\0\0", &GAME, 1), ERR_CAPACITY);
    // an achievement shard in the registry position is refused
    let mut ix = vec![4];
    ix.extend_from_slice(b"x\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    ix.extend_from_slice(&GAME);
    ix.push(1);
    assert_eq!(call(&mut fx.w, &[(fx.op, true, false), (fx.ach[0], false, true)], &ix), ERR_DATA);
}

#[test]
fn odds_rules_and_freeze() {
    let mut fx = Fx::new();
    assert_eq!(fx.set_odds(&[(1, 0, 5000), (2, 1, 4999)]), ERR_ODDS);
    assert_eq!(fx.set_odds(&[(1, 0, 5000), (1, 1, 5000)]), ERR_ODDS, "duplicate kind");
    assert_eq!(fx.set_odds(&[(1, 0, 0), (2, 1, 10_000)]), ERR_ODDS, "zero weight");
    assert_eq!(fx.set_odds(&[]), ERR_DATA);
    let nine: Vec<(u8, u8, u16)> = (1..=9).map(|k| (k, 0, 1000)).collect();
    assert_eq!(fx.set_odds(&nine), ERR_DATA);
    assert_eq!(fx.freeze(), ERR_ODDS, "cannot freeze with no odds");
    assert_eq!(fx.set_odds(&[(1, 0, 5000), (2, 1, 5000)]), 0);
    assert_eq!(fx.set_odds(&STD), 0, "odds may be replaced until frozen");
    assert_eq!(fx.w[fx.reg].data[R_NKINDS], 4);
    assert_eq!(fx.freeze(), 0);
    assert_eq!(fx.w[fx.reg].data[R_FROZEN], 1);
    assert_eq!(fx.set_odds(&[(1, 0, 10_000)]), ERR_FROZEN, "AT16: odds are fixed once frozen");
    assert_eq!(fx.freeze(), ERR_FROZEN);
    // registering a game stays possible after the freeze
    assert_eq!(fx.register(&GAME_MAGIC_OVER, &GAME, 1), 0);
    assert_eq!(&fx.w[fx.reg].data[R_KINDS..R_KINDS + 16], &[1, 0, 0x70, 0x17, 2, 1, 0xb8, 0x0b, 3, 2, 0x84, 0x03, 4, 3, 100, 0]);
}

// ================================================================ AT3, AT5, AT18: claims
#[test]
fn claim_records_a_validated_achievement() {
    let mut fx = Fx::ready();
    host::set_clock(500, 1_800_000_123);
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    assert_eq!(fx.ach_count(), 1);
    let s = fx.ach_slot(0, 0);
    assert_eq!(s[A_STATE], 1);
    assert_eq!(s[A_FLAGS], F_ELIGIBLE);
    assert_eq!(rd16(&s, A_ID), ACH_SIX_HITTER);
    assert_eq!(&s[A_GAME..A_GAME + 16], &GAME_MAGIC_OVER);
    assert_eq!(&s[A_PLAYER..A_PLAYER + 32], &pk(1));
    assert_eq!(rd32(&s, A_EVENT), 5);
    assert_eq!(s[A_EVENT + 4], 0);
    assert_eq!(rd64(&s, A_ISSUED), 1_800_000_123);
    assert_eq!(rd32(&s, A_REQ_SEQ), 0);
    // funded shard: the save is scheduled in the same transaction
    let cpis = host::cpis();
    assert_eq!(cpis.len(), 1);
    assert_eq!(cpis[0].program, MAGIC_PROGRAM);
    assert_eq!(cpis[0].data, vec![1, 0, 0, 0]);
    assert_eq!(cpis[0].metas[0], (fx.w[fx.ach[0]].key, true, true));
    assert_eq!(fx.w[fx.ach[0]].data[H_DIRTY], 0);
}

#[test]
fn claim_accepts_one_six_and_a_conceded_win() {
    // the winning side needs at least one six over the match; a win by concession is still a finished table
    let mut fx = Fx::ready();
    fx.set_table(0, T_FINISHED, 2, pk(9), pk(1), 3, 1, (4, 1));
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0, "side B winner with exactly one six");
}

#[test]
fn claim_refuses_unfinished_tables_and_tables_without_a_result() {
    // Changed 23 Nov 2026 (Boundary Hitter): a tie is a finished result and may be claimed; no result is "not finished".
    for (phase, winner, expect) in [(5u8, 1u8, ERR_NOT_FINISHED), (0, 1, ERR_NOT_FINISHED), (T_FINISHED, 0, ERR_NOT_FINISHED), (T_FINISHED, 4, ERR_NOT_FINISHED), (T_FINISHED, 3, 0)] {
        let mut fx = Fx::ready();
        fx.set_table(0, phase, winner, pk(1), pk(2), 5, 1, (3, 3));
        let p = fx.player(1);
        assert_eq!(fx.claim(p, 0), expect, "phase {} winner {}", phase, winner);
        assert_eq!(fx.ach_count(), if expect == 0 { 1 } else { 0 });
    }
}

#[test]
fn claim_is_personal_to_a_player_who_hit_a_boundary() {
    // AT18, changed 23 Nov 2026 (Boundary Hitter): winning is no longer needed; the claimant's own side must have hit
    // a four or a six, and only the two players of the match can claim.
    let mut fx = Fx::ready();
    // the loser hit two sixes, the winner none: the loser may claim, the winner may not
    fx.set_table(0, T_FINISHED, 1, pk(1), pk(2), 5, 1, (0, 2));
    let (a, b) = (fx.player(1), fx.player(2));
    assert_eq!(fx.claim(a, 0), ERR_NO_SIXES);
    assert_eq!(fx.claim(b, 0), 0);
    // a tie with sixes on both sides: the other player may claim too
    fx.set_table(0, T_FINISHED, 3, pk(1), pk(2), 6, 1, (2, 2));
    assert_eq!(fx.claim(a, 0), 0);
    assert_eq!(fx.ach_count(), 2);
    // the AI side has no key: a person cannot claim it, whatever they pass
    let mut fx = Fx::ready();
    let a = fx.player(1);
    fx.set_table(0, T_FINISHED, 2, pk(1), [0; 32], 7, 1, (0, 5));
    assert_eq!(fx.claim(a, 0), ERR_NO_SIXES, "side A has no boundary of its own");
    // a third party cannot claim someone else's match
    fx.win(1, 0, 8);
    let c = fx.player(3);
    assert_eq!(fx.claim(c, 0), ERR_NOT_WINNER);
    assert_eq!(fx.ach_count(), 0);
    assert_eq!(fx.claim(a, 0), 0);
    assert_eq!(fx.ach_count(), 1);
}

#[test]
fn a_four_alone_is_a_boundary() {
    // 23 Nov 2026: one four with no six, in a lost match, earns the achievement; no four and no six does not
    let mut fx = Fx::ready();
    let a = fx.player(1);
    fx.set_table(0, T_FINISHED, 2, pk(1), pk(2), 9, 1, (0, 0));
    assert_eq!(fx.claim(a, 0), ERR_NO_SIXES);
    let t = fx.tables[0];
    wr16(&mut fx.w[t].data, T_TOTALS + 2, 1); // side A: one four
    assert_eq!(fx.claim(a, 0), 0);
    assert_eq!(fx.ach_count(), 1);
}

#[test]
fn a_person_who_beats_the_ai_with_a_six_may_claim() {
    // Decision of 20 Nov 2026: AI victories stay eligible for Six Hitter. The AI side is the zero key.
    let mut fx = Fx::ready();
    let a = fx.player(1);
    // person (side A) wins with two sixes, the AI side (zero key) has none
    fx.set_table(0, T_FINISHED, 1, pk(1), [0; 32], 11, 1, (2, 0));
    assert_eq!(fx.claim(a, 0), 0);
    assert_eq!(fx.ach_count(), 1);
    // same on the other side of the table, and a win with no six still fails
    let mut fx = Fx::ready();
    let b = fx.player(2);
    fx.set_table(1, T_FINISHED, 2, [0; 32], pk(2), 12, 1, (0, 0));
    assert_eq!(fx.claim(b, 1), ERR_NO_SIXES);
    fx.set_table(1, T_FINISHED, 2, [0; 32], pk(2), 13, 1, (0, 1));
    assert_eq!(fx.claim(b, 1), 0);
}

#[test]
fn claim_needs_totals_and_a_match_number() {
    let mut fx = Fx::ready();
    let p = fx.player(1);
    fx.set_table(0, T_FINISHED, 1, pk(1), [0; 32], 5, 0, (9, 0));
    assert_eq!(fx.claim(p, 0), ERR_NO_TOTALS);
    fx.set_table(0, T_FINISHED, 1, pk(1), [0; 32], 0, 1, (9, 0));
    assert_eq!(fx.claim(p, 0), ERR_TABLE);
    fx.win(1, 0, 5);
    let t = fx.tables[0];
    fx.w[t].data[0] = b'X';
    assert_eq!(fx.claim(p, 0), ERR_TABLE, "wrong table magic");
}

#[test]
fn claim_refuses_forged_or_misdirected_accounts() {
    // AT3 and AT5
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    // a table owned by someone else
    let t = fx.tables[0];
    fx.w[t].owner = ATTACKER;
    assert_eq!(fx.claim(p, 0), ERR_OWNER);
    fx.w[t].owner = GAME;
    // a perfect copy of a winning table at an address that is not the table PDA
    let copy = fx.add(pk(500), GAME, 1, fx.w[t].data.clone(), false);
    let m = fx.claim_metas(p, copy);
    assert_eq!(call(&mut fx.w, &m, &Fx::claim_ix(ACH_SIX_HITTER, 0)), ERR_PDA);
    // the right table account but the wrong index named
    let m = fx.claim_metas(p, t);
    assert_eq!(call(&mut fx.w, &m, &Fx::claim_ix(ACH_SIX_HITTER, 1)), ERR_PDA);
    // table index out of range, unknown achievement
    assert_eq!(call(&mut fx.w, &m, &Fx::claim_ix(ACH_SIX_HITTER, 16)), ERR_RANGE);
    assert_eq!(call(&mut fx.w, &m, &Fx::claim_ix(2, 0)), ERR_NO_ACH);
    // unsigned player
    let mut m = fx.claim_metas(p, t);
    m[0].1 = false;
    assert_eq!(call(&mut fx.w, &m, &Fx::claim_ix(ACH_SIX_HITTER, 0)), ERR_SIGNER);
    // shards: read only, swapped, attacker owned, missing
    let mut m = fx.claim_metas(p, t);
    m[2].2 = false;
    assert_eq!(call(&mut fx.w, &m, &Fx::claim_ix(ACH_SIX_HITTER, 0)), ERR_ACCOUNTS);
    let mut m = fx.claim_metas(p, t);
    m.swap(2, 3);
    assert_eq!(call(&mut fx.w, &m, &Fx::claim_ix(ACH_SIX_HITTER, 0)), ERR_ACCOUNTS);
    let fake = fx.add(fx.w[fx.ach[0]].key, ATTACKER, RICH, fx.w[fx.ach[0]].data.clone(), false);
    let mut m = fx.claim_metas(p, t);
    m[2].0 = fake;
    assert_eq!(call(&mut fx.w, &m, &Fx::claim_ix(ACH_SIX_HITTER, 0)), ERR_OWNER);
    let m = fx.claim_metas(p, t);
    assert_eq!(call(&mut fx.w, &m[..9], &Fx::claim_ix(ACH_SIX_HITTER, 0)), ERR_DATA);
    // registry owned by someone else
    let r = fx.reg;
    fx.w[r].owner = ATTACKER;
    assert_eq!(fx.claim(p, 0), ERR_OWNER);
    fx.w[r].owner = PROG;
    assert_eq!(fx.ach_count(), 0);
    assert_eq!(fx.claim(p, 0), 0, "the honest claim still works after every refusal");
}

#[test]
fn claim_follows_the_registry() {
    // an unregistered game and a disabled game both refuse
    let mut fx = Fx::new();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), ERR_GAME_OFF);
    assert_eq!(fx.register(&GAME_MAGIC_OVER, &GAME, 0), 0);
    assert_eq!(fx.claim(p, 0), ERR_GAME_OFF);
    assert_eq!(fx.register(&GAME_MAGIC_OVER, &GAME, 1), 0);
    assert_eq!(fx.claim(p, 0), 0);
    // a table of a different program than the one registered does not count, even at its own correct PDA
    let mut fx = Fx::new();
    assert_eq!(fx.register(&GAME_MAGIC_OVER, &ATTACKER, 1), 0);
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), ERR_OWNER, "AT5: a table of a program that is not the registered issuer");
}

#[test]
fn claim_is_lifetime_and_once_per_player() {
    // AT4 and AT17
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p1 = fx.player(1);
    assert_eq!(fx.claim(p1, 0), 0);
    assert_eq!(fx.claim(p1, 0), ERR_ALREADY_CLAIMED, "same table, same match");
    fx.win(1, 0, 6);
    assert_eq!(fx.claim(p1, 0), ERR_ALREADY_CLAIMED, "same table, a later match");
    fx.win(1, 1, 9);
    assert_eq!(fx.claim(p1, 1), ERR_ALREADY_CLAIMED, "another table");
    assert_eq!(fx.ach_count(), 1);
    // another player is unaffected
    fx.win(2, 1, 10);
    let p2 = fx.player(2);
    assert_eq!(fx.claim(p2, 1), 0);
    assert_eq!(fx.ach_count(), 2);
}

#[test]
fn claim_allocation_fills_shards_in_order_then_refuses() {
    // AT19
    let mut fx = Fx::ready();
    for n in 0..(4 * ACH_PER) as u16 {
        fx.win(n + 1, 0, n as u32 + 1);
        let p = fx.player(n + 1);
        assert_eq!(fx.claim(p, 0), 0, "claim {}", n);
    }
    for i in 0..4 {
        let n = (0..ACH_PER).filter(|s| fx.w[fx.ach[i]].data[HEADER + s * ACH_SLOT] == 1).count();
        assert_eq!(n, ACH_PER, "shard {}", i);
    }
    assert_eq!(fx.ach_count(), 424);
    let extra = 4 * ACH_PER as u16 + 1;
    fx.win(extra, 0, 9_999);
    let p = fx.player(extra);
    assert_eq!(fx.claim(p, 0), ERR_CAPACITY);
    assert_eq!(fx.ach_count(), 424);
    // and every recorded player is distinct
    let mut seen = std::collections::HashSet::new();
    for i in 0..4 {
        for s in 0..ACH_PER {
            let o = HEADER + s * ACH_SLOT;
            let d = &fx.w[fx.ach[i]].data;
            assert!(seen.insert(d[o + A_PLAYER..o + A_PLAYER + 32].to_vec()));
        }
    }
}

// ================================================================ saving to Solana
#[test]
fn save_is_self_scheduled_only_when_the_shard_is_funded() {
    let rent = rent_minimum(SHARD_SIZE);
    assert_eq!(rent, (128 + SHARD_SIZE as u64) * 3480 * 2);
    for (lamports, scheduled) in [(rent + SPONSOR_MARGIN, true), (rent + SPONSOR_MARGIN - 1, false), (1_000, false)] {
        let mut fx = Fx::ready();
        let a = fx.ach[0];
        fx.w[a].lamports = lamports;
        fx.win(1, 0, 5);
        let p = fx.player(1);
        assert_eq!(fx.claim(p, 0), 0);
        assert_eq!(host::cpis().len(), scheduled as usize, "lamports {}", lamports);
        assert_eq!(fx.w[a].data[H_DIRTY], if scheduled { 0 } else { 1 });
    }
}

#[test]
fn save_uses_the_current_rent_rule_too() {
    // the newer rent shape: threshold 1.0 and a doubled per byte rate gives the same minimum
    host::reset();
    host::set_rent(6960, 0x3FF0_0000_0000_0000);
    assert_eq!(rent_minimum(SHARD_SIZE), (128 + SHARD_SIZE as u64) * 6960);
}

#[test]
fn commit_is_permissionless_but_only_for_unsaved_changes() {
    let mut fx = Fx::ready();
    let a = fx.ach[0];
    fx.w[a].lamports = 1_000;
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    assert_eq!(fx.w[a].data[H_DIRTY], 1);
    assert!(host::cpis().is_empty());
    let stranger = fx.player(77);
    // unfunded: refused, nothing changes
    assert_eq!(fx.commit(stranger, a), ERR_SPONSOR_LOW);
    assert_eq!(fx.w[a].data[H_DIRTY], 1);
    // sponsor tops it up; anyone may then trigger the save
    fx.w[a].lamports = RICH;
    assert_eq!(fx.commit(stranger, a), 0);
    assert_eq!(fx.w[a].data[H_DIRTY], 0);
    let c = host::cpis();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].program, MAGIC_PROGRAM);
    // spam: nothing unsaved, so a second call is refused (no repeated scheduling by a stranger)
    assert_eq!(fx.commit(stranger, a), ERR_NOTHING_TO_SAVE);
    assert_eq!(fx.commit(stranger, fx.ach[1]), ERR_NOTHING_TO_SAVE);
}

#[test]
fn commit_checks_its_accounts() {
    let mut fx = Fx::ready();
    let a = fx.ach[0];
    fx.w[a].data[H_DIRTY] = 1;
    let s = fx.player(5);
    let not_ledger = fx.player(6);
    assert_eq!(fx.commit(s, not_ledger), ERR_DATA, "not a ledger account");
    fx.w[a].owner = ATTACKER;
    assert_eq!(fx.commit(s, a), ERR_OWNER);
    fx.w[a].owner = PROG;
    let m = [(s, true, true), (a, false, true), (fx.magic, false, false), (fx.ctx, false, true), (fx.reg, false, true)];
    assert_eq!(call(&mut fx.w, &m, &[2]), ERR_ACCOUNTS, "wrong fee vault");
    let m = [(s, true, true), (a, false, true), (fx.vrf, false, false), (fx.ctx, false, true), (fx.vault, false, true)];
    assert_eq!(call(&mut fx.w, &m, &[2]), ERR_ACCOUNTS, "not the magic program");
    let m = [(s, false, true), (a, false, true), (fx.magic, false, false), (fx.ctx, false, true), (fx.vault, false, true)];
    assert_eq!(call(&mut fx.w, &m, &[2]), ERR_ACCOUNTS, "unsigned payer");
    assert_eq!(fx.commit(s, a), 0);
}

#[test]
fn commit_and_undelegate_is_operator_only_while_the_reserve_is_healthy() {
    let mut fx = Fx::ready();
    let m = |fx: &Fx, op: usize, sig: bool, pda: usize, ctx: usize| [(op, sig, true), (pda, false, true), (fx.magic, false, false), (ctx, false, true)];
    let ix = [3u8, KIND_ACH, 0];
    let good = m(&fx, fx.op, true, fx.ach[0], fx.ctx);
    let stranger = fx.player(3);
    let bad_signer = m(&fx, stranger, true, fx.ach[0], fx.ctx);
    assert_eq!(call(&mut fx.w, &bad_signer, &ix), ERR_SIGNER);
    let unsigned = m(&fx, fx.op, false, fx.ach[0], fx.ctx);
    assert_eq!(call(&mut fx.w, &unsigned, &ix), ERR_SIGNER);
    let wrong_pda = m(&fx, fx.op, true, fx.ach[1], fx.ctx);
    assert_eq!(call(&mut fx.w, &wrong_pda, &ix), ERR_PDA);
    let wrong_ctx = m(&fx, fx.op, true, fx.ach[0], fx.vault);
    assert_eq!(call(&mut fx.w, &wrong_ctx, &ix), ERR_ACCOUNTS);
    assert_eq!(call(&mut fx.w, &good, &[3, 7, 0]), ERR_DATA);
    assert_eq!(call(&mut fx.w, &good, &ix), 0);
    let c = host::cpis();
    assert_eq!(c[0].program, MAGIC_PROGRAM);
    assert_eq!(c[0].data, vec![2, 0, 0, 0]);
}

#[test]
fn anyone_can_hand_back_an_account_whose_reserve_is_low() {
    let mut fx = Fx::ready();
    let a = fx.ach[2];
    let stranger = fx.player(5);
    let m = [(stranger, true, true), (a, false, true), (fx.magic, false, false), (fx.ctx, false, true)];
    let ix = [3u8, KIND_ACH, 2];
    let low = rent_minimum(fx.w[a].data.len()) + LOW_RESERVE;
    // ten paid saves left is not low yet
    fx.w[a].lamports = low;
    assert_eq!(call(&mut fx.w, &m, &ix), ERR_SIGNER);
    // fewer than ten: anyone may hand it back
    fx.w[a].lamports = low - 1;
    let before = host::cpis().len();
    assert_eq!(call(&mut fx.w, &m, &ix), 0);
    let c = host::cpis();
    assert_eq!(c.len(), before + 1);
    assert_eq!(c[before].program, MAGIC_PROGRAM);
    assert_eq!(c[before].data, vec![2, 0, 0, 0]);
    // the stranger must still sign, and still name the right account and context
    let unsigned = [(stranger, false, true), (a, false, true), (fx.magic, false, false), (fx.ctx, false, true)];
    assert_eq!(call(&mut fx.w, &unsigned, &ix), ERR_SIGNER);
    assert_eq!(call(&mut fx.w, &m, &[3, KIND_ACH, 1]), ERR_PDA);
    let wrong_ctx = [(stranger, true, true), (a, false, true), (fx.magic, false, false), (fx.vault, false, true)];
    assert_eq!(call(&mut fx.w, &wrong_ctx, &ix), ERR_ACCOUNTS);
}

// ================================================================ AT20: request state machine
#[test]
fn request_marks_pending_and_asks_the_vrf_for_the_right_callback() {
    let mut fx = Fx::ready();
    host::set_clock(777, 1_800_000_000);
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    assert_eq!(fx.request(p, 0, 0), 0);
    let s = fx.ach_slot(0, 0);
    assert_eq!(s[A_FLAGS], F_PENDING);
    assert_eq!(rd32(&s, A_REQ_SEQ), 1);
    assert_eq!(rd64(&s, A_REQ_AT), 1_800_000_000);
    let c = host::cpis();
    assert_eq!(c.len(), 1, "one VRF request, and no commit scheduled by a request");
    let v = &c[0];
    assert_eq!(v.program, VRF_PROGRAM);
    assert_eq!(v.metas[0], (pk(1), true, true));
    assert_eq!(v.metas[1], (fx.w[fx.ident].key, false, true));
    assert_eq!(v.metas[2], (VRF_QUEUE, true, false));
    // data: tag 10, seed, our program id, callback discriminator, six accounts, caller data
    let d = &v.data;
    assert_eq!(d[0], 10);
    let mut expected_seed = [0u8; 32];
    let mut ss = [0u8; 7];
    ss[3..7].copy_from_slice(&1u32.to_le_bytes());
    host::sha256(&[&fx.w[fx.ach[0]].key, &ss, &777u64.to_le_bytes()], &mut expected_seed);
    assert_eq!(&d[8..40], &expected_seed);
    assert_eq!(&d[40..72], &PROG);
    assert_eq!(rd32(d, 72), 8);
    assert_eq!(&d[76..84], b"PLAYVRF1");
    assert_eq!(rd32(d, 84), 6);
    let acct = |i: usize| &d[88 + i * 34..88 + (i + 1) * 34];
    assert_eq!(&acct(0)[..32], &fx.w[fx.ach[0]].key);
    assert_eq!(acct(0)[33], 1);
    assert_eq!(&acct(1)[..32], &fx.w[fx.reg].key);
    assert_eq!(acct(1)[33], 0);
    for i in 0..4 {
        assert_eq!(&acct(2 + i)[..32], &fx.w[fx.rel[i]].key);
        assert_eq!(acct(2 + i)[33], 1);
    }
    assert_eq!(rd32(d, 88 + 6 * 34), 7);
    assert_eq!(&d[88 + 6 * 34 + 4..], &[0, 0, 0, 1, 0, 0, 0]);
    // the identity signs by seed
    assert_eq!(v.seeds, vec![vec![IDENTITY_SEED.to_vec(), vec![255]]]);
}

#[test]
fn request_state_machine_and_replacement() {
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    let other = fx.player(2);
    // nothing to request yet
    assert_eq!(fx.request(p, 0, 0), ERR_NO_ACH);
    assert_eq!(fx.claim(p, 0), 0);
    assert_eq!(fx.request(p, 0, ACH_PER as u16), ERR_RANGE);
    assert_eq!(fx.request(other, 0, 0), ERR_NOT_OWNER, "someone else's achievement");
    assert_eq!(fx.request(p, 0, 0), 0);
    assert_eq!(fx.request(p, 0, 0), ERR_TOO_EARLY);
    host::advance_secs(7);
    assert_eq!(fx.request(p, 0, 0), ERR_TOO_EARLY);
    assert_eq!(rd32(&fx.ach_slot(0, 0), A_REQ_SEQ), 1);
    host::advance_secs(1);
    assert_eq!(fx.request(other, 0, 0), ERR_NOT_OWNER, "only the player may replace a pending request");
    assert_eq!(fx.request(p, 0, 0), 0);
    assert_eq!(rd32(&fx.ach_slot(0, 0), A_REQ_SEQ), 2);
    assert_eq!(fx.ach_slot(0, 0)[A_FLAGS], F_PENDING);
}

#[test]
fn request_refuses_wrong_accounts_and_unfrozen_odds() {
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    let ix = [8u8, 0, 0];
    let mut m = fx.request_metas(p, 0);
    m[2].0 = fx.vrf_ident; // not this program's identity PDA
    assert_eq!(call(&mut fx.w, &m, &ix), ERR_ACCOUNTS);
    let mut m = fx.request_metas(p, 0);
    m[3].0 = fx.sys; // not the VRF program
    assert_eq!(call(&mut fx.w, &m, &ix), ERR_ACCOUNTS);
    let mut m = fx.request_metas(p, 0);
    m[4].0 = fx.vault; // not the oracle queue
    assert_eq!(call(&mut fx.w, &m, &ix), ERR_ACCOUNTS);
    let mut m = fx.request_metas(p, 0);
    m[0].1 = false;
    assert_eq!(call(&mut fx.w, &m, &ix), ERR_SIGNER);
    let m = fx.request_metas(p, 0);
    assert_eq!(call(&mut fx.w, &m[..7], &ix), ERR_DATA, "registry account is required");
    let mut m = fx.request_metas(p, 0);
    m[1].2 = false;
    assert_eq!(call(&mut fx.w, &m, &ix), ERR_ACCOUNTS, "read only shard");
    // odds not frozen: no VRF request is spent
    let mut fx = Fx::new();
    assert_eq!(fx.register(&GAME_MAGIC_OVER, &GAME, 1), 0);
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    assert_eq!(fx.request(p, 0, 0), ERR_NOT_FROZEN);
    assert!(host::cpis().iter().all(|c| c.program != VRF_PROGRAM));
}

#[test]
fn a_failed_vrf_request_leaves_the_achievement_requestable() {
    // relies on whole-transaction rollback, which the harness imitates and the runtime provides
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    host::fail_cpi_to(Some(VRF_PROGRAM));
    assert_eq!(fx.request(p, 0, 0), ERR_CPI);
    assert_eq!(fx.ach_slot(0, 0)[A_FLAGS], F_ELIGIBLE);
    host::fail_cpi_to(None);
    assert_eq!(fx.request(p, 0, 0), 0);
}

// ================================================================ the VRF callback and minting
#[test]
fn callback_mints_the_drawn_relic_to_the_requester() {
    let mut fx = Fx::ready();
    host::set_clock(10, 1_800_000_000);
    // 9_950 draws kind 4 (rarity 3); its low byte 0xDE = 222, and 222 mod 4 = 2, so shard 2
    let rnd = rnd_for(9_950, 0x31);
    let (sh, sl) = fx.earn(1, 0, 5, &rnd);
    assert_eq!((sh, sl), (2, 0));
    let r = fx.rel_slot(sh, sl);
    assert_eq!(r[L_STATE], 1);
    assert_eq!(r[L_KIND], 4);
    assert_eq!(r[L_RARITY], 3);
    assert_eq!(rd32(&r, L_COUNT), 0);
    assert_eq!(&r[L_ID..L_ID + 32], &rnd);
    assert_eq!(&r[L_OWNER..L_OWNER + 32], &pk(1));
    assert_eq!(&r[L_ORIGIN..L_ORIGIN + 32], &pk(1));
    assert_eq!(rd64(&r, L_MINTED), 1_800_000_000);
    let a = fx.ach_slot(0, 0);
    assert_eq!(a[A_FLAGS], F_MINTED);
    // the callback cannot schedule a save (it has no rollup accounts); the shards wait for the next commit
    assert_eq!(fx.w[fx.rel[2]].data[H_DIRTY], 1);
    assert_eq!(fx.w[fx.ach[0]].data[H_DIRTY], 1);
    let p = fx.player(1);
    assert_eq!(fx.commit(p, fx.rel[2]), 0);
    assert_eq!(fx.commit(p, fx.ach[0]), 0);
}

#[test]
fn callback_rejects_abuse() {
    // AT14
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    let rnd = rnd_for(100, 0x41);
    let ix = |seq: u32| Fx::callback_ix(&rnd, 0, 0, seq);
    // never requested
    let m = fx.callback_metas(true, 0);
    assert_eq!(call(&mut fx.w, &m, &ix(1)), ERR_NOT_PENDING);
    assert_eq!(fx.request(p, 0, 0), 0);
    // not signed, or signed by some other key
    let m = fx.callback_metas(false, 0);
    assert_eq!(call(&mut fx.w, &m, &ix(1)), ERR_IDENTITY);
    let mut m = fx.callback_metas(true, 0);
    m[0].0 = fx.ident;
    assert_eq!(call(&mut fx.w, &m, &ix(1)), ERR_IDENTITY);
    let mut m = fx.callback_metas(true, 0);
    m[0].0 = fx.player(9);
    assert_eq!(call(&mut fx.w, &m, &ix(1)), ERR_IDENTITY);
    // wrong data length, wrong shard, wrong slot
    let m = fx.callback_metas(true, 0);
    assert_eq!(call(&mut fx.w, &m, &ix(1)[..46]), ERR_DATA);
    assert_eq!(call(&mut fx.w, &m, &Fx::callback_ix(&rnd, 1, 0, 1)), ERR_DATA);
    assert_eq!(call(&mut fx.w, &m, &Fx::callback_ix(&rnd, 0, ACH_PER as u16, 1)), ERR_DATA);
    assert_eq!(call(&mut fx.w, &m, &Fx::callback_ix(&rnd, 0, 1, 1)), ERR_NOT_PENDING, "an empty slot");
    // attacker owned relic shard, attacker owned achievement shard, relic shards out of order
    let fake = fx.add(fx.w[fx.rel[0]].key, ATTACKER, RICH, fx.w[fx.rel[0]].data.clone(), false);
    let mut m = fx.callback_metas(true, 0);
    m[3].0 = fake;
    assert_eq!(call(&mut fx.w, &m, &ix(1)), ERR_OWNER);
    let mut m = fx.callback_metas(true, 0);
    m.swap(3, 4);
    assert_eq!(call(&mut fx.w, &m, &ix(1)), ERR_ACCOUNTS);
    // a wrong sequence number
    let m = fx.callback_metas(true, 0);
    assert_eq!(call(&mut fx.w, &m, &ix(2)), ERR_SEQ);
    assert_eq!(call(&mut fx.w, &m, &ix(0)), ERR_SEQ);
    assert_eq!(fx.ach_slot(0, 0)[A_FLAGS], F_PENDING, "nothing above changed the request");
    // the genuine callback, then a replay of it
    assert_eq!(fx.callback(0, 0, 1, &rnd), 0);
    assert_eq!(fx.callback(0, 0, 1, &rnd), ERR_NOT_PENDING);
    assert_eq!(fx.callback(0, 0, 1, &rnd_for(100, 0x42)), ERR_NOT_PENDING, "a second callback with other randomness");
    assert_eq!(fx.request(p, 0, 0), ERR_MINTED);
    assert!((0..4).map(|i| (0..REL_PER).filter(|s| fx.w[fx.rel[i]].data[HEADER + s * REL_SLOT] == 1).count()).sum::<usize>() == 1);
}

#[test]
fn callback_needs_frozen_odds() {
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    assert_eq!(fx.request(p, 0, 0), 0);
    let reg = fx.reg;
    fx.w[reg].data[R_FROZEN] = 0;
    assert_eq!(fx.callback(0, 0, 1, &rnd_for(5, 1)), ERR_NOT_FROZEN);
}

#[test]
fn callback_with_unusable_randomness_leaves_a_retry() {
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    assert_eq!(fx.request(p, 0, 0), 0);
    let mut hi = [0u8; 32];
    for i in 0..16 {
        hi[i * 2..i * 2 + 2].copy_from_slice(&65_000u16.to_le_bytes());
    }
    assert_eq!(fx.callback(0, 0, 1, &hi), ERR_DRAW);
    assert_eq!(fx.ach_slot(0, 0)[A_FLAGS], F_PENDING);
    host::advance_secs(8);
    assert_eq!(fx.request(p, 0, 0), 0);
    assert_eq!(fx.callback(0, 0, 2, &rnd_for(10, 0x51)), 0);
}

#[test]
fn relic_placement_probes_forward_and_refuses_when_full() {
    // AT19
    let mut fx = Fx::ready();
    // fill shard 2 completely
    let sh2 = fx.rel[2];
    for s in 0..REL_PER {
        fx.w[sh2].data[HEADER + s * REL_SLOT] = 1;
    }
    let rnd = rnd_for(9_950, 0x61); // starts at shard 2
    let (sh, sl) = fx.earn(1, 0, 5, &rnd);
    assert_eq!((sh, sl), (3, 0), "moves on to the next shard");
    // fill the rest: every shard is full, the callback fails and the request stays pending
    for i in 0..4 {
        for s in 0..REL_PER {
            let o = HEADER + s * REL_SLOT;
            fx.w[fx.rel[i]].data[o] = 1;
        }
    }
    fx.win(2, 1, 6);
    let p2 = fx.player(2);
    assert_eq!(fx.claim(p2, 1), 0);
    let (ash, asl) = fx.find_ach(2).unwrap();
    assert_eq!(fx.request(p2, ash, asl as u16), 0);
    assert_eq!(fx.callback(ash as u8, asl as u16, 1, &rnd_for(10, 0x62)), ERR_CAPACITY);
    assert_eq!(fx.ach_slot(ash, asl)[A_FLAGS], F_PENDING);
}

// ================================================================ transfer
#[test]
fn transfer_moves_ownership_and_keeps_the_origin() {
    // T6 and T10
    let mut fx = Fx::ready();
    let rnd = rnd_for(100, 0x71);
    let (sh, sl) = fx.earn(1, 0, 5, &rnd);
    let (p1, p2, p3) = (fx.player(1), fx.player(2), fx.player(3));
    assert_eq!(fx.transfer(p1, sh, sl as u16, &pk(2)), 0);
    let r = fx.rel_slot(sh, sl);
    assert_eq!(&r[L_OWNER..L_OWNER + 32], &pk(2));
    assert_eq!(&r[L_ORIGIN..L_ORIGIN + 32], &pk(1));
    assert_eq!(rd32(&r, L_COUNT), 1);
    // the save is scheduled with the transfer
    assert_eq!(host::cpis().len(), 1);
    // the previous owner has lost control
    assert_eq!(fx.transfer(p1, sh, sl as u16, &pk(3)), ERR_NOT_OWNER);
    // a chain: 2 to 3, 3 back to 1
    assert_eq!(fx.transfer(p2, sh, sl as u16, &pk(3)), 0);
    assert_eq!(fx.transfer(p3, sh, sl as u16, &pk(1)), 0);
    let r = fx.rel_slot(sh, sl);
    assert_eq!(&r[L_OWNER..L_OWNER + 32], &pk(1));
    assert_eq!(rd32(&r, L_COUNT), 3);
    assert_eq!(&r[L_ID..L_ID + 32], &rnd, "identity never changes");
}

#[test]
fn transfer_rejections() {
    let mut fx = Fx::ready();
    let rnd = rnd_for(100, 0x72);
    let (sh, sl) = fx.earn(1, 0, 5, &rnd);
    let (p1, p2) = (fx.player(1), fx.player(2));
    assert_eq!(fx.transfer(p2, sh, sl as u16, &pk(2)), ERR_NOT_OWNER, "not the owner");
    assert_eq!(fx.transfer(p1, sh, sl as u16, &pk(1)), ERR_SAME_OWNER);
    assert_eq!(fx.transfer(p1, sh, sl as u16, &[0; 32]), ERR_DATA);
    assert_eq!(fx.transfer(p1, sh, sl as u16 + 1, &pk(2)), ERR_NO_ACH, "an empty slot");
    assert_eq!(fx.transfer(p1, sh, REL_PER as u16, &pk(2)), ERR_RANGE);
    // unsigned owner, attacker owned shard, read only shard, wrong length
    let mut ix = vec![10];
    ix.extend_from_slice(&(sl as u16).to_le_bytes());
    ix.extend_from_slice(&pk(2));
    let ok = [(p1, true, false), (fx.rel[sh], false, true), (fx.magic, false, false), (fx.ctx, false, true), (fx.vault, false, true)];
    let mut m = ok;
    m[0].1 = false;
    assert_eq!(call(&mut fx.w, &m, &ix), ERR_SIGNER);
    let mut m = ok;
    m[1].2 = false;
    assert_eq!(call(&mut fx.w, &m, &ix), ERR_ACCOUNTS);
    assert_eq!(call(&mut fx.w, &ok, &ix[..30]), ERR_DATA);
    let key = fx.w[fx.rel[sh]].key;
    let data = fx.w[fx.rel[sh]].data.clone();
    let fake = fx.add(key, ATTACKER, RICH, data, false);
    let mut m = ok;
    m[1].0 = fake;
    assert_eq!(call(&mut fx.w, &m, &ix), ERR_OWNER);
    // an achievement shard is not a relic shard
    let mut m = ok;
    m[1].0 = fx.ach[0];
    assert_eq!(call(&mut fx.w, &m, &ix), ERR_DATA);
    // counter overflow is refused, not wrapped
    let o = HEADER + sl * REL_SLOT + L_COUNT;
    let rs = fx.rel[sh];
    wr32(&mut fx.w[rs].data, o, u32::MAX);
    assert_eq!(call(&mut fx.w, &ok, &ix), ERR_RANGE);
    // the honest owner is untouched by all of it
    assert_eq!(&fx.rel_slot(sh, sl)[L_OWNER..L_OWNER + 32], &pk(1));
}

#[test]
fn transfer_without_a_funded_shard_defers_the_save() {
    let mut fx = Fx::ready();
    let (sh, sl) = fx.earn(1, 0, 5, &rnd_for(100, 0x73));
    let rs = fx.rel[sh];
    fx.w[rs].lamports = 1_000;
    let p1 = fx.player(1);
    assert_eq!(fx.transfer(p1, sh, sl as u16, &pk(2)), 0, "the transfer stands in the rollup");
    assert!(host::cpis().is_empty());
    assert_eq!(fx.w[rs].data[H_DIRTY], 1);
}

#[test]
fn transfer_does_not_reopen_the_reward() {
    let mut fx = Fx::ready();
    let (sh, sl) = fx.earn(1, 0, 5, &rnd_for(100, 0x74));
    let (p1, p2) = (fx.player(1), fx.player(2));
    assert_eq!(fx.transfer(p1, sh, sl as u16, &pk(2)), 0);
    let (ash, asl) = fx.find_ach(1).unwrap();
    assert_eq!(fx.request(p1, ash, asl as u16), ERR_MINTED);
    assert_eq!(fx.request(p2, ash, asl as u16), ERR_NOT_OWNER);
    fx.win(1, 1, 99);
    assert_eq!(fx.claim(p1, 1), ERR_ALREADY_CLAIMED, "AT17: giving the relic away does not earn another");
}

// ================================================================ AT22 and AT23: interleavings
// The rollup runs one transaction at a time per account, so concurrency reduces to orderings of whole instructions.
// These cover the orderings that matter. How a live rollup actually schedules them is PENDING LIVE VERIFICATION.
#[test]
fn interleaved_claims_from_two_tables_and_two_players() {
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    fx.win(2, 1, 5);
    fx.win(1, 2, 6);
    let (p1, p2) = (fx.player(1), fx.player(2));
    assert_eq!(fx.claim(p1, 0), 0);
    assert_eq!(fx.claim(p2, 1), 0);
    assert_eq!(fx.claim(p1, 2), ERR_ALREADY_CLAIMED);
    assert_eq!(fx.claim(p2, 1), ERR_ALREADY_CLAIMED);
    assert_eq!(fx.ach_count(), 2);
    let (a, b) = (fx.find_ach(1).unwrap(), fx.find_ach(2).unwrap());
    assert_ne!(a, b);
}

#[test]
fn stale_callback_after_replacement_cannot_mint() {
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    assert_eq!(fx.request(p, 0, 0), 0); // seq 1
    host::advance_secs(9);
    assert_eq!(fx.request(p, 0, 0), 0); // seq 2 replaces it
    assert_eq!(fx.callback(0, 0, 1, &rnd_for(10, 0x81)), ERR_SEQ, "late answer to the first request");
    assert_eq!(fx.callback(0, 0, 2, &rnd_for(20, 0x82)), 0);
    assert_eq!(fx.callback(0, 0, 2, &rnd_for(30, 0x83)), ERR_NOT_PENDING, "duplicate of the winning answer");
    assert!(fx.find_relic(&rnd_for(10, 0x81)).is_none());
    assert!(fx.find_relic(&rnd_for(30, 0x83)).is_none());
    assert!(fx.find_relic(&rnd_for(20, 0x82)).is_some());
}

#[test]
fn two_requesters_mint_into_distinct_slots() {
    let mut fx = Fx::ready();
    let a = fx.earn(1, 0, 5, &rnd_for(9_950, 0x91));
    let b = fx.earn(2, 1, 6, &rnd_for(9_950, 0x92));
    assert_ne!(a, b);
    assert_eq!(a.0, b.0, "same starting shard, so consecutive slots");
}

#[test]
fn only_one_of_two_competing_transfers_succeeds() {
    let mut fx = Fx::ready();
    let (sh, sl) = fx.earn(1, 0, 5, &rnd_for(100, 0xA1));
    let p1 = fx.player(1);
    let _ = fx.player(2);
    let _ = fx.player(3);
    assert_eq!(fx.transfer(p1, sh, sl as u16, &pk(2)), 0);
    assert_eq!(fx.transfer(p1, sh, sl as u16, &pk(3)), ERR_NOT_OWNER);
    let r = fx.rel_slot(sh, sl);
    assert_eq!(&r[L_OWNER..L_OWNER + 32], &pk(2));
    assert_eq!(rd32(&r, L_COUNT), 1);
}

// ================================================================ accounts: create, delegate, return
#[test]
fn init_creates_ledger_accounts_for_the_operator_only() {
    let mut fx = Fx::bare();
    let mut ix = vec![0, KIND_REG, 0];
    ix.extend_from_slice(&50_000_000u64.to_le_bytes());
    let m = [(fx.op, true, true), (fx.reg, false, true), (fx.sys, false, false)];
    let stranger = fx.player(4);
    assert_eq!(call(&mut fx.w, &[(stranger, true, true), (fx.reg, false, true), (fx.sys, false, false)], &ix), ERR_SIGNER);
    assert_eq!(call(&mut fx.w, &m, &ix), 0);
    let r = &fx.w[fx.reg];
    assert_eq!(r.owner, PROG);
    assert_eq!(r.data.len(), REG_SIZE);
    assert_eq!(&r.data[..8], MAGIC_LEDGER);
    assert_eq!((r.data[H_KIND], r.data[H_VERSION], r.data[H_IDX]), (KIND_REG, VERSION, 0));
    assert_eq!(&r.data[R_ADMIN..R_ADMIN + 32], &OP);
    assert_eq!(r.lamports, 50_000_000);
    assert_eq!(fx.w[fx.op].lamports, 10_000_000_000 - 50_000_000);
    let c = host::cpis();
    assert_eq!(c[0].program, SYSTEM_PROGRAM);
    assert_eq!(c[0].seeds, vec![vec![REG_SEED.to_vec(), vec![255]]]);
    // shards and bad requests
    let mut ix = vec![0, KIND_REL, 3];
    ix.extend_from_slice(&1u64.to_le_bytes());
    assert_eq!(call(&mut fx.w, &[(fx.op, true, true), (fx.rel[3], false, true), (fx.sys, false, false)], &ix), 0);
    assert_eq!(fx.w[fx.rel[3]].data.len(), SHARD_SIZE);
    assert_eq!(fx.w[fx.rel[3]].data[H_IDX], 3);
    let bad = |kind: u8, idx: u8| {
        let mut ix = vec![0, kind, idx];
        ix.extend_from_slice(&1u64.to_le_bytes());
        ix
    };
    assert_eq!(call(&mut fx.w, &[(fx.op, true, true), (fx.rel[0], false, true), (fx.sys, false, false)], &bad(KIND_REL, 4)), ERR_DATA);
    assert_eq!(call(&mut fx.w, &[(fx.op, true, true), (fx.reg, false, true), (fx.sys, false, false)], &bad(KIND_REG, 1)), ERR_DATA);
    assert_eq!(call(&mut fx.w, &[(fx.op, true, true), (fx.ach[0], false, true), (fx.sys, false, false)], &bad(3, 0)), ERR_DATA);
    // the wrong account for the named shard
    assert_eq!(call(&mut fx.w, &[(fx.op, true, true), (fx.ach[1], false, true), (fx.sys, false, false)], &bad(KIND_ACH, 0)), ERR_PDA);
    // creating the registry again is refused by the system program (account already in use)
    let mut again = vec![0, KIND_REG, 0];
    again.extend_from_slice(&1u64.to_le_bytes());
    assert_eq!(call(&mut fx.w, &m, &again), ERR_CPI);
}

fn delegate_ix(kind: u8, idx: u8, commit_ms: u32) -> Vec<u8> {
    let mut ix = vec![1, kind, idx, 1];
    ix.extend_from_slice(&VALIDATOR);
    ix.extend_from_slice(&commit_ms.to_le_bytes());
    ix
}

#[test]
fn delegate_hands_a_shard_to_the_delegation_program() {
    let mut fx = Fx::new();
    let pda = fx.ach[1];
    fx.w[pda].data[HEADER + 3] = 0x5a;
    let buf_key = find_pda(&[b"buffer", &fx.w[pda].key], &PROG).0;
    let buffer = fx.add(buf_key, SYS, 0, vec![], false);
    let record = fx.add(pk(600), SYS, 0, vec![], false);
    let meta = fx.add(pk(601), SYS, 0, vec![], false);
    let m = [
        (fx.op, true, true),
        (pda, false, true),
        (fx.prog, false, false),
        (buffer, false, true),
        (record, false, true),
        (meta, false, true),
        (fx.deleg, false, false),
        (fx.sys, false, false),
    ];
    // anyone but the operator: only to this program's validator and at the operator's interval
    let stranger = fx.player(4);
    fx.w[stranger].lamports = RICH;
    let mut bad = m;
    bad[0].0 = stranger;
    assert_eq!(call(&mut fx.w, &bad, &delegate_ix(KIND_ACH, 1, 10_000)), ERR_SIGNER, "other interval");
    let mut other_validator = delegate_ix(KIND_ACH, 1, 30_000);
    other_validator[4] ^= 1;
    assert_eq!(call(&mut fx.w, &bad, &other_validator), ERR_SIGNER, "other validator");
    let mut no_validator = delegate_ix(KIND_ACH, 1, 30_000);
    no_validator[3] = 0;
    assert_eq!(call(&mut fx.w, &bad, &no_validator), ERR_SIGNER, "no validator named");
    bad[0].1 = false;
    assert_eq!(call(&mut fx.w, &bad, &delegate_ix(KIND_ACH, 1, 30_000)), ERR_SIGNER, "unsigned");
    assert_eq!(call(&mut fx.w, &m, &delegate_ix(KIND_ACH, 0, 30_000)), ERR_PDA, "named shard is not this account");
    assert_eq!(call(&mut fx.w, &m, &delegate_ix(KIND_ACH, 1, 30_000)[..39]), ERR_ACCOUNTS);
    let mut bad = m;
    bad[3].0 = record;
    assert_eq!(call(&mut fx.w, &bad, &delegate_ix(KIND_ACH, 1, 30_000)), ERR_PDA, "buffer must be the derived buffer");
    assert_eq!(call(&mut fx.w, &m, &delegate_ix(KIND_ACH, 1, 30_000)), 0);
    let c = host::cpis();
    let programs: Vec<[u8; 32]> = c.iter().map(|x| x.program).collect();
    assert_eq!(programs, vec![SYSTEM_PROGRAM, SYSTEM_PROGRAM, DELEGATION_PROGRAM]);
    assert_eq!(fx.w[pda].owner, DELEGATION_PROGRAM, "assigned away");
    assert!(fx.w[pda].data.iter().all(|b| *b == 0), "the shard was moved to the buffer and cleared");
    assert_eq!(fx.w[buffer].data.len(), 0, "buffer closed");
    // delegation arguments: commit interval, two seeds (name and index), the validator
    let d = &c[2].data;
    assert_eq!(&d[..8], &[0u8; 8]);
    assert_eq!(rd32(d, 8), 30_000);
    assert_eq!(rd32(d, 12), 2);
    assert_eq!(rd32(d, 16), 8);
    assert_eq!(&d[20..28], ACH_SEED);
    assert_eq!(rd32(d, 28), 1);
    assert_eq!(d[32], 1);
    assert_eq!(d[33], 1);
    assert_eq!(&d[34..66], &VALIDATOR);
    assert_eq!(d.len(), 66);
}

#[test]
fn a_stranger_can_delegate_an_account_back_to_the_same_validator() {
    let mut fx = Fx::new();
    let pda = fx.rel[3];
    let buf_key = find_pda(&[b"buffer", &fx.w[pda].key], &PROG).0;
    let buffer = fx.add(buf_key, SYS, 0, vec![], false);
    let record = fx.add(pk(610), SYS, 0, vec![], false);
    let meta = fx.add(pk(611), SYS, 0, vec![], false);
    let stranger = fx.player(9);
    fx.w[stranger].lamports = RICH;
    let m = [
        (stranger, true, true),
        (pda, false, true),
        (fx.prog, false, false),
        (buffer, false, true),
        (record, false, true),
        (meta, false, true),
        (fx.deleg, false, false),
        (fx.sys, false, false),
    ];
    assert_eq!(call(&mut fx.w, &m, &delegate_ix(KIND_REL, 3, 30_000)), 0);
    assert_eq!(fx.w[pda].owner, DELEGATION_PROGRAM);
    let c = host::cpis();
    let d = &c[c.len() - 1].data;
    assert_eq!(c[c.len() - 1].program, DELEGATION_PROGRAM);
    assert_eq!(rd32(d, 8), 30_000);
    assert_eq!(&d[34..66], &VALIDATOR);
    assert_eq!(fx.w[stranger].lamports, RICH, "the buffer's rent went back to the stranger");
}

#[test]
fn undelegate_callback_restores_the_shard_from_the_buffer() {
    let mut fx = Fx::bare();
    let pda = fx.ach[2];
    let (ub, _) = find_pda(&[b"undelegate-buffer", &fx.w[pda].key], &DELEGATION_PROGRAM);
    let mut shard = ledger(KIND_ACH, 2);
    shard[HEADER + 5] = 0x77;
    let buffer = fx.add(ub, DELEGATION_PROGRAM, 0, shard.clone(), false);
    let payer = fx.player(8);
    fx.w[payer].lamports = 1_000_000_000;
    let mut ix = EXTERNAL_UNDELEGATE.to_vec();
    ix.extend_from_slice(&2u32.to_le_bytes());
    ix.extend_from_slice(&8u32.to_le_bytes());
    ix.extend_from_slice(ACH_SEED);
    ix.extend_from_slice(&1u32.to_le_bytes());
    ix.push(2);
    let m = [(pda, false, true), (buffer, true, false), (payer, true, true), (fx.sys, false, false)];
    // not signed by the delegation buffer, wrong buffer owner, wrong buffer address
    let mut bad = m;
    bad[1].1 = false;
    assert_eq!(call(&mut fx.w, &bad, &ix), ERR_SIGNER);
    fx.w[buffer].owner = ATTACKER;
    assert_eq!(call(&mut fx.w, &m, &ix), ERR_SIGNER);
    fx.w[buffer].owner = DELEGATION_PROGRAM;
    let mut other_ix = ix.clone();
    *other_ix.last_mut().unwrap() = 1;
    assert_eq!(call(&mut fx.w, &m, &other_ix), ERR_PDA, "seed index names a different shard");
    let mut bad_seed = ix.clone();
    bad_seed[16] = b'X';
    assert_eq!(call(&mut fx.w, &m, &bad_seed), ERR_DATA);
    assert_eq!(call(&mut fx.w, &m, &ix), 0);
    let a = &fx.w[pda];
    assert_eq!(a.owner, PROG);
    assert_eq!(a.data, shard);
    assert_eq!(a.lamports, rent_minimum(SHARD_SIZE));
    assert_eq!(fx.w[payer].lamports, 1_000_000_000 - rent_minimum(SHARD_SIZE));
    // the registry has a single seed
    let reg = fx.reg;
    let (ub, _) = find_pda(&[b"undelegate-buffer", &fx.w[reg].key], &DELEGATION_PROGRAM);
    let buf2 = fx.add(ub, DELEGATION_PROGRAM, 0, ledger(KIND_REG, 0), false);
    let mut ix = EXTERNAL_UNDELEGATE.to_vec();
    ix.extend_from_slice(&1u32.to_le_bytes());
    ix.extend_from_slice(&8u32.to_le_bytes());
    ix.extend_from_slice(REG_SEED);
    let m = [(reg, false, true), (buf2, true, false), (payer, true, true), (fx.sys, false, false)];
    assert_eq!(call(&mut fx.w, &m, &ix), 0);
    assert_eq!(fx.w[reg].data.len(), REG_SIZE);
}

#[test]
fn unknown_instructions_are_refused() {
    let mut fx = Fx::ready();
    let p = fx.player(1);
    assert_eq!(call(&mut fx.w, &[(p, true, false)], &[]), ERR_TAG);
    assert_eq!(call(&mut fx.w, &[(p, true, false)], &[9]), ERR_TAG);
    assert_eq!(call(&mut fx.w, &[(p, true, false)], &[200, 1, 2, 3]), ERR_TAG);
}

// ================================================================ fixture for the JavaScript reader
/// Writes the ledger after a fixed scenario so the JavaScript reader can be checked against the program's real byte
/// layout. Run with PLAY_EXPORT_FIXTURE=1. The addresses in the fixture come from the host's FAKE derivation and are not
/// real PDAs: the reader tests use this file for parsing only, and label it as a local fixture.
#[test]
fn export_reader_fixture() {
    if std::env::var("PLAY_EXPORT_FIXTURE").is_err() {
        return;
    }
    let mut fx = Fx::ready();
    host::set_clock(10, 1_800_000_000);
    let rnd = rnd_for(9_950, 0x31);
    let (sh, sl) = fx.earn(1, 0, 5, &rnd);
    let p1 = fx.player(1);
    assert_eq!(fx.transfer(p1, sh, sl as u16, &pk(2)), 0);
    fx.win(3, 1, 6);
    let p3 = fx.player(3);
    assert_eq!(fx.claim(p3, 1), 0);
    let mut s = String::from("{\n  \"label\": \"LOCAL FIXTURE exported by the PLAY host tests; addresses are fake; parsing checks only\",\n");
    s += &format!("  \"registry\": \"{}\",\n", hex(&fx.w[fx.reg].data));
    s += &format!("  \"achievements\": [{}],\n", (0..4).map(|i| format!("\"{}\"", hex(&fx.w[fx.ach[i]].data))).collect::<Vec<_>>().join(", "));
    s += &format!("  \"relics\": [{}],\n", (0..4).map(|i| format!("\"{}\"", hex(&fx.w[fx.rel[i]].data))).collect::<Vec<_>>().join(", "));
    s += &format!("  \"players\": {{\"p1\": \"{}\", \"p2\": \"{}\", \"p3\": \"{}\"}},\n", hex(&pk(1)), hex(&pk(2)), hex(&pk(3)));
    s += &format!("  \"relicId\": \"{}\",\n  \"relicShard\": {}, \"relicSlot\": {},\n  \"game\": \"{}\", \"gameProgram\": \"{}\"\n}}\n", hex(&rnd), sh, sl, hex(&GAME_MAGIC_OVER), hex(&GAME));
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/test/fixtures");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("ledger_fixture.json"), s).unwrap();
}

#[test]
fn no_player_instruction_moves_lamports_out_of_the_player() {
    // The part of AT1 a host test can show: claim, relic request, callback and transfer never debit a player and never call
    // the system program. It does NOT show who pays the transaction fee on the rollup (S3), which stays PENDING LIVE VERIFICATION.
    let mut fx = Fx::ready();
    let (p1, p2) = (fx.player(1), fx.player(2));
    fx.w[p1].lamports = 7_000;
    fx.w[p2].lamports = 11_000;
    host::clear_cpis();
    let rnd = rnd_for(100, 0x71);
    let (sh, sl) = fx.earn(1, 0, 5, &rnd);
    assert_eq!(fx.transfer(p1, sh, sl as u16, &pk(2)), 0);
    assert_eq!((fx.w[p1].lamports, fx.w[p2].lamports), (7_000, 11_000), "player balances are untouched");
    let calls = host::cpis();
    assert!(!calls.is_empty(), "the test must actually have exercised cross-program calls");
    for c in &calls {
        assert_ne!(c.program, SYSTEM_PROGRAM, "the program never calls the system program on a player's behalf");
        assert!(c.program == MAGIC_PROGRAM || c.program == VRF_PROGRAM, "only the rollup and VRF programs are called");
    }
}

// ---------------------------------------------------------------- settlement after a mint (post-kickoff upgrade design)
impl Fx {
    fn request_settle(&mut self, player: usize, shard: usize, slot: u16) -> u32 {
        let m = self.request_metas(player, shard);
        let mut ix = vec![8];
        ix.extend_from_slice(&slot.to_le_bytes());
        ix.push(1);
        call(&mut self.w, &m, &ix)
    }
    fn callback_metas_settle(&self, ach_shard: usize) -> Vec<M> {
        let mut m = self.callback_metas(true, ach_shard);
        m.push((self.magic, false, false));
        m.push((self.ctx, false, true));
        m.push((self.vault, false, true));
        m
    }
    fn commit_with(&mut self, payer: usize, acct: usize, data: &[u8]) -> u32 {
        let m = [(payer, true, true), (acct, false, true), (self.magic, false, false), (self.ctx, false, true), (self.vault, false, true)];
        call(&mut self.w, &m, data)
    }
    /// claim, request with the settle byte, and the callback with the save accounts; returns (shard, slot, callback result)
    fn earn_settled(&mut self, p: u16, rnd: &[u8; 32]) -> u32 {
        self.win(p, 0, 5);
        let pl = self.player(p);
        assert_eq!(self.claim(pl, 0), 0);
        let (sh, sl) = self.find_ach(p).unwrap();
        assert_eq!(self.request_settle(pl, sh, sl as u16), 0);
        let seq = rd32(&self.ach_slot(sh, sl), A_REQ_SEQ);
        let m = self.callback_metas_settle(sh);
        call(&mut self.w, &m, &Fx::callback_ix(rnd, sh as u8, sl as u16, seq))
    }
}

#[test]
fn request_with_the_settle_byte_names_the_save_accounts_for_the_callback() {
    let mut fx = Fx::ready();
    host::set_clock(777, 1_800_000_000);
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    assert_eq!(fx.request_settle(p, 0, 0), 0);
    let c = host::cpis();
    let v = c.iter().rev().find(|x| x.program == VRF_PROGRAM).expect("vrf request");
    let d = &v.data;
    assert_eq!(rd32(d, 84), 9, "six ledger accounts and three save accounts");
    let acct = |i: usize| &d[88 + i * 34..88 + (i + 1) * 34];
    assert_eq!(&acct(6)[..32], &MAGIC_PROGRAM);
    assert_eq!(acct(6)[33], 0);
    assert_eq!(&acct(7)[..32], &MAGIC_CONTEXT);
    assert_eq!(acct(7)[33], 1);
    assert_eq!(&acct(8)[..32], &fx.w[fx.vault].key);
    assert_eq!(acct(8)[33], 1);
    assert_eq!(rd32(d, 88 + 9 * 34), 7, "caller data unchanged");
}

#[test]
fn request_without_the_settle_byte_is_exactly_as_before_and_bad_bytes_are_refused() {
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    let m = fx.request_metas(p, 0);
    for bad in [vec![8u8, 0, 0, 2], vec![8, 0, 0, 1, 1], vec![8, 0]] {
        assert_eq!(call(&mut fx.w, &m, &bad), ERR_DATA, "{:?}", bad);
    }
    assert_eq!(fx.ach_slot(0, 0)[A_FLAGS], F_ELIGIBLE, "refused requests change nothing");
    assert_eq!(fx.request(p, 0, 0), 0);
    let c = host::cpis();
    let v = c.iter().rev().find(|x| x.program == VRF_PROGRAM).unwrap();
    assert_eq!(rd32(&v.data, 84), 6);
}

#[test]
fn callback_with_save_accounts_schedules_the_relic_shard_save() {
    let mut fx = Fx::ready();
    host::set_clock(10, 1_800_000_000);
    let rnd = rnd_for(9_950, 0x31);
    let before = host::cpis().len();
    assert_eq!(fx.earn_settled(1, &rnd), 0);
    let (sh, sl) = fx.find_relic(&rnd).expect("relic");
    assert_eq!((sh, sl), (2, 0));
    let r = &fx.w[fx.rel[sh]].data;
    assert_eq!(r[H_DIRTY], 0, "the relic shard is saved by the callback itself");
    assert_eq!(rd32(r, H_SAVED_TS), 1_800_000_000);
    assert_eq!(fx.w[fx.ach[0]].data[H_DIRTY], 1, "the achievement shard stays marked unsaved");
    let c = host::cpis();
    let last = c.last().unwrap();
    assert!(c.len() > before);
    assert_eq!(last.program, MAGIC_PROGRAM);
    assert_eq!(last.metas[0].0, fx.w[fx.rel[sh]].key, "the shard that was written, not another");
    assert_eq!(last.data, vec![1, 0, 0, 0]);
    // a later replay of the same callback cannot mint a second relic
    let seq = rd32(&fx.ach_slot(0, 0), A_REQ_SEQ);
    let m = fx.callback_metas_settle(0);
    assert_eq!(call(&mut fx.w, &m, &Fx::callback_ix(&rnd, 0, 0, seq)), ERR_NOT_PENDING);
}

#[test]
fn an_unfunded_relic_shard_still_mints_and_stays_recoverable() {
    let mut fx = Fx::ready();
    host::set_clock(10, 1_800_000_000);
    for i in 0..4 {
        let k = fx.rel[i];
        fx.w[k].lamports = 1_000;
    }
    let rnd = rnd_for(9_950, 0x31);
    assert_eq!(fx.earn_settled(1, &rnd), 0);
    let (sh, _) = fx.find_relic(&rnd).expect("relic");
    assert_eq!(fx.w[fx.rel[sh]].data[H_DIRTY], 1, "no reserve, no scheduled save, the mint stands");
    let stranger = fx.player(77);
    assert_eq!(fx.commit(stranger, fx.rel[sh]), ERR_SPONSOR_LOW);
    let k = fx.rel[sh];
    fx.w[k].lamports = RICH;
    assert_eq!(fx.commit(stranger, fx.rel[sh]), 0, "anyone can finish the save once it is funded");
}

#[test]
fn callback_refuses_wrong_save_accounts_before_writing_anything() {
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    assert_eq!(fx.request_settle(p, 0, 0), 0);
    let rnd = rnd_for(9_950, 0x31);
    let ix = Fx::callback_ix(&rnd, 0, 0, 1);
    let good = fx.callback_metas_settle(0);
    let mut swapped = good.clone();
    swapped.swap(8, 9);
    assert_eq!(call(&mut fx.w, &swapped, &ix), ERR_ACCOUNTS);
    let mut other_magic = good.clone();
    other_magic[7].0 = fx.sys;
    assert_eq!(call(&mut fx.w, &other_magic, &ix), ERR_ACCOUNTS);
    let mut fake_vault = good.clone();
    fake_vault[9].0 = fx.ctx;
    assert_eq!(call(&mut fx.w, &fake_vault, &ix), ERR_ACCOUNTS);
    assert_eq!(fx.ach_slot(0, 0)[A_FLAGS], F_PENDING, "still pending");
    assert!(fx.find_relic(&rnd).is_none(), "nothing minted");
    assert_eq!(call(&mut fx.w, &good, &ix), 0, "the genuine callback still works afterwards");
}

#[test]
fn the_seven_account_callback_of_an_old_request_still_mints_without_a_save() {
    let mut fx = Fx::ready();
    host::set_clock(10, 1_800_000_000);
    let rnd = rnd_for(9_950, 0x31);
    let before = host::cpis().len();
    let (sh, _) = fx.earn(1, 0, 5, &rnd);
    assert_eq!(fx.w[fx.rel[sh]].data[H_DIRTY], 1);
    assert_eq!(rd32(&fx.w[fx.rel[sh]].data, H_SAVED_TS), 0);
    // the only CPI since is the VRF request, no commit
    assert!(host::cpis().iter().skip(before).all(|x| x.program == VRF_PROGRAM));
}

#[test]
fn forced_commit_is_idempotent_rate_limited_and_permissionless() {
    let mut fx = Fx::ready();
    host::set_clock(10, 1_800_000_000);
    let rnd = rnd_for(9_950, 0x31);
    assert_eq!(fx.earn_settled(1, &rnd), 0);
    let (sh, _) = fx.find_relic(&rnd).unwrap();
    let rel = fx.rel[sh];
    let stranger = fx.player(77);
    let op = fx.op;
    // clean now: the plain commit is refused, a forced one is too soon
    assert_eq!(fx.commit_with(stranger, rel, &[2]), ERR_NOTHING_TO_SAVE);
    assert_eq!(fx.commit_with(stranger, rel, &[2, 1]), ERR_TOO_SOON);
    host::set_clock(11, 1_800_000_029);
    assert_eq!(fx.commit_with(op, rel, &[2, 1]), ERR_TOO_SOON);
    // after 30 seconds the operator may schedule the save again, and then must wait again
    let before = host::cpis().len();
    host::set_clock(12, 1_800_000_030);
    assert_eq!(fx.commit_with(stranger, rel, &[2, 1]), ERR_TOO_SOON, "anyone else waits a day");
    assert_eq!(fx.commit_with(op, rel, &[2, 1]), 0);
    assert_eq!(host::cpis().len(), before + 1);
    assert_eq!(rd32(&fx.w[rel].data, H_SAVED_TS), 1_800_000_030);
    assert_eq!(fx.commit_with(op, rel, &[2, 1]), ERR_TOO_SOON);
    // anyone may force a save a day after the last one, and then must wait again
    host::set_clock(13, 1_800_000_030 + 86_399);
    assert_eq!(fx.commit_with(stranger, rel, &[2, 1]), ERR_TOO_SOON);
    host::set_clock(14, 1_800_000_030 + 86_400);
    assert_eq!(fx.commit_with(stranger, rel, &[2, 1]), 0);
    assert_eq!(rd32(&fx.w[rel].data, H_SAVED_TS), (1_800_000_030 + 86_400) as u32);
    assert_eq!(fx.commit_with(stranger, rel, &[2, 1]), ERR_TOO_SOON);
    // the data of the account is untouched by saving again
    assert_eq!(fx.w[rel].data[H_DIRTY], 0);
    // malformed force bytes and lengths
    host::set_clock(15, 1_800_000_100 + 2 * 86_400);
    assert_eq!(fx.commit_with(stranger, rel, &[2, 2]), ERR_DATA);
    assert_eq!(fx.commit_with(stranger, rel, &[2, 1, 1]), ERR_ACCOUNTS);
    // a funded-but-forced save still needs the reserve
    fx.w[rel].lamports = 1_000;
    assert_eq!(fx.commit_with(stranger, rel, &[2, 1]), ERR_SPONSOR_LOW);
}

#[test]
fn a_dirty_account_is_saved_by_the_plain_commit_and_the_stamp_follows() {
    let mut fx = Fx::ready();
    host::set_clock(10, 1_800_000_500);
    let a = fx.ach[0];
    fx.w[a].lamports = 1_000;
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    assert_eq!(rd32(&fx.w[a].data, H_SAVED_TS), 0, "no save scheduled yet");
    fx.w[a].lamports = RICH;
    assert_eq!(fx.commit(p, a), 0);
    assert_eq!(rd32(&fx.w[a].data, H_SAVED_TS), 1_800_000_500);
}

// ---------------------------------------------------------------- sponsor as VRF payer (optional ninth account)
impl Fx {
    /// the request with a separate payer account: the player stays the identity and signs, the payer only pays
    fn request_paid_by(&mut self, player: usize, payer: usize, payer_signer: bool, payer_writable: bool, shard: usize, slot: u16, settle: bool) -> u32 {
        let mut m = self.request_metas(player, shard);
        m[0] = (player, true, false); // the player is only a signer here, not a writable account
        m.push((payer, payer_signer, payer_writable));
        let mut ix = vec![8];
        ix.extend_from_slice(&slot.to_le_bytes());
        if settle {
            ix.push(1);
        }
        call(&mut self.w, &m, &ix)
    }
}

#[test]
fn a_separate_payer_pays_the_vrf_call_and_the_player_stays_the_identity() {
    let mut fx = Fx::ready();
    host::set_clock(900, 1_800_000_000);
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    let sponsor = fx.player(9);
    assert_eq!(fx.request_paid_by(p, sponsor, true, true, 0, 0, true), 0);
    let c = host::cpis();
    let v = c.iter().rev().find(|x| x.program == VRF_PROGRAM).expect("vrf request");
    // (key, writable, signer): the first VRF account is the payer and it is the sponsor, not the player
    assert_eq!(v.metas[0].0, fx.w[sponsor].key, "the sponsor pays");
    assert!(v.metas[0].1 && v.metas[0].2);
    assert_ne!(v.metas[0].0, fx.w[p].key);
    // the achievement still belongs to the player: the request names the player's slot and the callback mints to the player
    assert_eq!(&fx.ach_slot(0, 0)[A_PLAYER..A_PLAYER + 32], &fx.w[p].key);
    let seq = rd32(&fx.ach_slot(0, 0), A_REQ_SEQ);
    let m = fx.callback_metas_settle(0);
    assert_eq!(call(&mut fx.w, &m, &Fx::callback_ix(&[3u8; 32], 0, 0, seq)), 0);
    let rel = (0..4).map(|i| fx.rel_slot(i, 0)).find(|s| s[L_STATE] == 1).expect("minted");
    assert_eq!(&rel[L_OWNER..L_OWNER + 32], &fx.w[p].key, "the relic belongs to the player, not the payer");
}

#[test]
fn a_separate_payer_cannot_stand_in_for_the_player() {
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    let sponsor = fx.player(9);
    let other = fx.player(2);
    // the payer must sign and be writable, or nothing changes
    assert_eq!(fx.request_paid_by(p, sponsor, false, true, 0, 0, false), ERR_SIGNER);
    assert_eq!(fx.request_paid_by(p, sponsor, true, false, 0, 0, false), ERR_SIGNER);
    assert_eq!(fx.ach_slot(0, 0)[A_FLAGS], F_ELIGIBLE, "refused requests change nothing");
    // a payer cannot request someone else's relic: the achievement must be the signing player's
    assert_eq!(fx.request_paid_by(other, sponsor, true, true, 0, 0, false), ERR_NOT_OWNER);
    // the player must still sign
    let mut m = fx.request_metas(p, 0);
    m[0] = (p, false, false);
    m.push((sponsor, true, true));
    assert_eq!(call(&mut fx.w, &m, &[8, 0, 0]), ERR_SIGNER);
    // the payer as the only signer does not identify anyone
    assert_eq!(fx.ach_slot(0, 0)[A_FLAGS], F_ELIGIBLE);
}

#[test]
fn eight_accounts_still_means_the_player_pays() {
    let mut fx = Fx::ready();
    fx.win(1, 0, 5);
    let p = fx.player(1);
    assert_eq!(fx.claim(p, 0), 0);
    assert_eq!(fx.request(p, 0, 0), 0);
    let c = host::cpis();
    let v = c.iter().rev().find(|x| x.program == VRF_PROGRAM).expect("vrf request");
    assert_eq!(v.metas[0].0, fx.w[p].key);
}

fn vault_page(fx: &mut Fx, player: usize, n: u32, prefund: u64) -> usize {
    let key = find_pda(&[vault::PAGE_SEED, &fx.w[player].key, &n.to_le_bytes()], &PROG).0;
    fx.add(key, SYS, prefund, vec![], false)
}

fn vault_open(fx: &mut Fx, player: usize, page: usize, n: u32, prev: Option<usize>) -> u32 {
    let mut ix = vec![11];
    ix.extend(n.to_le_bytes());
    let mut m = vec![(player, true, false), (fx.op, true, true), (page, false, true), (fx.sys, false, false)];
    if let Some(prev) = prev { m.push((prev, false, false)); }
    call(&mut fx.w, &m, &ix)
}

#[test]
fn vault_open_checks_seed_privileges_and_predecessor() {
    let mut fx = Fx::build(true);
    let player = fx.player(1);
    let p0 = vault_page(&mut fx, player, 0, 500_000);
    let p1 = vault_page(&mut fx, player, 1, 0);
    let large = vault_page(&mut fx, player, 256, 0);
    let prior = vault_page(&mut fx, player, 255, 0);
    assert_eq!(vault_open(&mut fx, player, p1, 0, None), ERR_PDA);
    assert_eq!(vault_open(&mut fx, player, p1, 1, Some(p0)), ERR_OWNER);
    let ix = [11, 0, 0, 0, 0];
    let mut metas = [(player, false, false), (fx.op, true, true), (p0, false, true), (fx.sys, false, false)];
    assert_eq!(call(&mut fx.w, &metas, &ix), ERR_SIGNER);
    metas[0].1 = true;
    metas[2].2 = false;
    assert_eq!(call(&mut fx.w, &metas, &ix), ERR_ACCOUNTS);
    assert_eq!(vault_open(&mut fx, player, p0, 0, None), 0);
    assert_eq!(fx.w[p0].lamports, rent_minimum(vault::PAGE_LEN));
    assert_eq!(vault_open(&mut fx, player, p0, 0, None), ERR_OWNER);
    assert_eq!(vault_open(&mut fx, player, p1, 1, Some(p0)), ERR_DATA);
    fx.w[p0].data[48..80].fill(7);
    fx.w[p0].data[80..112].fill(7);
    assert_eq!(vault_open(&mut fx, player, p1, 1, Some(p0)), ERR_DATA);
    fx.w[p0].data[80..112].fill(8);
    assert_eq!(vault_open(&mut fx, player, p1, 1, Some(p0)), 0);
    fx.w[prior].owner = PROG;
    fx.w[prior].data = fx.w[p0].data.clone();
    wr32(&mut fx.w[prior].data, 40, 255);
    assert_eq!(vault_open(&mut fx, player, large, 256, Some(prior)), 0);
    assert_eq!(rd32(&fx.w[large].data, 40), 256);
}

#[test]
fn vault_relocation_then_atomic_transfer_rejects_replay_and_full_page() {
    let mut fx = Fx::build(true);
    let alice = fx.player(1);
    let bob = fx.player(2);
    let a = vault_page(&mut fx, alice, 0, 0);
    let b = vault_page(&mut fx, bob, 0, 0);
    assert_eq!(vault_open(&mut fx, alice, a, 0, None), 0);
    assert_eq!(vault_open(&mut fx, bob, b, 0, None), 0);
    let id = [17; 32];
    let idx = fx.add(find_pda(&[vault::INDEX_SEED, &id], &PROG).0, SYS, 0, vec![], false);
    let legacy = fx.rel[0];
    let record = fx.add(find_pda(&[b"delegation", &fx.w[legacy].key], &DELEGATION_PROGRAM).0, SYS, 0, vec![], false);
    let owner = fx.w[alice].key;
    let recipient = fx.w[bob].key;
    let old = &mut fx.w[legacy].data[HEADER..HEADER + REL_SLOT];
    old[L_STATE] = 1;
    old[L_ID..L_ID + 32].copy_from_slice(&id);
    old[L_OWNER..L_OWNER + 32].copy_from_slice(&owner);
    let mut ix = vec![12, 0, 0, 0, 0, 0, 0, 0];
    ix.extend(id);
    let mut metas = vec![(alice, true, false), (fx.op, true, true), (a, false, true), (idx, false, true), (fx.sys, false, false), (record, false, false)];
    metas.extend(fx.rel.iter().enumerate().map(|(i, &x)| (x, false, i == 0)));
    fx.w[record].owner = DELEGATION_PROGRAM;
    assert_eq!(call(&mut fx.w, &metas, &ix), ERR_OWNER);
    fx.w[record].owner = SYS;
    let other = fx.rel[1];
    let duplicate = fx.w[legacy].data[HEADER..HEADER + REL_SLOT].to_vec();
    fx.w[other].data[HEADER..HEADER + REL_SLOT].copy_from_slice(&duplicate);
    assert_eq!(call(&mut fx.w, &metas, &ix), ERR_DATA);
    fx.w[other].data[HEADER..HEADER + REL_SLOT].fill(0);
    host::fail_cpi_to(Some(SYSTEM_PROGRAM));
    assert_eq!(call(&mut fx.w, &metas, &ix), ERR_CPI);
    assert_eq!(fx.w[legacy].data[HEADER], 1);
    assert!(fx.w[idx].data.is_empty());
    host::fail_cpi_to(None);
    assert_eq!(call(&mut fx.w, &metas, &ix), 0);
    assert_eq!(call(&mut fx.w, &metas, &ix), ERR_NOT_OWNER);
    assert_eq!(fx.w[legacy].data[HEADER], vault::MOVED);
    let transfer = |rev: u64, to: [u8;32]| {
        let mut x = vec![13, 0, 0, 0, 0, 0, 0, 0, 0];
        x.extend(rev.to_le_bytes());
        x.extend(id);
        x.extend(to);
        x
    };
    let forward = [(alice, true, false), (a, false, true), (b, false, true), (idx, false, true)];
    let reverse = [(bob, true, false), (b, false, true), (a, false, true), (idx, false, true)];
    fx.w[b].data[48..80].fill(7);
    fx.w[b].data[80..112].fill(8);
    assert_eq!(call(&mut fx.w, &forward, &transfer(0, recipient)), ERR_CAPACITY);
    assert_eq!(&fx.w[a].data[48..80], &id);
    fx.w[b].data[48..].fill(0);
    assert_eq!(call(&mut fx.w, &forward, &transfer(0, recipient)), 0);
    assert_eq!(&fx.w[b].data[48..80], &id);
    assert_eq!(call(&mut fx.w, &reverse, &transfer(1, owner)), 0);
    assert_eq!(call(&mut fx.w, &forward, &transfer(0, recipient)), ERR_NOT_OWNER);
    assert_eq!(&fx.w[a].data[48..80], &id);
    assert_eq!(rd64(&fx.w[idx].data, 16), 2);
}
