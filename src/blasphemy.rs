use ncurses::*;

const INPUT_TIMEOUT: i32 = 16; // milliseconds
const WORD_MAXLEN: usize = 24;

struct Vector {
	pub x: i32,
	pub y: i32,
}

impl Vector {
	pub fn new() -> Self {
		Vector { x: 0, y: 0 }
	}
}

struct Blasphemy {
	gamestate: Gamestate,
	term_size: Vector,
	word_pos: Vector,
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
				word: String::with_capacity(WORD_MAXLEN),
			},
			term_size: Vector::new(),
			word_pos: Vector::new(),
		}
	}

	fn input(&mut self) -> bool {
		let key = getch();
		match key {
			KEY_F4 => true, // quit
			0x41..=0x5a | 0x61..=0x7a => {
				// [A-Za-z]
				let c = char::from_u32(key as u32).unwrap().to_ascii_uppercase();
				if self.gamestate.word.len() < WORD_MAXLEN {
					self.gamestate.word.push(c);
				}
				false
			}
			0x7f | KEY_BACKSPACE => {
				self.gamestate.word.pop();
				false
			}
			_ => false,
		}
	}

	fn draw(&mut self) {
		erase();

		getmaxyx(stdscr(), &mut self.term_size.y, &mut self.term_size.x);

		self.draw_input_box();

		if let Some(_) = webster::dictionary(&self.gamestate.word) {
			addstr("That's a word!");
		}

		self.draw_word();

		refresh();
	}

	fn draw_input_box(&mut self) {
		const BOX: [&str; 4] = [
			"        type a word         ",
			"|==========================|",
			"|                          |",
			"|==========================|",
		];

		let mut line_pos = Vector::new();
		getyx(stdscr(), &mut line_pos.y, &mut line_pos.x);

		line_pos.x = (self.term_size.x - BOX[0].len() as i32) / 2;
		line_pos.y += 2;

		self.word_pos.x = line_pos.x + 2;
		self.word_pos.y = line_pos.y + 2;

		for s in BOX {
			mvaddstr(line_pos.y, line_pos.x, s);
			line_pos.y += 1;
		}

		addstr("\n");
	}

	fn draw_word(&self) {
		mvaddstr(self.word_pos.y, self.word_pos.x, &self.gamestate.word);
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
		if b.input() {
			break;
		}
		b.draw();
	}
}
