{
    description = "Authenticated CRDTs";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
        rust-overlay.url = "github:oxalica/rust-overlay";
	};

    outputs = { self, rust-overlay, ... } @ inputs: 
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
                    rust-analyzer
                    typescript-language-server
                    (python3.withPackages (pp: with pp;[
                        python-lsp-server
                    ]))
                ];

                shellHook = ''
            
                '';
            };

            graph = pkgs.mkShell {
                inherit ROOT;
                name = "graphing";

                buildInputs = with pkgs; [
                    (python3.withPackages (pp: with pp;[
                        pandas
                        matplotlib
                        requests
                    ]))
                ];

                shellHook = ''
                    plot-box() {
                        pushd $ROOT/logs
                        python3 $ROOT/scripts/graphing/box_graph.py $1
                        popd
                    }
                    plot-line() {
                        pushd $ROOT/logs
                        python3 $ROOT/scripts/graphing/line_graph.py $1
                        popd
                    }
                '';
            };


            run = pkgs.mkShell {
                inherit ROOT;
                name = "Run";

                buildInputs = with pkgs;[
                    cargo rustc 
                    nodejs_23
                    nodePackages.npm
                    libnotify
                    openssl
                    pkg-config
                    sqlite
                ];

                shellHook = ''
                    build() {
                        cargo build --color=always 2>&1 | less
                    }

                    bench1() {
                        rm -rf $ROOT/logs/bench1/*
                        rm -rf $ROOT/logs/base1/*
                        for i in {1..9}; do 
                            $ROOT/benchmarks/bench1.sh 
                            $ROOT/benchmarks/baseline1.sh 
                        done

                        notify-send -u critical \
                            "Benchmark 1 Finished!"
                    }
                    bench2() {
                        rm -rf $ROOT/logs/bench2/*
                        rm -rf $ROOT/logs/base2/*
                        for i in {1..9}; do 
                            $ROOT/benchmarks/bench2.sh 
                            $ROOT/benchmarks/baseline2.sh 
                        done
                        notify-send -u critical \
                            "Benchmark 2 Finished!"
                    }
                    bench3() {
                        rm -rf $ROOT/logs/bench3/*
                        rm -rf $ROOT/logs/base3/*
                        for i in {1..9}; do 
                            $ROOT/benchmarks/bench3.sh 
                            $ROOT/benchmarks/baseline3.sh 
                        done
                        notify-send -u critical \
                            "Benchmark 3 Finished!"
                    }
                '';
            };
        };
    };
}
