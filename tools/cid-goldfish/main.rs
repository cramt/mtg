// Goldfish for attack-of-the-cids: no opponents, greedy pilot, colours ignored
// (gauntlet's pips file owns colour). Counts Cids binned and Cids on the battlefield.
use std::io::{self, BufRead};

#[derive(Clone, Copy, PartialEq, Debug)]
enum R { Land, Rock, Cid, Supplier, Crab, Glimpse, Trauma, Doombot, Stillness, Buried, Looter,
         Discard, Fix, Servitude, Angel, Animate, Reanimate, LivingDeath, LegendOff, Draw2, Tutor, Entomb, Blank }

fn role(name: &str, cats: &str) -> R {
    use R::*;
    match name {
        "Cid, Timeless Artificer" => Cid,
        "Stitcher's Supplier" => Supplier, "Hedron Crab" => Crab,
        "Glimpse the Unthinkable" => Glimpse, "Traumatize" => Trauma,
        "Doombot Harbinger" | "Flayed One" => Doombot, "Stillness in Motion" | "Out of the Tombs" => Stillness,
        "Buried Alive" => Buried, "Likeness Looter" => Looter,
        "Bitter Triumph" | "Key to the Side-Door" => Discard,
        "Fix What's Broken" => Fix, "Immortal Servitude" => Servitude,
        "Angel of Glory's Rise" => Angel, "Animate Dead" | "Necromancy" => Animate,
        "Reanimate" => Reanimate, "Patriarch's Bidding" => LivingDeath, "Storm of Souls" => Servitude,
        "Mirror Box" | "Council of Reeds" => LegendOff,
        "Thopter Spy Network" | "Losheel, Clockwork Scholar" => Draw2,
        "Entomb" => Entomb,
        "Mystical Tutor" | "Diabolic Intent" | "Scheming Symmetry" => Tutor,
        _ if cats.contains("Land") => Land,
        _ if cats.contains("Ramp") => Rock,
        _ => Blank,
    }
}
fn cost(r: R) -> u32 { use R::*; match r {
    Land => 0, Rock => 2, Cid => 4, Supplier | Crab | Reanimate | Tutor | Entomb => 1,
    Glimpse | Stillness | Looter | Discard | Animate => 2, Doombot | Buried | LegendOff => 3,
    Trauma | LivingDeath => 5, Fix => 4, Servitude | Angel => 7, Draw2 => 3, Blank => 3 } }

struct Rng(u64);
impl Rng { fn next(&mut self) -> u64 { self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = self.0; z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB); z ^ (z >> 31) }
    fn below(&mut self, n: usize) -> usize { (self.next() % n as u64) as usize } }

#[derive(Default, Clone)] struct Out { yard: [f64; 11], bf: [f64; 11], bf3: [f64; 11], cycled: f64, reanim_turn_sum: f64, reanim_n: f64 }

