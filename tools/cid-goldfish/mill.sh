#!/usr/bin/env bash
# mill.sh <game> <n>: move the top n cards of the library to the graveyard, print them
g=$1; n=$2
scryfall play peek "$n" --name "$g" 2>&1 | sed 1d | awk '{print $1}' | while read -r id; do
  scryfall play move "$id" graveyard --from library --name "$g" >/dev/null 2>&1 || echo "ERR $id"; echo "$id"; done | paste -sd' '
