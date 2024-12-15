# Vector Commitments for Smart Contracts

## Requirements

- [Foundry](https://github.com/foundry-rs/foundry)
- [Rust](https://www.rust-lang.org/tools/install)
- [Python v3](https://www.python.org/downloads/)

## Setup

Clone the repository

Install packages and dependencies

```sh
make setup
```

## Develop

Create Solidity contracts binding for Rust

```sh
make bind
```

Build crates

```sh
make build

make build-release
```

Test

```sh
make test
```

## Run evaluation script

Performance evaluation scripts are located in `./scripts` folder. `.sh` files are for running Rust binary files built from `./app/bin`. `.py` files are for plotting the csv files into graph using matplotlib.
