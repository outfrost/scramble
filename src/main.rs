use std::thread;

mod blasphemy;

fn main() {
	let ui_thread = thread::spawn(blasphemy::run);
	ui_thread.join().unwrap();
}