fn game(deck: &[R], rng: &mut Rng, cyc: bool, o: &mut Out) {
    let mut lib = deck.to_vec();
    for i in (1..lib.len()).rev() { let j = rng.below(i + 1); lib.swap(i, j); }
    // mulligan: keep 2-5 lands once, else free mull to 7 (Commander) and keep
    let lands = lib[..7].iter().filter(|&&c| c == R::Land).count();
    if !(2..=5).contains(&lands) { for i in (1..lib.len()).rev() { let j = rng.below(i + 1); lib.swap(i, j); } }
    let mut hand: Vec<R> = lib.drain(..7).collect();
    let (mut lands_bf, mut rocks, mut yard_cid, mut bf_cid) = (0u32, 0u32, 0u32, 0u32);
    let (mut saka, mut legend_off, mut crabs, mut still, mut looter) = (false, false, 0u32, 0u32, false);
    let mut reanimated_at = 0usize; let mut angel_yard = false;
    let mill = |lib: &mut Vec<R>, n: usize, yard: &mut u32, ay: &mut bool| { for _ in 0..n.min(lib.len()) { match lib.remove(0) { R::Cid => *yard += 1, R::Angel => *ay = true, _ => {} } } };
    for t in 1..=10usize {
        if t > 1 { if let Some(c) = (!lib.is_empty()).then(|| lib.remove(0)) { hand.push(c) } }
        for _ in 0..still { mill(&mut lib, 3, &mut yard_cid, &mut angel_yard) }
        if looter && !lib.is_empty() { hand.push(lib.remove(0));
            if let Some(i) = hand.iter().position(|&c| c == R::Cid) { hand.remove(i); yard_cid += 1 }
            else if let Some(i) = hand.iter().position(|&c| c == R::Blank) { hand.remove(i); } }
        if let Some(i) = hand.iter().position(|&c| c == R::Land) { hand.remove(i); lands_bf += 1;
            for _ in 0..crabs { mill(&mut lib, 3, &mut yard_cid, &mut angel_yard) } }
        let mut mana = lands_bf + rocks;
        let mut new_rocks = 0;
        let off = |saka: bool, lo: bool| saka || lo;
        loop {
            let has = |h: &Vec<R>, r: R| h.iter().position(|&c| c == r);
            let mut cast = |h: &mut Vec<R>, i: usize, mana: &mut u32| { let r = h.remove(i); *mana -= cost(r); r };
            // 1. mass reanimation once there's a pile and the legend rule is off (or Sakashima fits too)
            let pile = yard_cid >= 3;
            let mut did = false;
            for r in [R::Fix, R::LivingDeath, R::Servitude, R::Angel] {
                if let Some(i) = has(&hand, r) { let need = cost(r) + if off(saka, legend_off) { 0 } else { 4 };
                    if pile && mana >= need { if !off(saka, legend_off) { saka = true; mana -= 4 }
                        cast(&mut hand, i, &mut mana); bf_cid += yard_cid; yard_cid = 0; did = true;
                        if reanimated_at == 0 { reanimated_at = t } break } } }
            for r in [R::Reanimate, R::Animate] { if let Some(i) = has(&hand, r) {
                if angel_yard && pile && off(saka, legend_off) && mana >= cost(r) { cast(&mut hand, i, &mut mana); angel_yard = false; bf_cid += yard_cid; yard_cid = 0; did = true; if reanimated_at == 0 { reanimated_at = t } break } } }
            if did { continue }
            if let Some(i) = has(&hand, R::Entomb) { if mana >= 1 && pile && !angel_yard { if let Some(j) = lib.iter().position(|&c| c == R::Angel) { cast(&mut hand, i, &mut mana); lib.remove(j); angel_yard = true; continue } } }
            if did { continue }
            if let Some(i) = has(&hand, R::Tutor) { if mana >= 1 && !hand.iter().any(|&c| matches!(c, R::Fix|R::Servitude|R::Angel|R::LivingDeath)) {
                if let Some(j) = lib.iter().position(|&c| c == R::Fix || c == R::LivingDeath) { cast(&mut hand, i, &mut mana); hand.push(lib.remove(j)); continue } } }
            if let Some(i) = has(&hand, R::Rock) { if mana >= 2 && t <= 5 { cast(&mut hand, i, &mut mana); new_rocks += 1; continue } }
            if !saka && mana >= 4 && t >= 3 { saka = true; mana -= 4; continue }
            for r in [R::Reanimate, R::Animate] { if let Some(i) = has(&hand, r) {
                if yard_cid >= 1 && off(saka, legend_off) && mana >= cost(r) { cast(&mut hand, i, &mut mana); yard_cid -= 1; bf_cid += 1; did = true; break } } }
            if did { continue }
            for r in [R::Supplier, R::Crab, R::Stillness, R::Glimpse, R::Buried, R::Looter, R::Doombot, R::Trauma, R::LegendOff, R::Discard, R::Draw2] {
                if let Some(i) = has(&hand, r) { if mana >= cost(r) {
                    if r == R::Discard && !hand.contains(&R::Cid) { continue }
                    cast(&mut hand, i, &mut mana); did = true;
                    match r { R::Supplier => mill(&mut lib, 3, &mut yard_cid, &mut angel_yard), R::Crab => crabs += 1,
                        R::Stillness => still += 1, R::Glimpse => mill(&mut lib, 10, &mut yard_cid, &mut angel_yard),
                        R::Doombot => mill(&mut lib, 4, &mut yard_cid, &mut angel_yard), R::Looter => looter = true,
                        R::Trauma => { let n = lib.len() / 2; mill(&mut lib, n, &mut yard_cid, &mut angel_yard) }
                        R::Buried => { for _ in 0..3 { if let Some(j) = lib.iter().position(|&c| c == R::Cid) { lib.remove(j); yard_cid += 1 } } }
                        R::LegendOff => legend_off = true,
                        R::Discard => { let j = hand.iter().position(|&c| c == R::Cid).unwrap(); hand.remove(j); yard_cid += 1 }
                        R::Draw2 => { for _ in 0..2 { if !lib.is_empty() { hand.push(lib.remove(0)) } } }
                        _ => {} }
                    break } } }
            if did { continue }
            if let Some(i) = has(&hand, R::Cid) {
                if mana >= 4 && (off(saka, legend_off) || bf_cid == 0) && !(cyc && yard_cid < 3) { cast(&mut hand, i, &mut mana); bf_cid += 1; continue }
                if mana >= 2 { hand.remove(i); mana -= 2; yard_cid += 1; o.cycled += 1.0;
                    if !lib.is_empty() { hand.push(lib.remove(0)) } continue } }
            break
        }
        rocks += new_rocks;
        o.yard[t] += yard_cid as f64; o.bf[t] += bf_cid as f64; if bf_cid >= 3 { o.bf3[t] += 1.0 }
    }
    if reanimated_at > 0 { o.reanim_turn_sum += reanimated_at as f64; o.reanim_n += 1.0 }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cyc = args.get(1).map(|s| s == "cyclefirst").unwrap_or(false);
    let mut deck = vec![];
    for line in io::stdin().lock().lines() { let l = line.unwrap(); let p: Vec<&str> = l.splitn(3, '|').collect();
        let q: usize = p[0].parse().unwrap(); for _ in 0..q { deck.push(role(p[1], p[2])) } }
    let n = 40000; let mut o = Out::default(); let mut rng = Rng(42);
    let lands: f64 = (0..20000).map(|_| { let mut d = deck.clone(); for i in (1..d.len()).rev() { let j = rng.below(i+1); d.swap(i,j) }
        d[..7].iter().filter(|&&c| c == R::Land).count() as f64 }).sum::<f64>() / 20000.0;
    let l = deck.iter().filter(|&&c| c == R::Land).count() as f64;
    eprintln!("library {} lands {} | opener lands mean {:.3} (hypergeometric {:.3})", deck.len(), l, lands, 7.0 * l / deck.len() as f64);
    for _ in 0..n { game(&deck, &mut rng, cyc, &mut o) }
    let f = n as f64;
    for t in [4, 5, 6, 7, 8] { println!("T{t}: Cids in yard {:.2}  on battlefield {:.2}  P(3+ on bf) {:.1}%", o.yard[t]/f, o.bf[t]/f, 100.0*o.bf3[t]/f) }
    println!("mass reanimation resolved in {:.1}% of games, mean turn {:.2}; cycled {:.2}/game", 100.0*o.reanim_n/f, o.reanim_turn_sum/o.reanim_n.max(1.0), o.cycled/f);
}
