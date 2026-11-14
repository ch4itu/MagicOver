//! Host tests of the pure rules. `LUDO_TRACE=<file>` also writes random games (actions and the state after each) as
//! JSON lines, which tools/ludo-diff.js replays through the Freeflow page's own JavaScript and compares.
use super::*;

fn fresh(seated: u8) -> [u8; TABLE_SIZE] {
    let mut t = [0u8; TABLE_SIZE];
    t[0..8].copy_from_slice(MAGIC);
    reset_table(&mut t);
    t[T_PHASE] = PH_PLAYING;
    for s in 0..4 {
        if seated & (1 << s) != 0 {
            t[seat_at(s) + K_FLAGS] = 1;
            t[T_SEATED] |= 1 << s;
        }
    }
    t
}
fn put(t: &mut [u8], seat: usize, i: usize, v: u8) {
    t[seat_at(seat) + K_PIECES + i] = v;
}
fn pieces(t: &[u8], seat: usize) -> [u8; 4] {
    [0, 1, 2, 3].map(|i| piece(t, seat, i))
}
/// Position (0-50) a seat must use to stand on ring square r (the 52nd ring square, just behind its start, is not on
/// its way).
fn rel(seat: usize, r: u8) -> u8 {
    ((r as u16 + 52 - STARTS[seat] as u16) % 52) as u8
}

#[test]
fn entry_needs_a_six_and_lands_on_the_start_square() {
    let mut t = fresh(0b0011);
    for d in 1..=5 {
        assert!(!legal(&t, 0, 0, d));
    }
    assert!(legal(&t, 0, 0, 6));
    apply_move(&mut t, 0, 0, 6);
    assert_eq!(pieces(&t, 0), [0, 255, 255, 255]);
    assert_eq!(ring_square(0, 0), 0);
    assert_eq!(ring_square(1, 0), 13);
    assert_eq!(t[seat_at(0) + K_BONUS], 1); // a six earns a bonus roll
}

#[test]
fn finishing_needs_the_exact_count() {
    let mut t = fresh(0b0011);
    put(&mut t, 0, 0, 53);
    assert!(legal(&t, 0, 0, 3));
    assert!(!legal(&t, 0, 0, 4));
    apply_move(&mut t, 0, 0, 3);
    assert_eq!(piece(&t, 0, 0), HOME_POS);
    assert_eq!(t[seat_at(0) + K_HOME], 1);
    assert!(!legal(&t, 0, 0, 1)); // home tokens never move
}

#[test]
fn capture_sends_a_single_token_home_but_not_on_safe_squares() {
    let mut t = fresh(0b0011);
    // blue's single token on ring square 5 (not safe); red lands on it
    put(&mut t, 1, 2, rel(1, 5));
    put(&mut t, 0, 0, 3);
    assert!(legal(&t, 0, 0, 2));
    apply_move(&mut t, 0, 0, 2);
    assert_eq!(piece(&t, 1, 2), BASE_POS);
    assert_eq!(t[seat_at(0) + K_CAPTURES], 1);
    assert_eq!(t[seat_at(0) + K_BONUS], 1);
    // on safe square 8 the two share the square
    let mut t = fresh(0b0011);
    put(&mut t, 1, 0, rel(1, 8));
    put(&mut t, 0, 0, 5);
    apply_move(&mut t, 0, 0, 3);
    assert_eq!(piece(&t, 1, 0), rel(1, 8));
    assert_eq!(t[seat_at(0) + K_CAPTURES], 0);
    assert_eq!(t[seat_at(0) + K_BONUS], 0);
}

#[test]
fn a_block_of_two_stops_other_colours_passing_or_landing_but_not_its_own() {
    let mut t = fresh(0b0111);
    put(&mut t, 1, 0, rel(1, 10));
    put(&mut t, 1, 1, rel(1, 10));
    put(&mut t, 0, 0, 7);
    assert!(legal(&t, 0, 0, 2)); // to 9
    assert!(!legal(&t, 0, 0, 3)); // landing on 10
    assert!(!legal(&t, 0, 0, 5)); // passing 10
    // blue itself passes its own block
    put(&mut t, 1, 2, rel(1, 9));
    assert!(legal(&t, 1, 2, 4));
    // a pair on a safe square (a star or a start square) blocks nobody: every colour shares those squares
    let mut t = fresh(0b0011);
    put(&mut t, 1, 0, rel(1, 8));
    put(&mut t, 1, 1, rel(1, 8));
    put(&mut t, 0, 0, 6);
    assert!(legal(&t, 0, 0, 2)); // landing on the star
    assert!(legal(&t, 0, 0, 4)); // passing it
    let mut t = fresh(0b0011);
    put(&mut t, 1, 0, 0); // blue's two tokens on blue's start square (ring 13)
    put(&mut t, 1, 1, 0);
    put(&mut t, 0, 0, 10);
    assert!(legal(&t, 0, 0, 5)); // red passes it
    // the home lane is private: no block can stand there
    let mut t = fresh(0b0011);
    put(&mut t, 0, 0, 49);
    assert!(legal(&t, 0, 0, 6));
}

