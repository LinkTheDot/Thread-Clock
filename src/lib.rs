use anyhow::anyhow;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::{
  broadcast,
  broadcast::{Receiver, Sender},
};
use tokio::sync::{
  oneshot,
  oneshot::{Receiver as OneReceiver, Sender as OneSender},
};
use tokio::task::JoinHandle;
use tokio::time::Duration;

///The deafult tickrate in milliseconds that the clock runs at when [`Clock::new()`](crate::Clock::new()) is called.
pub const DEFAULT_TICKRATE: u32 = 24;

///A type for the time that the clock returns.
pub type Time = u64;

#[derive(Debug)]
/// The time receiver is a reduced part of the clock that can be passed into separate threads.
///
/// The time receiver can do anything the clock can except starting, stopping, and creating time
/// receivers.
///
/// Make sure no remaining time receivers make any calls after the clock has been stopped. This
/// will cause the thread they're in to be blocked.
///
/// # Creation
/// ```
///use thread_clock::Clock;
///use std::thread;
///
///let mut clock = Clock::new().unwrap();
///
///clock.start();
///
///let mut time_receiver = clock.spawn_receiver();
///
///let time = time_receiver.time();
///
///let final_time = clock.stop().unwrap();
///
///assert_eq!(final_time, time + 1);
/// ```
pub struct TimeReceiver {
  runtime: Arc<Runtime>,
  time_receiver: Receiver<Time>,
}

impl TimeReceiver {
  ///Waits for the next tick and returns the time.
  ///
  ///# Example
  ///
  ///```
  ///use thread_clock::Clock;
  ///
  ///let mut clock = Clock::new().unwrap();
  ///clock.start();
  ///
  ///let mut time_receiver = clock.spawn_receiver();
  ///
  ///let time = time_receiver.time();
  ///
  ///assert_eq!(time, 0);
  ///```
  pub fn time(&mut self) -> Time {
    Clock::get_time(&self.runtime, &mut self.time_receiver)
  }

  ///Waits for the next tick.
  ///
  ///# Example
  ///
  ///```
  ///use thread_clock::Clock;
  ///
  ///let mut clock = Clock::new().unwrap();
  ///clock.start();
  ///
  ///let mut time_receiver = clock.spawn_receiver();
  ///
  ///time_receiver.wait_for_tick();
  ///
  ///let time = time_receiver.time();
  ///
  ///assert_eq!(time, 1);
  ///```
  pub fn wait_for_tick(&mut self) {
    Clock::get_time(&self.runtime, &mut self.time_receiver);
  }

  ///Waits for the input amount of ticks.
  ///
  ///# Example
  ///
  ///```
  ///use thread_clock::Clock;
  ///
  ///let mut clock = Clock::new().unwrap();
  ///clock.start();
  ///
  ///let mut time_receiver = clock.spawn_receiver();
  ///
  ///time_receiver.wait_for_x_ticks(5);
  ///
  ///let time = time_receiver.time();
  ///
  ///assert_eq!(time, 5);
  ///```
  pub fn wait_for_x_ticks(&mut self, x: u32) {
    Clock::wait_for_ticks(&self.runtime, &mut self.time_receiver, x);
  }

  ///Waits until the imput time.
  ///
  ///If the time has already occurred then nothing will happen.
  ///
  ///# Example
  ///
  ///```
  ///use thread_clock::Clock;
  ///
  ///let mut clock = Clock::new().unwrap();
  ///clock.start();
  ///
  ///let mut time_receiver = clock.spawn_receiver();
  ///
  ///time_receiver.wait_for_time(9);
  ///
  ///let time = time_receiver.time();
  ///
  ///assert_eq!(time, 10);
  ///```
  pub fn wait_for_time(&mut self, time: Time) {
    Clock::wait_until(&self.runtime, &mut self.time_receiver, time);
  }
}

#[derive(Debug)]
///The clock can be started, stopped, and receive the current time.
///
///Using the clock is as simple as starting it and calling clock.time().
///
///Do not try using the clock before starting it as this will block the current thread it's in.
///
///# Usage
///
///```
///use thread_clock::Clock;
///
///let mut clock = Clock::new().unwrap();
///
///clock.start();
///
///let time = clock.time();
///
///let final_time = clock.stop().unwrap();
///
///assert_eq!(final_time, time + 1);
///```
pub struct Clock {
  runtime: Arc<Runtime>,
  clock_handle: Option<JoinHandle<()>>,
  clock_stopper: Option<OneSender<()>>,
  time_receiver: Receiver<Time>,
  clock_sender: Sender<Time>,
  tick_rate: u32,
}

impl Clock {
  ///Creates a new clock with a default tickrate of 24ms.
  ///
  ///If you want a custom tickrate, create the clock with [`Clock::custom`](crate::Clock::custom()) instead.
  ///
  ///Counting doesn't start until you call [`clock.start()`](crate::Clock::start()) on the clock.
  ///
  ///```
  ///use thread_clock::Clock;
  ///
  ///let mut clock = Clock::new().unwrap();
  ///
  ///clock.start();
  ///```
  pub fn new() -> anyhow::Result<Self> {
    Clock::new_clock(None)
  }

  ///Creates a new clock with a custom tickrate.
  ///
  ///# Example
  ///
  ///```
  ///use thread_clock::Clock;
  ///
  ///let mut clock = Clock::custom(10).unwrap();
  ///```
  pub fn custom(tick_rate: u32) -> anyhow::Result<Self> {
    Clock::new_clock(Some(tick_rate))
  }

