use ncurses::*;

const INPUT_TIMEOUT: i32 = 16; // milliseconds

enum Input {
	None,
	Quit,
}

pub fn run() {
	init();
	loop {
		match input() {
			Input::Quit => break,
			_ => (),
		}
		draw();
	}
    terminate();
}

fn init() {
	initscr();
	keypad(stdscr(), true);
	cbreak();
	timeout(INPUT_TIMEOUT);
}

fn terminate() {
	endwin();
}

fn input() -> Input {
    let key = getch();
    if key == KEY_F4 {
    	Input::Quit
    }
    else {
    	Input::None
    }
}

fn draw() {
	erase();
    addstr("Hello, world!");
    refresh();
}
