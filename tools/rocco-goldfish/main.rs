// Goldfish for the-trouble-with-tribbles: how soon Rocco, Cabaretti Caterer ({X}{R}{G}{W})
// can be cast with X=3 (6 mana) or X=4 (7 mana). No opponents, greedy pilot, colours ignored
// (the deck's criteria file and Karsten own colour). Creature-only mana counts in full,
// because Rocco is a creature spell. Ramp is mapped by name in `role()`; an unknown card
// with a "Ramp" category panics so a new ramp card can't silently read as a blank.
use std::io::{self, BufRead};

#[derive(Clone, Copy, PartialEq, Debug)]
enum R {
    Land { tapped: bool },
    // one-drop that makes `n` more mana from next turn on
    Elf,
    // puts a land from hand onto the battlefield tapped when it enters (Elvish Pioneer)
    PioneerElf,
    // three-drop that makes `n` more mana from next turn on
    Dork3(u32),
    // three-drop that puts a tapped land onto the battlefield, `to_hand` also adds one to hand
    LandRamp3 { to_hand: bool },
    // three-drop that puts an untapped land onto the battlefield
    UntappedLand3,
    Blank,
}

fn role(name: &str, cats: &str) -> R {
    use R::*;
    match name {
        "Llanowar Elves" | "Elvish Mystic" | "Fyndhorn Elves" | "Birds of Paradise" | "Avacyn's Pilgrim"
        | "Boreal Druid" | "Delighted Halfling" | "Arbor Elf" | "Wild Growth" | "Utopia Sprawl" => Elf,
        "Elvish Pioneer" | "Arboreal Grazer" => PioneerElf,
        "Somberwald Sage" | "Llanowar Tribe" => Dork3(3),
        // Knotvine Mystic is {1},{T}: three mana, two net
        "Knotvine Mystic" | "Shaman of Forgotten Ways" | "Gwenna, Eyes of Gaea" | "Sage of the Maze"
        | "Faeburrow Elder" | "Transdimensional Bovine" | "Fyndhorn Elder" | "Greenweaver Druid"
        | "Nantuko Elder" | "Overgrowth" => Dork3(2),
        "Llanowar Visionary" | "Enduring Vitality" => Dork3(1),
        "Cultivate" | "Kodama's Reach" | "Flare of Cultivation" => LandRamp3 { to_hand: true },
        "Farhaven Elf" | "Simulacrum Shaper" | "They Went This Way" | "Hit It!" | "Topiary Stomper" => LandRamp3 { to_hand: false },
        "Wood Elves" => UntappedLand3,
        "Elegant Parlor" | "Lush Portico" | "Commercial District" | "Jetmir's Garden" => Land { tapped: true },
        _ if cats.contains("Land") => Land { tapped: false },
        _ if cats.contains("Ramp") => panic!("ramp card without a role: {name}"),
        _ => Blank,
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize { (self.next() % n as u64) as usize }
    fn shuffle<T>(&mut self, v: &mut [T]) { for i in (1..v.len()).rev() { let j = self.below(i + 1); v.swap(i, j) } }
}

const TURNS: usize = 7;
#[derive(Default)]
struct Out { x3: [f64; TURNS + 1], x4: [f64; TURNS + 1], mana: [f64; TURNS + 1] }

fn is_land(c: R) -> bool { matches!(c, R::Land { .. }) }

fn game(deck: &[R], rng: &mut Rng, o: &mut Out) {
    let mut lib = deck.to_vec();
    rng.shuffle(&mut lib);
    // keep 2-5 lands; Commander's first mulligan is free, then keep whatever comes
    if !(2..=5).contains(&lib[..7].iter().filter(|&&c| is_land(c)).count()) { rng.shuffle(&mut lib) }
    let mut hand: Vec<R> = lib.drain(..7).collect();
    let (mut lands, mut tapped_next, mut dorks, mut dorks_next) = (0u32, 0u32, 0u32, 0u32);
    let (mut hit6, mut hit7) = (false, false);
    for t in 1..=TURNS {
        // Commander: every player draws on turn one (CR 103.8 exempts multiplayer)
        if !lib.is_empty() { hand.push(lib.remove(0)) }
        lands += tapped_next; dorks += dorks_next; tapped_next = 0; dorks_next = 0;
        // land drop: an untapped land first, a tapped one only if that's all there is
        if let Some(i) = hand.iter().position(|&c| c == R::Land { tapped: false }) { hand.remove(i); lands += 1 }
        else if let Some(i) = hand.iter().position(|&c| c == R::Land { tapped: true }) { hand.remove(i); tapped_next += 1 }
        let mut mana = lands + dorks;
        o.mana[t] += mana as f64;
        if mana >= 6 { hit6 = true }
        if mana >= 7 { hit7 = true }
        if hit6 { o.x3[t] += 1.0 }
        if hit7 { o.x4[t] += 1.0 }
        // spend: the biggest three-drop first, then one-drops
        loop {
            let pick = hand.iter().enumerate().filter(|(_, c)| match c {
                R::Dork3(_) | R::LandRamp3 { .. } | R::UntappedLand3 => mana >= 3,
                R::Elf | R::PioneerElf => mana >= 1,
                _ => false,
            }).max_by_key(|(_, c)| match c { R::Dork3(n) => 10 + n, R::UntappedLand3 => 9, R::LandRamp3 { to_hand: true } => 8, R::LandRamp3 { .. } => 7, R::Elf => 5, R::PioneerElf => 4, _ => 0 }).map(|(i, _)| i);
            let Some(i) = pick else { break };
            match hand.remove(i) {
                R::Dork3(n) => { mana -= 3; dorks_next += n }
                R::LandRamp3 { to_hand } => { mana -= 3; tapped_next += 1; if to_hand { hand.push(R::Land { tapped: false }) } }
                R::UntappedLand3 => { mana -= 3; lands += 1; mana += 1 }
                R::Elf => { mana -= 1; dorks_next += 1 }
                R::PioneerElf => { mana -= 1;
                    if let Some(j) = hand.iter().position(|&c| is_land(c)) { hand.remove(j); tapped_next += 1 } }
                _ => unreachable!(),
            }
        }
    }
}

fn main() {
    let mut deck = vec![];
    for line in io::stdin().lock().lines() {
        let l = line.unwrap();
        let p: Vec<&str> = l.splitn(3, '|').collect();
        let q: usize = p[0].parse().unwrap();
        for _ in 0..q { deck.push(role(p[1], p[2])) }
    }
    let n = 200_000;
    let mut rng = Rng(42);
    let l = deck.iter().filter(|&&c| is_land(c)).count();
    let ramp = deck.iter().filter(|&&c| !is_land(c) && c != R::Blank).count();
    eprintln!("library {} lands {} ramp {} | {} games", deck.len(), l, ramp, n);
    let mut o = Out::default();
    for _ in 0..n { game(&deck, &mut rng, &mut o) }
    let f = n as f64;
    for t in 2..=TURNS {
        println!("T{t}: mean mana {:.2} | P(Rocco X>=3) {:5.1}% | P(Rocco X>=4) {:5.1}%", o.mana[t] / f, 100.0 * o.x3[t] / f, 100.0 * o.x4[t] / f)
    }
}
