use rust_clock::Clock;
use std::thread;

#[test]
fn counting_works() {
  let mut clock = Clock::custom(1).unwrap_or_else(|err| panic!("Couldn't create clock: '{err}'"));
  let mut previous_time = 0;

  clock.start();
  clock.wait_for_tick(); // waits so the time will be 1

  for _ in 0..2000 {
    let time = clock.time();

    assert!(time == previous_time + 1);

    previous_time = time;
  }

  clock
    .stop()
    .unwrap_or_else(|err| panic!("An error has occurred while stopping the clock: '{err}'"));
}

#[cfg(test)]
mod time_receiver {
  use super::*;

  #[test]
  fn logic_works() {
    let mut clock = Clock::new().unwrap_or_else(|err| panic!("Couldn't create clock: '{err}'"));

    clock.start();

    let mut time_receiver = clock.spawn_receiver();

    let handle = thread::spawn(move || {
      for _ in 0..5 {
        let _ = time_receiver.time();
      }
    });

    for _ in 0..5 {
      let _ = clock.time();
    }

    handle
      .join()
      .unwrap_or_else(|err| panic!("An error has occurred while joining the handle: '{err:?}'"));

    clock
      .stop()
      .unwrap_or_else(|err| panic!("An error has occurred while stopping the clock: '{err}'"));
  }

  #[test]
  fn creating_receiver_before_starting_clock() {
    let mut clock = Clock::new().unwrap_or_else(|err| panic!("Couldn't create clock: '{err}'"));
    let mut time_receiver = clock.spawn_receiver();

    clock.start();

    let time = time_receiver.time();

    let last_time = clock
      .stop()
      .unwrap_or_else(|err| panic!("An error has occurred while stopping the clock: '{err}'"));

    assert_eq!(last_time, time);
  }
}