#[test]
fn three_sixes_in_a_row_forfeit_the_third() {
    let mut t = fresh(0b0011);
    put(&mut t, 0, 0, 10);
    apply_roll(&mut t, 0, 6);
    assert_eq!(t[seat_at(0) + K_WAIT], 6);
    apply_move(&mut t, 0, 0, 6);
    apply_roll(&mut t, 0, 6);
    apply_move(&mut t, 0, 0, 6);
    apply_roll(&mut t, 0, 6);
    assert_eq!(t[seat_at(0) + K_WAIT], 0);
    assert_eq!(t[seat_at(0) + K_SIXES], 0);
    assert_eq!(t[seat_at(0) + K_LAST], 6);
    assert_eq!(piece(&t, 0, 0), 22);
    // a non-six in between resets the count
    apply_roll(&mut t, 0, 6);
    apply_move(&mut t, 0, 0, 6);
    apply_roll(&mut t, 0, 2);
    assert_eq!(t[seat_at(0) + K_SIXES], 0);
}

#[test]
fn a_die_with_no_legal_move_is_not_kept() {
    let mut t = fresh(0b0011);
    apply_roll(&mut t, 0, 4); // all at base
    assert_eq!(t[seat_at(0) + K_WAIT], 0);
    assert_eq!(t[seat_at(0) + K_BONUS], 0);
    let ev = rd32(&t, T_EVENTS) - 1;
    assert_eq!(t[T_RING + (ev as usize % RING) * EVENT], E_NOMOVE);
}

#[test]
fn four_home_wins_and_ends_the_match() {
    let mut t = fresh(0b0011);
    for i in 0..3 {
        put(&mut t, 0, i, HOME_POS);
    }
    put(&mut t, 0, 3, HOME_POS - 1);
    assert!(apply_move(&mut t, 0, 3, 1));
    assert_eq!(t[T_PHASE], PH_FINISHED);
    assert_eq!(t[T_WINNER], 0);
    assert_eq!(t[seat_at(0) + K_BONUS], 0);
}

#[test]
fn leaving_mid_match_removes_the_colour_and_the_last_one_wins() {
    let mut t = fresh(0b0111);
    put(&mut t, 2, 0, 20);
    put(&mut t, 2, 1, HOME_POS);
    leave_seat(&mut t, 2);
    assert_eq!(pieces(&t, 2), [255, HOME_POS, 255, 255]);
    assert!(!in_play(&t, 2));
    assert_eq!(t[T_PHASE], PH_PLAYING);
    leave_seat(&mut t, 0);
    assert_eq!(t[T_PHASE], PH_FINISHED);
    assert_eq!(t[T_WINNER], 1);
}

#[test]
fn leaving_the_lobby_frees_the_seat() {
    let mut t = fresh(0b0011);
    t[T_PHASE] = PH_LOBBY;
    leave_seat(&mut t, 0);
    assert_eq!(t[T_SEATED], 0b0010);
    assert_eq!(t[seat_at(0) + K_FLAGS], 0);
    assert_eq!(t[T_PHASE], PH_LOBBY);
    leave_seat(&mut t, 1);
    assert_eq!(t[T_PHASE], PH_FREE);
}

#[test]
fn a_seat_plays_with_its_smaran_key_or_its_session_key_only() {
    let mut t = fresh(0);
    t[T_PHASE] = PH_LOBBY;
    let mut who = [0u8; 64];
    who[..32].copy_from_slice(&[7u8; 32]);
    who[32..].copy_from_slice(&[9u8; 32]);
    take_seat(&mut t, 2, &who, b"Ann");
    assert!(plays_for(&t, 2, &[7u8; 32]) && plays_for(&t, 2, &[9u8; 32]) && !plays_for(&t, 2, &[8u8; 32]));
    assert_eq!(seat_of(&t, &[7u8; 32]), Some(2));
    assert_eq!(seat_of(&t, &[9u8; 32]), None); // a session key never counts as the seat's owner
}

#[test]
fn tables_are_reclaimed_only_when_free_finished_a_while_or_abandoned() {
    let mut t = fresh(0b0011);
    wr64(&mut t, T_ACTIVE, 1000);
    assert!(!reclaimable(&t, 1000 + ABANDON - 1));
    assert!(reclaimable(&t, 1000 + ABANDON));
    t[T_PHASE] = PH_FINISHED;
    assert!(!reclaimable(&t, 1000 + FINISH_HOLD - 1));
    assert!(reclaimable(&t, 1000 + FINISH_HOLD));
    t[T_PHASE] = PH_FREE;
    assert!(reclaimable(&t, 0));
}

