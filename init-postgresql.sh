#!/usr/bin/env nix-shell
#! nix-shell -i bash -p postgresql

BLUE="$(printf '\033[0;34m')"
GREEN='\033[1;32m'
YELLOW='\033[1;33m'
NC="$(printf '\033[0m')"

if [[ ! -d dev ]]; then
  echo -e "''${YELLOW}Initializing postgresql database:''${NC}"
  initdb --encoding=unicode --no-locale -U postgres dev | sed "s/.*/  ''${BLUE}&''${NC}/"
fi;
