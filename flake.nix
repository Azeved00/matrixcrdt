{
    description = "Rust CRDTs";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
        rust-overlay.url = "github:oxalica/rust-overlay";
	};

    outputs = { self, rust-overlay, ... } @ inputs: 
    let

        overlays = [ (import rust-overlay) ];
        pkgs = import inputs.nixpkgs { inherit system overlays; };
        ROOT = let p = builtins.getEnv "PWD"; in if p == "" then self else p;
        name = "Rust CRDTs";
        system = "x86_64-linux";

        rustVersion = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        });
    in {
        devShells."${system}" = {
            default = pkgs.mkShell {
                inherit name ROOT;

                buildInputs = with pkgs; [
                    #cargo rustc 
                    rustVersion 

                    protobuf
                    openssl
                    pkg-config
                    killall
                    sqlite
                ];

                shellHook = ''
                    alias search_todo="find ${ROOT}/counter -name '*.rs' -print0 | xargs -0 grep -H -n \"todo\""
                    alias search_error="find ${ROOT}/counter -name '*.rs' -print0 | xargs -0 grep -H -n \"error!\""
                    alias todo="search_todo && search_error"

                    debug() {
                        RUST_LOG=info cargo run --bin $1
                    }
                    build() {
                        cargo build --bin $1 --color=always 2>&1 | less
                    }
                    bench() {
                        pushd $ROOT
                        nix develop .#python --impure -c bash -c "$ROOT/scripts/main.sh $1"
                        popd
                    }
                '';
            };

            python = pkgs.mkShell {
                inherit ROOT;
                name = "Python thingy";

                buildInputs = with pkgs; [
                    (python3.withPackages (pp: with pp;[
                        pandas
                        matplotlib
                        pyqtwebengine
                    ]))
                ];

                shellHook = '''';
            };
        };
    };
}
