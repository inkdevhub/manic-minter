# Manic Minter

[![workflow][a1]][a2] [![stack-exchange][s1]][s2] [![discord][d1]][d2] [![built-with-ink][i1]][i2] [![License][ap1]][ap2]

[s1]: https://img.shields.io/badge/click-white.svg?logo=StackExchange&label=ink!%20Support%20on%20StackExchange&labelColor=white&color=blue
[s2]: https://substrate.stackexchange.com/questions/tagged/ink?tab=Votes
[a1]: https://github.com/swanky-dapps/nft/actions/workflows/test.yml/badge.svg
[a2]: https://github.com/swanky-dapps/nft/actions/workflows/test.yml
[d1]: https://img.shields.io/discord/722223075629727774?style=flat-square&label=discord
[d2]: https://discord.gg/Z3nC9U4
[i1]: /.images/ink.svg
[i2]: https://github.com/paritytech/ink
[ap1]: https://img.shields.io/badge/License-Apache%202.0-blue.svg
[ap2]: https://opensource.org/licenses/Apache-2.0

This is an example project using ink! smart contract. It utilizes the cross contract call and PSP22 standard to create a fungible project.

### Purpose
This is an unaudited project template.
It explains how the cross contract calls are done in ink! and how to use the PSP22 standard.
The test is written in ink_e2e environment which testing cross contract calls.
### License
Apache 2.0

### 🏗️ How to use - Contracts
##### 💫 Build
- Use this [instructions](https://use.ink/getting-started/setup) to setup your ink!/Rust environment

```sh
cargo contract build --release
```

##### 💫 Run unit test

```sh
cargo test
```
##### 💫 Deploy
First start your local node. Recommended [swanky-node](https://github.com/AstarNetwork/swanky-node) v0.13.0
```sh
./target/release/swanky-node --dev --tmp -lruntime=trace -lruntime::contracts=debug -lerror
```
- or deploy with polkadot.JS. Instructions on [Astar docs](https://docs.astar.network/docs/wasm/sc-dev/polkadotjs-ui)

##### 💫 Run integration test
Define environment variable CONTRAC_NODE to point to the path where you have installation of [swanky-node](https://github.com/AstarNetwork/swanky-node) or where you have any other node which supports pallet-contract
And then:
```sh
cargo test --features e2e-tests
```


#### 📚 Learn
Follow the [From Zero to ink! Hero](https://docs.astar.network/docs/build/wasm/from-zero-to-ink-hero/) tutorial to learn how to build this smart contract.
