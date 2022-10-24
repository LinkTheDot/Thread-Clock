# Rust Clock


Rust Clock will allow you to create a clock which can give the
amount of ticks that have passed since it has started.

The purpose of this clock is to allow synchronization of actions
between threads as the clock can be cloned to run anywhere.
However using this for actual time tracking may be a bit iffy as
every tick has approximately a +1.4ms drift.

## Examples

### Using clock for time

```rust
use rust_clock::Clock;

fn main() {
  let mut clock = Clock::new().unwrap();

  clock.start();

  clock.wait_for_time(50);

  let time = clock.stop().unwrap();

  assert_eq!(time, 51);
}
```

### Using clock for thread synching

```rust
use rust_clock::Clock;
use std::thread;

fn main() {
  let mut clock = Clock::new().unwrap();

  clock.start();

  let mut time_receiver = clock.spawn_receiver();

  let handle = thread::spawn(move || {
    for _ in 0..5 {
      time_receiver.wait_for_tick();
    }

    let time = time_receiver.time();

    assert_eq!(time, 5);
  });

  for _ in 0..5 {
    clock.wait_for_tick();
  }

  let time = clock.time();

  assert_eq!(time, 5);

  let _ = handle.join();

  let final_time = clock.stop().unwrap();

  assert_eq!(final_time, 6);
}
```
