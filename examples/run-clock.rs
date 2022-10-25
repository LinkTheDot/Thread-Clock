use thread_clock::Clock;

fn main() {
  let mut clock = Clock::new().unwrap();

  clock.start();

  for _ in 0..5 {
    let time = clock.time();

    println!("The time is {time}");
  }

  // returns an error if the clock hasn't started
  let final_time = clock.stop().unwrap();

  println!("The final time was {final_time:?}");
}
