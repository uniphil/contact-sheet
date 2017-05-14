#!/usr/bin/env nix-shell
#! nix-shell -i bash -p postgresql rustChannels.nightly.cargo

cargo install diesel_cli --no-default-features --features postgres
