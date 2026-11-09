# rocco-goldfish

Goldfish for `decks/the-trouble-with-tribbles`: shuffles the list, plays a greedy ramp pilot for
seven turns, and reports how often Rocco, Cabaretti Caterer ({X}{R}{G}{W}) is castable at X=3
(6 mana) and X=4 (7 mana) by each turn. Colours are ignored (the deck's criteria file owns
those). Creature-only mana counts in full, since Rocco is a creature spell. Ramp is mapped to a
role by name in `role()`, and a "Ramp"-category card with no role panics.

It exists because gauntlet refuses `{X}` casts, and `can_cast` under a `[casting]` line reads
the mana left after the line's own ramp casts. Delete it once gauntlet can cast an X commander.

    nix shell nixpkgs#rustc -c rustc --edition 2021 -O -o /tmp/roccosim main.rs
    ./deck.sh path/to/the-trouble-with-tribbles.deck.toml | /tmp/roccosim

Not modelled: removal, wipes, colour, the commander tax, cards that draw.
