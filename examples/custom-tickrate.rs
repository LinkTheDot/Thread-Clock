use rust_clock::Clock;

fn main() {
  // starts a clock with a tick_rate of 250ms
  let mut clock = Clock::custom(250).unwrap();

  clock.start();

  for _ in 0..5 {
    let time = clock.time();

    println!("The time is {time}");
  }

  let final_time = clock.stop();

  println!("The final time is {final_time:?}");
}
