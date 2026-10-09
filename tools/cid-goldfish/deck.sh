#!/usr/bin/env bash
# .deck.toml -> qty|name|categories, commanders dropped (Sakashima is modelled as always available)
grep -E '^\s+\{ name' "$1" | grep -v '"Commander"' | sed -E 's/^\s+\{ name = "([^"]+)"(, qty = ([0-9]+))?, in = \[([^]]*)\].*/\3|\1|\4/; s/^\|/1|/'
