use std::thread;
use thread_clock::Clock;

#[cfg(test)]
mod clock {
  use super::*;

  #[test]
  fn counting_works() {
    let mut clock = Clock::custom(1)
      .unwrap_or_else(|error| panic!("An error has occurred while creating the clock: '{error}'"));
    let expected_final_time = 1001;

    clock.start();

    let mut previous_time = 0;
    clock.wait_for_tick();

    for _ in 0..1000 {
      let time = clock.time();

      println!("The time is {time}");

      assert!(time == previous_time + 1);

      previous_time = time;
    }

    let final_time = clock
      .stop()
      .unwrap_or_else(|error| panic!("An error has occurred while stopping the clock: '{error}'"));

    assert_eq!(final_time, expected_final_time);
  }

  #[test]
  fn wait_for_x_ticks_logic() {
    let mut clock = Clock::new()
      .unwrap_or_else(|error| panic!("An error has occurred while creating the clock: '{error}'"));
    let expected_final_time = 10;

    clock.start();
    clock.wait_for_x_ticks(10);

    let final_time = clock
      .stop()
      .unwrap_or_else(|error| panic!("An error has occurred while stopping the clock: '{error}'"));

    assert_eq!(expected_final_time, final_time);
  }

  #[test]
  fn wait_for_time_logic() {
    let mut clock = Clock::custom(1)
      .unwrap_or_else(|error| panic!("An error has occurred while creating the clock: '{error}'"));
    let expected_final_time = 11;

    clock.start();
    clock.wait_for_time(10);

    let final_time = clock
      .stop()
      .unwrap_or_else(|error| panic!("An error has occurred while stopping the clock: '{error}'"));

    assert_eq!(expected_final_time, final_time);
  }

  #[test]
  fn stop_clock_before_starting() {
    let clock = Clock::new()
      .unwrap_or_else(|error| panic!("An error has occurred while creating the clock: '{error}'"));

    let final_time = clock.stop();

    assert!(final_time.is_err());
  }

  #[test]
  fn filled_channel_no_lagged_error() {
    let mut clock = Clock::custom(1)
      .unwrap_or_else(|error| panic!("An error has occurred while creating the clock: '{error}'"));

    let expected_final_time = 1;

    clock.start();

    let mut time_receiver = clock.spawn_receiver();

    time_receiver.wait_for_tick();

    let final_time = clock
      .stop()
      .unwrap_or_else(|error| panic!("An error has occurred while stopping the clock: '{error}'"));

    assert_eq!(expected_final_time, final_time);
  }
}

#[cfg(test)]
mod time_receiver {
  use super::*;

  #[test]
  fn time_receiver_works() {
    let mut clock = Clock::custom(1)
      .unwrap_or_else(|error| panic!("An error has occurred while creating the clock: '{error}'"));

    let expected_final_time = 1001;
    let mut previous_time = 0;
    let mut time_receiver = clock.spawn_receiver();

    clock.start();
    clock.wait_for_tick();

    let _ = thread::spawn(move || {
      let mut previous_time = 0;

      for _ in 0..1000 {
        let time = time_receiver.time();

        assert!(time == previous_time + 1);

        previous_time = time;
      }
    });

    for _ in 0..1000 {
      let time = clock.time();

      assert!(time == previous_time + 1);

      previous_time = time;
    }

    let final_time = clock
      .stop()
      .unwrap_or_else(|error| panic!("An error has occurred while stopping the clock: '{error}'"));

    assert_eq!(expected_final_time, final_time);
  }

  #[test]
  fn time_receiver_methods() {
    let mut clock = Clock::custom(1)
      .unwrap_or_else(|error| panic!("An error has occurred while creating the clock: '{error}'"));
    let expected_final_time = 19;

    clock.start();

    let mut time_receiver = clock.spawn_receiver();

    time_receiver.wait_for_time(10); // time = 10
    time_receiver.time(); // time = 11
    time_receiver.wait_for_tick(); // time = 12
    time_receiver.wait_for_x_ticks(5); // time = 17
    time_receiver.time(); // time = 18

    let final_time = clock
      .stop() // time = 19
      .unwrap_or_else(|error| panic!("An error has occurred while stopping the clock: '{error}'"));

    assert_eq!(expected_final_time, final_time);
  }
}
