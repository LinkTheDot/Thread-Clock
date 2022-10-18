// #![allow(unused)]

use anyhow::anyhow;
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::{
  broadcast,
  broadcast::{Receiver, Sender},
};
use tokio::task::JoinHandle;
use tokio::sync::{oneshot, oneshot::{Sender as OneSender, Receiver as OneReceiver}};

pub const DEFAULT_TICKRATE: u32 = 24;

pub type Time = u32;
pub type ClockError = String;

pub struct MainClock {
  runtime: Runtime,
  clock_handle: Option<JoinHandle<()>>,
  clock_stopper: Option<OneSender<()>>,
  time_receiver: Receiver<Time>,
  clock_sender: Sender<Time>,
  tick_rate: u32,
}

impl MainClock {
  pub fn new() -> anyhow::Result<Self> {
    let runtime = Runtime::new()?;
    let clock_handle = None;
    let clock_stopper = None;
    let (clock_sender, time_receiver) = broadcast::channel::<u32>(1);
    let tick_rate = DEFAULT_TICKRATE;

    Ok(MainClock {
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
    let time = self.get_time();

    let _ = self.clock_stopper.ok_or_else(|| anyhow!("The clock hasn't started."))?.send(());

    Ok(time)
  }

  pub fn get_time(&mut self) -> Time {
    self.runtime.block_on(self.time_receiver.recv()).unwrap()


  }

  fn create_clock_thread(&self, mut stopper_receiver: OneReceiver<()>) -> JoinHandle<()> {
    let time_sender = self.clock_sender.clone();
    let tick_rate = self.tick_rate.into();

    let mut time = 0;

    self.runtime.spawn(async move {
      while stopper_receiver.try_recv().is_err() {
        thread::sleep(Duration::from_millis(tick_rate));

        let _ = time_sender.send(time);

        time += 1;
      }
    })
  }
}

