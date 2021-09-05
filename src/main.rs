use std::thread;
use webster;

mod blasphemy;

fn main() {
	webster::preload();

	let ui_thread = thread::spawn(blasphemy::run);
	ui_thread.join().unwrap();
}
