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
- `Ctrl + c` to quit

### Guide
background blocks variants
- blue: water, player will die of drown if not leave water in 10 continual steps
- black: barrier, player is unable to step on.
