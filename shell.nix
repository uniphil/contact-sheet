with import <nixpkgs> {};

stdenv.mkDerivation {
    name = "contacts";
    buildInputs = [
        openssl
        postgresql
        rustChannels.nightly.cargo
        rustChannels.nightly.rust
    ];
    LD_LIBRARY_PATH="${postgresql}/lib";
    shellHook = ''
        export OPENSSL_DIR="${openssl.dev}"
        export OPENSSL_LIB_DIR="${openssl.out}/lib"
        export PATH="$PWD/bin:$PATH"
        export DATABASE_URL="postgres://postgres@localhost/contacts"
        if ! type diesel > /dev/null 2> /dev/null; then
          cargo install diesel_cli --no-default-features --features postgres --root $PWD
        fi
        diesel setup
    '';
}
