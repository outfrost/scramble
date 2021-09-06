use nanorand::{tls::TlsWyRand, Rng};
use ncurses::*;

const INPUT_TIMEOUT: i32 = 16; // milliseconds
const WORD_MAXLEN: usize = 24;
const STARTING_LETTER_COUNT: usize = 12;

struct Vector {
	pub x: i32,
	pub y: i32,
}

impl Vector {
	pub fn new() -> Self {
		Vector { x: 0, y: 0 }
	}
}

struct Gamestate {
	pub word: String,
	pub bank: Vec<Letter>,
	rng: TlsWyRand,
}

#[derive(Clone)]
struct Letter {
	pub c: char,
}

enum WordQuality {
	TooShort,
	Invalid,
	MissingLetters(Vec<char>),
	Valid(u32),
}

struct Blasphemy {
	gamestate: Gamestate,
	term_size: Vector,
	word_pos: Vector,
}

impl Blasphemy {
	pub fn new() -> Self {
		initscr();
		keypad(stdscr(), true);
		cbreak();
		timeout(INPUT_TIMEOUT);
		noecho();

		let mut b = Blasphemy {
			gamestate: Gamestate {
				word: String::with_capacity(WORD_MAXLEN),
				bank: Vec::with_capacity(STARTING_LETTER_COUNT),
				rng: nanorand::tls_rng(),
			},
			term_size: Vector::new(),
			word_pos: Vector::new(),
		};
		b.fill_bank();
		b
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
				// Backspace
				self.gamestate.word.pop();
				false
			}
			0x09 | KEY_BTAB => {
				// Tab
				self.gamestate.word.clear();
				false
			}
			0x0a | 0x0d | KEY_ENTER => {
				// Enter
				self.accept_word();
				false
			}
			_ => false,
		}
	}

	fn accept_word(&mut self) {
		self.gamestate.word.clear();
	}

	fn fill_bank(&mut self) {
		while self.gamestate.bank.len() < STARTING_LETTER_COUNT {
			self.gamestate.bank.push(Letter {
				c: char::from_u32(self.gamestate.rng.generate_range(0x41..=0x5a)).unwrap(),
			});
		}
	}

	fn appraise_word(&self) -> WordQuality {
		if self.gamestate.word.len() < 3 {
			return WordQuality::TooShort;
		}

		let mut available_letters = self.gamestate.bank.clone();
		let mut missing_letters = Vec::<char>::with_capacity(WORD_MAXLEN);
		for c in self.gamestate.word.chars() {
			match available_letters.iter().position(|letter| letter.c == c) {
				Some(i) => {
					available_letters.swap_remove(i);
				}
				None => {
					missing_letters.push(c);
				}
			}
		}

		if !missing_letters.is_empty() {
			WordQuality::MissingLetters(missing_letters)
		}
		else if let Some(_) = webster::dictionary(&self.gamestate.word) {
			WordQuality::Valid(0)
		}
		else {
			WordQuality::Invalid
		}
	}

	fn draw(&mut self) {
		erase();

		getmaxyx(stdscr(), &mut self.term_size.y, &mut self.term_size.x);

		mvaddstr(0, 0, " [F4] quit\n");

		self.draw_input_box();
		self.draw_message();
		self.draw_letter_bank();

		self.draw_word();

		refresh();
	}

	fn draw_input_box(&mut self) {
		#[rustfmt::skip]
		const BOX: [&str; 5] = [
			"        type a word         ",
			"|==========================|",
			"|                          |",
			"|==========================|",
			"        [Tab] clear         ",
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

	fn draw_message(&self) {
		let message: String = match self.appraise_word() {
			WordQuality::TooShort => "type at least 3 letters".into(),
			WordQuality::Invalid => "that's not in my dictionary".into(),
			WordQuality::MissingLetters(vec) => {
				let mut list = String::with_capacity(64);
				for c in vec {
					list.push(c);
					list.push_str(", ");
				}
				format!("you're missing some letters: {}", list)
			}
			WordQuality::Valid(_) => "valid word!".into(),
		};

		let mut line_pos = Vector::new();
		getyx(stdscr(), &mut line_pos.y, &mut line_pos.x);

		line_pos.x = (self.term_size.x - message.len() as i32) / 2;
		line_pos.y += 2;

		mvaddstr(line_pos.y, line_pos.x, &message);
	}

	fn draw_letter_bank(&self) {
		#[rustfmt::skip]
		const BOX: [&str; 4] = [
			"|====|",
			"|    |",
			"|    |",
			"|====|",
		];

		let mut line_pos = Vector::new();
		getyx(stdscr(), &mut line_pos.y, &mut line_pos.x);

		line_pos.y += 2;

		let letters_per_row = (self.term_size.x - 20) / (BOX[0].len() + 4) as i32;

		let mut row = 1;
		for (i, letter) in self.gamestate.bank.iter().enumerate() {
			if i >= (row * letters_per_row) as usize {
				line_pos.y += 5;
				row += 1;
			}

			line_pos.x = match row % 2 {
				1 => 14,
				_ => 10,
			} + ((i as i32 % letters_per_row) * (BOX[0].len() + 4) as i32);

			let mut line_y = line_pos.y;
			for s in BOX {
				mvaddstr(line_y, line_pos.x, s);
				line_y += 1;
			}

			mvaddstr(line_pos.y + 1, line_pos.x + 2, &String::from(letter.c));
		}

		mv(line_pos.y + 4, 0);
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
