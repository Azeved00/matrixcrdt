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
                    build() {
                        cargo build --color=always 2>&1 | less
                    }
                '';
            };
        };
    };
}
