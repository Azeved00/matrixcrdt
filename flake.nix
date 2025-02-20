{
    description = "Authenticated CRDTs";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
        rust-overlay.url = "github:oxalica/rust-overlay";
	};

    outputs = { self, rust-overlay, ... } @ inputs: 
    let

        overlays = [ (import rust-overlay) ];
        pkgs = import inputs.nixpkgs { inherit system overlays; };
        ROOT = let p = builtins.getEnv "PWD"; in if p == "" then self else p;
        system = "x86_64-linux";

        rustVersion = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        });

    in {
        devShells."${system}" = {
            rust-dev = pkgs.mkShell {
                inherit ROOT;
                name = "Rust Dev Env";

                buildInputs = with pkgs; [
                    #cargo rustc 
                    rustVersion 
                    openssl
                    pkg-config
                    killall
                    sqlite
                ];

                shellHook = ''
                    build() {
                        cargo build --color=always 2>&1 | less
                    }
                '';
            };
            js-dev = pkgs.mkShell {
                inherit ROOT;
                name = "JS Dev Env";

                buildInputs = with pkgs; [
                    nodejs_23
                    nodePackages.npm
                    typescript-language-server
                ];

                shellHook = '''';
            };

            run = pkgs.mkShell {
                inherit ROOT;
                name = "Run Env";

                buildInputs = with pkgs;[
                    nodejs_23
                    rustVersion
                ];

                shellHook = ''
                    build() {
                        cargo build --color=always 2>&1 | less
                    }
                '';
            };
        };
    };
}
