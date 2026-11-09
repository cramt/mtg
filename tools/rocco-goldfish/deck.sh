#!/usr/bin/env bash
# .deck.toml -> qty|name|categories via gauntlet's parser (resolves printing-only entries), commander dropped
gauntlet parse "$1" | jq -r '.[] | select([.categories[]?.name]|index("Commander")|not) | "\(.qty)|\(.name)|\([.categories[]?.name]|join(","))"'
