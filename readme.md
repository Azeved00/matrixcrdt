# Authenticated CRDTs

An implementation of an authenticated CRDT.
A solution that extends CRDTs to environments with 
multiple Clients and multiple Replicas of the server.

---

## Table of Contents
- 

## Naming convention 

## How to Use
99% of the usecases are exposed through [invoke](https://www.pyinvoke.org/) tasks.

You can check the list of the tasks through:
```bash
invoke --list
```

And run a specific task through:
```bash
invoke macro.simple.authdag
```


Note that to run `matrix` binaries you need to initiate a synapse server,
this can be achieved through dockerfile.

Generate the initial configuration:
```bash
docker compose run --rm synapse-init
```

Start the server:
```bash
docker compose up -d
```

Finally to stop or remove the server:
```bash
```


### Dependencies

The following packages are dependencies of this project,
a nix flake with shells

#### For backend and frontend running
    cargo rustc 
    nodejs_24npm
    openssl
    pkg-config
    sqlite
    docker

#### For benchmarking and invoke tasks
    libnotify
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


