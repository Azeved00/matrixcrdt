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
                name = "Rust Dev";

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
                name = "JS Dev";

                buildInputs = with pkgs; [
                    nodejs_23
                    nodePackages.npm
                    typescript-language-server
                ];

                shellHook = '''';
            };
            python = pkgs.mkShell {
                inherit ROOT;
                name = "graphing";

                buildInputs = with pkgs; [
                    (python3.withPackages (pp: with pp;[
                        pandas
                        matplotlib
                    ]))
                ];

                shellHook = ''
                    plot-box() {
                        pushd $ROOT/logs
                        python3 $ROOT/scripts/box_graph.py $1
                        popd
                    }
                    plot-line() {
                        pushd $ROOT/logs
                        python3 $ROOT/scripts/line_graph.py $1
                        popd
                    }
                '';
            };


            run = pkgs.mkShell {
                inherit ROOT;
                name = "running";

                buildInputs = with pkgs;[
                    nodejs_23
                    rustVersion
                    libnotify
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
