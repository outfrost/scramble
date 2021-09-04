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

struct Blasphemy {
	gamestate: Gamestate,
	term_size: Vector,
}

struct Gamestate {
	pub word: String,
}

impl Blasphemy {
	pub fn new() -> Self {
		initscr();
		keypad(stdscr(), true);
		cbreak();
		timeout(INPUT_TIMEOUT);
		noecho();

		Blasphemy {
			gamestate: Gamestate {
				word: String::with_capacity(24)
			},
			term_size: Vector { x: 0, y: 0 },
		}
	}

	fn input(&mut self) -> Input {
		let key = getch();
		if key == KEY_F4 {
			Input::Quit
		} else {
			Input::None
		}
	}

	fn draw(&mut self) {
		erase();

		getmaxyx(stdscr(), &mut self.term_size.y, &mut self.term_size.x);

		self.draw_input_box();

		refresh();
	}

	fn draw_input_box(&self) {
		const BOX: [&str; 4] = [
			"        type a word         ",
			"|==========================|",
			"|                          |",
			"|==========================|",
		];

		let mut line_pos = Vector { x: 0, y: 0 };
		getyx(stdscr(), &mut line_pos.y, &mut line_pos.x);

		line_pos.x = (self.term_size.x - BOX[0].len() as i32) / 2;
		line_pos.y += 2;

		for s in BOX {
			mvaddstr(line_pos.y, line_pos.x, s);
			line_pos.y += 1;
		}

		addstr("\n");
	}
}

impl Drop for Blasphemy {
	fn drop(&mut self) {
		endwin();
	}
}

pub fn run() {
	let mut b = Blasphemy::new();
	loop {
		match b.input() {
			Input::Quit => break,
			_ => (),
		}
		b.draw();
	}
}
