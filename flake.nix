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
                CONTAINER_NAME="synapse";
                IMAGE_NAME="matrixdotorg/synapse:latest";
                VOLUME_NAME="synapse-data";
                PORT="8008";

                shell-hook=''
                    alias matrix-start="docker run -d \
                      --name $CONTAINER_NAME \
                      -v $VOLUME_NAME:/data \
                      -p $PORT:$PORT \
                      $IMAGE_NAME"

                    alias matrix-stop="docker stop $CONTAINER_NAME"

                    alias matrix-clean="docker rm -f $CONTAINER_NAME"

                    alias matrix-generate="docker run --rm -it \
                      -e SYNAPSE_SERVER_NAME=your.matrix.host \
                      -e SYNAPSE_REPORT_STATS=yes \
                      -v $VOLUME_NAME:/data \
                      $IMAGE_NAME generate"
                '';
            };
        };
    };
}
