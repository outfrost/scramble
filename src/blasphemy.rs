use ncurses::*;

const INPUT_TIMEOUT: i32 = 16; // milliseconds

enum Input {
	None,
	Quit,
}

struct Vector {
	pub x: i32,
	pub y: i32,
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

	let mut term_size = Vector { x: 0, y: 0 };
	getmaxyx(stdscr(), &mut term_size.y, &mut term_size.x);

	draw_input_box(term_size);

    addstr("Hello, world!");
    refresh();
}

fn draw_input_box(term_size: Vector) {
	const BOX: [&str; 4] = [
		"        TYPE A WORD         ",
		"|==========================|",
		"|                          |",
		"|==========================|",
	];

	let mut line_pos = Vector { x: 0, y: 0 };
	getyx(stdscr(), &mut line_pos.y, &mut line_pos.x);

	line_pos.x = (term_size.x - BOX[0].len() as i32) / 2;
	line_pos.y += 2;

	for s in BOX {
		mvaddstr(line_pos.y, line_pos.x, s);
		line_pos.y += 1;
	}

	addstr("\n");
}
