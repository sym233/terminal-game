# Terminal game demo

Write in rust, run in terminals

Current work: map render and player interact.

### Run
```sh
cd adventrures
cargo run
```
### Operation
- Arrow key to move player (displayed as ☻)
- `b` check bag, to see what you have picked up
- `Ctrl + c` to quit

### Guide
background blocks variants
- blue: water, player will die of drown if not leave water in 10 continual steps.
- black: barrier, player is unable to step on.
- sign: '⚑', player can read a message on it.
- object: displayed as a char, player can pick it up once step on it.

