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
                        seaborn
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

                    (python3.withPackages (pp: with pp;[
                        numpy
                        requests
                        tqdm
                    ]))
                ];

                shellHook = ''
                    build() {
                        cargo build --color=always 2>&1 | less
                    }

                    micro-bench-1() {
                        rm -rf $ROOT/logs/micro/bench1/*
                        rm -rf $ROOT/logs/micro/base1/*
                        for i in {1..9}; do 
                            $ROOT/scripts/benchmarks/micro1-bench.sh 
                            $ROOT/scripts/benchmarks/micro1-baseline.sh 
                        done

                        notify-send -u critical \
                            "Micro Benchmark 1 Finished!"
                    }
                    micro-bench-2() {
                        rm -rf $ROOT/logs/micro/bench2/*
                        rm -rf $ROOT/logs/micro/base2/*
                        for i in {1..9}; do 
                            $ROOT/scripts/benchmarks/micro2-bench.sh 
                            $ROOT/scripts/benchmarks/micro2-baseline.sh 
                        done
                        notify-send -u critical \
                            "Micro Benchmark 2 Finished!"
                    }
                    micro-bench-3() {
                        rm -rf $ROOT/logs/micro/bench3/*
                        rm -rf $ROOT/logs/micro/base3/*
                        for i in {1..9}; do 
                            $ROOT/scripts/benchmarks/micro3-bench.sh 
                            $ROOT/scripts/benchmarks/micro3-baseline.sh 
                        done
                        notify-send -u critical \
                            "Micro Benchmark 3 Finished!"
                    }

                    macro-bench1(){
                        rm -rf $ROOT/logs/macro/bench1/*
                        rm -rf $ROOT/logs/macro/base1/*
                        for i in {1..9}; do 
                            $ROOT/scripts/benchmarks/macro/authdag.sh 
                            $ROOT/scripts/benchmarks/macro/matrix.sh 
                        done

                        notify-send -u critical \
                            "Macro Benchmark 1 Finished!"
                    }
                    macro-bench2(){
                        rm -rf $ROOT/logs/macro/bench2/*
                        rm -rf $ROOT/logs/macro/base2/*
                        for i in {1..9}; do 
                            $ROOT/scripts/benchmarks/macro/stateless.sh 
                            $ROOT/scripts/benchmarks/macro/authdag.sh 
                        done

                        notify-send -u critical \
                            "Macro Benchmark 2 Finished!"
                    }
                    macro-bench3(){
                        rm -rf $ROOT/logs/macro/bench3/*
                        rm -rf $ROOT/logs/macro/base3/*
                        for i in {1..9}; do 
                            $ROOT/scripts/benchmarks/macro/authless.sh 
                            $ROOT/scripts/benchmarks/macro/authdag.sh 
                        done

                        notify-send -u critical \
                            "Macro Benchmark 3 Finished!"
                    }
                '';
            };
        };
    };
}
