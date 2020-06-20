# Compile me

1. Install rustup: https://rustup.rs/
2. Run: rustup install nightly
3. In this directory, run: rustup override set nightly
4. Run cargo build --release

# Usage

Warning: the game can take a few minutes to launch the first time. Thanks our >1500-lines shader for that.

Example:
```
cargo run --release -- game \
          --with-shadows \
          --resolution 2 \
          --view-distance 10 \
          --layout fr
```

Main game parameters:
* with-shadows: if present, active shadows (will be slower)
* resolution: if greater than 1, the game is run with a poorer quality
* view-distance: number of chunks seen in each direction
* layout: fr or us, main keyboard mapping
* world: world path to load
* flat: if presents, the map is flat
* seed: (number) world random seed; by default 0

# In game options

**Move** Z,Q,S,D (fr) or W,A,S,D (us)
**Break a block** Left click
**Place a block** Right click
**Toggle pathtracing** P
**Toggle ambient light** L
**Set sun position** K
**Do daylight cycle** N
**Sneak** Left-shift (the player will be slower but won't fall)
**Toggle sprint** Left-control (the player will be faster)
**Toggle fly mode** Double click on space

