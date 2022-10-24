use anyhow::{anyhow, Context};
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

pub const DEFAULT_TICKRATE: u32 = 24;

pub type Time = u32;

#[derive(Debug)]
pub struct TimeReceiver {
  runtime: Arc<Runtime>,
  time_receiver: Receiver<Time>,
}

impl TimeReceiver {
  pub fn time(&mut self) -> Time {
    Clock::get_time(&self.runtime, &mut self.time_receiver)
  }

  pub fn wait_for_tick(&mut self) {
    Clock::get_time(&self.runtime, &mut self.time_receiver);
  }

  pub fn wait_for_x_ticks(&mut self, x: u32) {
    Clock::wait_for_ticks(&self.runtime, &mut self.time_receiver, x);
  }

  pub fn wait_for_time(&mut self, time: Time) {
    Clock::wait_until(&self.runtime, &mut self.time_receiver, time);
  }
}

#[derive(Debug)]
pub struct Clock {
  runtime: Arc<Runtime>,
  clock_handle: Option<JoinHandle<()>>,
  clock_stopper: Option<OneSender<()>>,
  time_receiver: Receiver<Time>,
  clock_sender: Sender<Time>,
  tick_rate: u32,
}

impl Clock {
  pub fn new() -> anyhow::Result<Self> {
    Clock::new_clock(None)
  }

  pub fn custom(tick_rate: u32) -> anyhow::Result<Self> {
    Clock::new_clock(Some(tick_rate))
  }

  fn new_clock(tick_rate: Option<u32>) -> anyhow::Result<Self> {
    let runtime = Arc::new(Runtime::new()?);
    let clock_handle = None;
    let clock_stopper = None;
    let (clock_sender, time_receiver) = broadcast::channel::<u32>(1);
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

  pub fn start(&mut self) {
    let (clock_stopper, stopper_receiver) = oneshot::channel();
    let handle = self.create_clock_thread(stopper_receiver);

    self.clock_handle = Some(handle);
    self.clock_stopper = Some(clock_stopper);
  }

  pub fn stop(mut self) -> anyhow::Result<Time> {
    if self.clock_stopper.is_some() {
      let time = Self::get_time(&self.runtime, &mut self.time_receiver);

      let _ = self
        .clock_stopper
        .context("The clock hasn't started.")?
        .send(());

      Ok(time)
    } else {
      Err(anyhow!("The clock hasn't started."))
    }
  }

  pub fn time(&mut self) -> Time {
    Self::get_time(&self.runtime, &mut self.time_receiver)
  }

  pub fn wait_for_tick(&mut self) {
    Self::get_time(&self.runtime, &mut self.time_receiver);
  }

  pub fn wait_for_x_ticks(&mut self, x: u32) {
    Self::wait_for_ticks(&self.runtime, &mut self.time_receiver, x);
  }

  pub fn wait_for_time(&mut self, time: Time) {
    Self::wait_until(&self.runtime, &mut self.time_receiver, time);
  }

  pub fn spawn_receiver(&self) -> TimeReceiver {
    TimeReceiver {
      runtime: Arc::clone(&self.runtime),
      time_receiver: self.clock_sender.subscribe(),
    }
  }

  // shared function split

  fn get_time(runtime: &Runtime, time_receiver: &mut Receiver<Time>) -> Time {
    let channel_was_empty = time_receiver.is_empty();
    let time = runtime.block_on(time_receiver.recv());

    if let (Ok(time), true) = (time, channel_was_empty) {
      time
    } else if time_receiver.is_empty() {
      runtime.block_on(time_receiver.recv()).unwrap()
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

      Self::wait_for_ticks(runtime, time_receiver, time_to_wait);
    }
  }

  // shared function split

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
}
