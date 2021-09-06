use std::{sync::mpsc, thread};
use tokio::{runtime::Runtime, task::JoinHandle};

mod blasphemy;
mod service;

fn main() {
	webster::preload();

	let (command_tx, command_rx) = mpsc::channel();

	thread::spawn(|| {
		let runtime = Runtime::new().unwrap();
		runtime.block_on(service::run(command_tx));
	});

	let ui_thread = thread::spawn(move || blasphemy::run(command_rx));
	ui_thread.join().unwrap();
}
