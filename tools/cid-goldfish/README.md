# cid-goldfish

Goldfish for `decks/attack-of-the-cids`: shuffles the list, plays a greedy pilot for ten
turns, and reports Cids in the graveyard, Cids on the battlefield, and when a mass
reanimation resolves. Colours are ignored (the deck's `pips.criteria.toml` owns those);
cards are mapped to roles by name in `role()`.

It exists because gauntlet can't model cycling, Buried Alive, mill-half, upkeep triggers or
reanimation yet (cramt/progress-engine#136-#140). Delete it once those land.

    nix shell nixpkgs#rustc -c rustc -O -o /tmp/cidsim main.rs
    ./deck.sh ../../decks/attack-of-the-cids.deck.toml | /tmp/cidsim

`mill.sh <game> <n>` mills n cards in a `scryfall play` game, for playtesting.
