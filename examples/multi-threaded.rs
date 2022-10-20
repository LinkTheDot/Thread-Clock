use rust_clock::Clock;
use std::thread;

fn main() {
  let mut clock = Clock::new().unwrap();

  clock.start();

  let mut time_receiver = clock.spawn_receiver();

  let handle = thread::spawn(move || {
    for _ in 0..5 {
      let time = time_receiver.time();

      println!("The time in thread is {time}");
    }
  });

  for _ in 0..5 {
    let time = clock.time();

    println!("The time in main is {time}");
  }

  let _ = handle.join();

  let final_time = clock.stop();

  println!("The final time was {final_time:?}");
}