#[test]
fn the_event_ring_keeps_the_last_seven() {
    let mut t = fresh(0b0001);
    for k in 0..20u8 {
        event(&mut t, [E_ROLL, 0, k, 0, 0, 0, 0, 0]);
    }
    assert_eq!(rd32(&t, T_EVENTS), 20);
    for n in 13..20u32 {
        let at = T_RING + (n as usize % RING) * EVENT;
        assert_eq!(t[at + 2], n as u8);
    }
    assert!(T_RING + RING * EVENT <= TABLE_SIZE);
    assert!(K_REQ_AT + 4 <= SEAT && T_RING + RING * EVENT == TABLE_SIZE && K_NAME + 16 == K_PIECES);
}

#[test]
fn dice_from_vrf_bytes_are_exactly_uniform() {
    // every byte value below 252 maps to a face 42 times; 252..255 are skipped
    let mut count = [0u32; 7];
    for b in 0..=255u8 {
        let mut rnd = [255u8; 32];
        rnd[0] = b;
        rnd[1] = 0; // fallback for skipped values
        let d = die_from(&rnd);
        assert!((1..=6).contains(&d));
        if b < 252 {
            count[d as usize] += 1;
        }
    }
    assert_eq!(&count[1..], &[42; 6]);
    assert_eq!(die_from(&[252, 253, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), 2);
}

// ---------------------------------------------------------------- random games for the differential check
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn state_json(t: &[u8]) -> String {
    let mut s = String::from("{\"pieces\":[");
    for seat in 0..4 {
        s += &format!("[{}]", pieces(t, seat).map(|v| if v == BASE_POS { -1 } else { v as i32 }).map(|v| v.to_string()).join(","));
        if seat < 3 {
            s.push(',');
        }
    }
    let field = |k: usize| (0..4).map(|seat| t[seat_at(seat) + k].to_string()).collect::<Vec<_>>().join(",");
    s += &format!(
        "],\"waiting\":[{}],\"sixes\":[{}],\"bonus\":[{}],\"last\":[{}],\"captures\":[{}],\"over\":{},\"winner\":{}}}",
        field(K_WAIT),
        field(K_SIXES),
        field(K_BONUS),
        field(K_LAST),
        field(K_CAPTURES),
        t[T_PHASE] == PH_FINISHED,
        t[T_WINNER]
    );
    s
}

/// Plays `games` random four-colour games with skewed dice (more sixes, to reach triple sixes, entries and blocks).
fn random_games(games: u64, out: &mut Vec<String>) -> (u32, u32, u32, u32, u32) {
    let (mut caps, mut forfeits, mut passes, mut wins, mut blocked) = (0u32, 0u32, 0u32, 0u32, 0u32);
    for g in 0..games {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ (g + 1).wrapping_mul(0x2545_F491_4F6C_DD1D));
        let mut t = fresh(0b1111);
        let mut lines = vec![format!("{{\"game\":{g},\"start\":{}}}", state_json(&t))];
        for _ in 0..4000 {
            if t[T_PHASE] != PH_PLAYING {
                break;
            }
            let seat = rng.below(4) as usize;
            let o = seat_at(seat);
            let action;
            if t[o + K_WAIT] == 0 {
                let d = if rng.below(10) < 3 { 6 } else { 1 + rng.below(6) as u8 };
                let before = rd32(&t, T_EVENTS);
                apply_roll(&mut t, seat, d);
                let kind = t[T_RING + (before as usize % RING) * EVENT];
                if kind == E_FORFEIT {
                    forfeits += 1;
                }
                action = format!("{{\"a\":\"roll\",\"seat\":{seat},\"die\":{d}");
            } else {
                let d = t[o + K_WAIT];
                let opts: Vec<usize> = (0..4).filter(|&i| legal(&t, seat, i, d)).collect();
                for i in 0..4 {
                    if !legal(&t, seat, i, d) && piece(&t, seat, i) < LANE && piece(&t, seat, i) + d <= HOME_POS {
                        blocked += 1;
                    }
                }
                if opts.is_empty() {
                    t[o + K_WAIT] = 0;
                    passes += 1;
                    action = format!("{{\"a\":\"pass\",\"seat\":{seat}");
                } else {
                    let i = opts[rng.below(opts.len() as u64) as usize];
                    let c0 = t[o + K_CAPTURES];
                    if apply_move(&mut t, seat, i, d) {
                        wins += 1;
                    }
                    caps += (t[o + K_CAPTURES] - c0) as u32;
                    action = format!("{{\"a\":\"move\",\"seat\":{seat},\"token\":{i}");
                }
            }
            lines.push(format!("{action},\"state\":{}}}", state_json(&t)));
        }
        out.extend(lines);
    }
    (caps, forfeits, passes, wins, blocked)
}

#[test]
fn random_games_keep_every_invariant() {
    let mut out = Vec::new();
    let (caps, forfeits, passes, wins, blocked) = random_games(400, &mut out);
    println!("400 games: {caps} captures, {forfeits} forfeited third sixes, {passes} passes, {wins} wins, {blocked} block refusals");
    assert!(caps > 1000 && forfeits > 50 && wins > 300 && blocked > 100);
    if let Ok(path) = std::env::var("LUDO_TRACE") {
        std::fs::write(&path, out.join("\n") + "\n").unwrap();
        println!("trace written to {path}: {} lines", out.len());
    }
}
