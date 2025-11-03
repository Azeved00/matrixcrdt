{
    description = "Authenticated CRDTs";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
        rust-overlay.url = "github:oxalica/rust-overlay";
	};

    outputs = { self, rust-overlay, nixpkgs, ... } @ inputs: 
    let
        overlays = [  ];
        pkgs = import inputs.nixpkgs { inherit system overlays; };
        ROOT = let p = builtins.getEnv "PWD"; in if p == "" then self else p;
        system = "x86_64-linux";
    in {
        devShells."${system}" = {
            dev = pkgs.mkShell {
                inherit ROOT;
                name = "Dev";

                buildInputs = with pkgs; [
                    cargo rustc
                ];

                shellHook = ''
            
                '';
            };

            run = pkgs.mkShell {
                inherit ROOT;
                name = "Run";
                PYTHONPATH=ROOT;

                buildInputs = with pkgs;[
                    cargo rustc 

                    nodejs_24
                    nodePackages.npm
                    libnotify
                    openssl
                    pkg-config
                    sqlite
                    unixtools.netstat

                    (python3.withPackages (pp: with pp;[
                        numpy
                        jinja2
                        scipy
                        invoke
                        requests
                        tqdm
                        pandas
                        matplotlib
                        seaborn
                    ]))
                ];

                shellHook = ''
                    build() {
                        cargo build --color=always 2>&1 | less
                    }
                '';
            };

            matrix-server = pkgs.mkShell{
                inherit ROOT;
                shellHook=''
                    alias matrix-start="docker compose up -d"
                    alias matrix-stop="docker compose stop"
                    alias matrix-clean="docker compose down"
                    alias matrix-generate="docker compose run --rm synapse-init"
                '';
            };
        };
    };
}
