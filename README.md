# wasm-pong

Implementation of Pong with local and online coop.
The game is modelled as a series of events, so that other players/observers can easily join a game by consuming the events.

# How to run

- Complete: `./run-server.sh`
- Dev: `./run-server.dev.sh`
  - For dev the rust server and Svelte client can be started manually to allow debugging

# Implementation

- Game engine is implemented in wasm/rust
- Web GUI is implemented in Svelte
- Web server is implemented in rust/hyper
- Game events are persisted in kafka
