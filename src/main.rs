#![allow(dead_code)]

use rust_clock::Clock;
use std::thread;

struct Test {}

fn main() {
  Test::run_clock();

  Test::waiting_for_time();
}

impl Test {
  fn run_clock() {
    let mut clock = Clock::new().unwrap();

    clock.start();

    let mut previous_time = 0;
    clock.time();

    let mut clock_receiver = clock.spawn_receiver();

    let handle = thread::spawn(move || {
      for _ in 1..100 {
        println!("Time in other thread is: {}", clock_receiver.time());
      }
    });

    for _ in 0..100 {
      let time = clock.time();

      assert!(time == previous_time + 1);

      println!("Time in main thread is : {}", time);

      previous_time = time;
    }

    let final_time = clock.stop().unwrap();

    println!();
    println!("The final time is: {}", final_time);

    let _ = handle.join();
  }

  fn waiting_for_time() {
    let mut clock = Clock::new().unwrap();

    clock.start();

    let mut clock_receiver = clock.spawn_receiver();

    let handle = thread::spawn(move || {
      clock_receiver.wait_for_time(100);

      println!("Time is up in the handle");
    });

    for _ in 0..100 {
      let time = clock.time();

      println!("The time in {time}");
    }

    let _ = handle.join();

    let _ = clock.stop();
  }
}