  ///Creates a new clock.
  fn new_clock(tick_rate: Option<u32>) -> anyhow::Result<Self> {
    let runtime = Arc::new(Runtime::new()?);
    let clock_handle = None;
    let clock_stopper = None;
    let (clock_sender, time_receiver) = broadcast::channel::<Time>(1);
    let tick_rate = match tick_rate {
      Some(tick_rate) => tick_rate,
      None => DEFAULT_TICKRATE,
    };

    Ok(Clock {
      runtime,
      clock_handle,
      clock_stopper,
      time_receiver,
      clock_sender,
      tick_rate,
    })
  }

  ///Starts the clock.
  ///
  ///# Example
  ///```
  ///use thread_clock::Clock;
  ///
  ///let mut clock = Clock::new().unwrap();
  ///
  ///clock.start();
  ///```
  pub fn start(&mut self) {
    let (clock_stopper, stopper_receiver) = oneshot::channel();
    let handle = self.create_clock_thread(stopper_receiver);

    self.clock_handle = Some(handle);
    self.clock_stopper = Some(clock_stopper);
  }

  ///Stops the clock and returns the final time.
  ///
  ///If the clock hasn't been started yet an error will be returned.
  ///
  ///# Example
  ///```
  ///use thread_clock::Clock;
  ///
  ///let mut clock = Clock::new().unwrap();
  ///clock.start();
  ///
  ///let final_time = clock.stop().unwrap();
  ///
  ///assert_eq!(final_time, 0);
  ///```
  pub fn stop(mut self) -> anyhow::Result<Time> {
    match self.clock_stopper {
      Some(clock_stopper) => {
        let time = Self::get_time(&self.runtime, &mut self.time_receiver);

        let _ = clock_stopper.send(());

        Ok(time)
      }

      None => Err(anyhow!("The clock hasn't started.")),
    }
  }

  ///Waits for the next tick and returns the time.
  ///
  ///# Example
  ///
  ///```
  ///use thread_clock::Clock;
  ///
  ///let mut clock = Clock::new().unwrap();
  ///clock.start();
  ///
  ///let time = clock.time();
  ///
  ///assert_eq!(time, 0);
  ///```
  pub fn time(&mut self) -> Time {
    Self::get_time(&self.runtime, &mut self.time_receiver)
  }

  ///Waits for the next tick.
  ///
  ///# Example
  ///
  ///```
  ///use thread_clock::Clock;
  ///
  ///let mut clock = Clock::new().unwrap();
  ///clock.start();
  ///
  ///clock.wait_for_tick();
  ///
  ///let time = clock.time();
  ///
  ///assert_eq!(time, 1);
  ///```
  pub fn wait_for_tick(&mut self) {
    Self::get_time(&self.runtime, &mut self.time_receiver);
  }

  ///Waits for the input amount of ticks.
  ///
  ///# Example
  ///
  ///```
  ///use thread_clock::Clock;
  ///
  ///let mut clock = Clock::new().unwrap();
  ///clock.start();
  ///
  ///clock.wait_for_x_ticks(5);
  ///
  ///let time = clock.time();
  ///
  ///assert_eq!(time, 5);
  ///```
  pub fn wait_for_x_ticks(&mut self, x: u32) {
    Self::wait_for_ticks(&self.runtime, &mut self.time_receiver, x);
  }

  ///Waits until the imput time.
  ///
  ///If the time has already occurred then nothing will happen.
  ///
  ///# Example
  ///
  ///```
  ///use thread_clock::Clock;
  ///
  ///let mut clock = Clock::new().unwrap();
  ///clock.start();
  ///
  ///clock.wait_for_time(9);
  ///
  ///let time = clock.time();
  ///
  ///assert_eq!(time, 10);
  ///```
  pub fn wait_for_time(&mut self, time: Time) {
    Self::wait_until(&self.runtime, &mut self.time_receiver, time);
  }

  ///Creates a [`time receiver`](crate::TimeReceiver) which has every function the clock does except starting,
  ///stopping, and creating new time receivers.
  ///
  ///The time receiver can be passed into other threads.
  pub fn spawn_receiver(&self) -> TimeReceiver {
    TimeReceiver {
      runtime: Arc::clone(&self.runtime),
      time_receiver: self.clock_sender.subscribe(),
    }
  }

  fn create_clock_thread(&self, mut stopper_receiver: OneReceiver<()>) -> JoinHandle<()> {
    let time_sender = self.clock_sender.clone();
    let tick_rate = self.tick_rate.into();

    self.runtime.spawn(async move {
      let mut time = 0;

      while stopper_receiver.try_recv().is_err() {
        tokio::time::sleep(Duration::from_millis(tick_rate)).await;

        let _ = time_sender.send(time);

        time += 1;
      }
    })
  }

  // shared function split

  fn get_time(runtime: &Runtime, time_receiver: &mut Receiver<Time>) -> Time {
    let channel_was_empty = time_receiver.is_empty();
    let time = runtime.block_on(time_receiver.recv());

    if let (Ok(time), true) = (time, channel_was_empty) {
      time
    } else if !time_receiver.is_empty() {
      let _ = runtime.block_on(time_receiver.recv()); // clear excess data

      runtime.block_on(time_receiver.recv()).unwrap()
    } else {
      runtime.block_on(time_receiver.recv()).unwrap()
    }
  }

  fn wait_for_ticks(runtime: &Runtime, time_receiver: &mut Receiver<Time>, x: u32) {
    for _ in 0..x {
      Self::get_time(runtime, time_receiver);
    }
  }

  fn wait_until(runtime: &Runtime, time_receiver: &mut Receiver<Time>, wait_for_time: Time) {
    let current_time = Clock::get_time(runtime, time_receiver);

    if current_time < wait_for_time {
      let time_to_wait = wait_for_time - current_time;

      Self::wait_for_ticks(runtime, time_receiver, time_to_wait as u32);
    }
  }
}
