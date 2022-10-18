#![allow(unused)]

use rust_clock::MainClock;

fn main() {
  run_clock();
}

fn run_clock() {
  let mut clock = MainClock::new().unwrap();

  clock.start();

  let mut previous_time = 0;
  clock.get_time();

  for _ in 0..100 {
    let time = clock.get_time();

    assert!(time == previous_time + 1);

    println!("The time is: '{}'", time);

    previous_time = time;
  }

  let final_time = clock.stop().unwrap();

  println!();
  println!("The final time is: '{}'", final_time);
}
