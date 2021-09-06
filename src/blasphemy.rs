use crate::service::Command;
use nanorand::{tls::TlsWyRand, Rng};
use ncurses::*;
use std::sync::mpsc::Receiver;

const INPUT_TIMEOUT: i32 = 16; // milliseconds
const WORD_MAXLEN: usize = 24;
const STARTING_LETTER_COUNT: usize = 14;

const LETTERS: [Letter; 31] = [
	Letter { c: 'A', points: 2 },
	Letter { c: 'A', points: 2 },
	Letter { c: 'B', points: 2 },
	Letter { c: 'C', points: 1 },
	Letter { c: 'D', points: 3 },
	Letter { c: 'E', points: 3 },
	Letter { c: 'E', points: 3 },
	Letter { c: 'F', points: 5 },
	Letter { c: 'G', points: 1 },
	Letter { c: 'H', points: 1 },
	Letter { c: 'I', points: 2 },
	Letter { c: 'I', points: 2 },
	Letter { c: 'J', points: 3 },
	Letter { c: 'K', points: 5 },
	Letter { c: 'L', points: 3 },
	Letter { c: 'M', points: 4 },
	Letter { c: 'N', points: 2 },
	Letter { c: 'O', points: 2 },
	Letter { c: 'O', points: 2 },
	Letter { c: 'P', points: 1 },
	Letter { c: 'Q', points: 4 },
	Letter { c: 'R', points: 2 },
	Letter { c: 'S', points: 1 },
	Letter { c: 'T', points: 4 },
	Letter { c: 'U', points: 2 },
	Letter { c: 'U', points: 2 },
	Letter { c: 'V', points: 5 },
	Letter { c: 'W', points: 4 },
	Letter { c: 'X', points: 3 },
	Letter { c: 'Y', points: 4 },
	Letter { c: 'Z', points: 2 },
];

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
	pub score: u32,
	rng: TlsWyRand,
}

#[derive(Clone, Copy)]
struct Letter {
	pub c: char,
	pub points: u32,
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
	command_rx: Receiver<Command>,
}

impl Blasphemy {
	pub fn new(command_rx: Receiver<Command>) -> Self {
		initscr();
		keypad(stdscr(), true);
		cbreak();
		timeout(INPUT_TIMEOUT);
		noecho();

		let mut b = Blasphemy {
			gamestate: Gamestate {
				word: String::with_capacity(WORD_MAXLEN),
				bank: Vec::with_capacity(STARTING_LETTER_COUNT),
				score: 0,
				rng: nanorand::tls_rng(),
			},
			term_size: Vector::new(),
			word_pos: Vector::new(),
			command_rx,
		};
		b.fill_bank();
		b
	}

	fn process_commands(&mut self) {
		match self.command_rx.try_recv() {
			Ok((replace, with)) => {
				if let Some(i) = self
					.gamestate
					.bank
					.iter()
					.position(|letter| letter.c == replace)
				{
					if let Some(l) = LETTERS.iter().find(|letter| letter.c == with) {
						let mut letter = l.clone();
						std::mem::swap(&mut self.gamestate.bank[i], &mut letter);
					}
				}
			}
			_ => (),
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
		if let WordQuality::Valid(points) = self.appraise_word() {
			for c in self.gamestate.word.chars() {
				if let Some(i) = self.gamestate.bank.iter().position(|letter| letter.c == c) {
					self.gamestate.bank.swap_remove(i);
				}
			}
			self.gamestate.score += points;
			self.gamestate.word.clear();
			self.fill_bank();
		}
	}

	fn fill_bank(&mut self) {
		while self.gamestate.bank.len() < STARTING_LETTER_COUNT {
			self.gamestate
				.bank
				.push(LETTERS[self.gamestate.rng.generate_range(0..LETTERS.len())]);
		}
	}

	fn appraise_word(&self) -> WordQuality {
		if self.gamestate.word.len() < 3 {
			return WordQuality::TooShort;
		}

		let mut available_letters = self.gamestate.bank.clone();
		let mut missing_letters = Vec::<char>::with_capacity(WORD_MAXLEN);
		let mut points = 0u32;
		for c in self.gamestate.word.chars() {
			match available_letters.iter().position(|letter| letter.c == c) {
				Some(i) => {
					let l = available_letters.swap_remove(i);
					points += l.points;
				}
				None => {
					missing_letters.push(c);
				}
			}
		}

		if !missing_letters.is_empty() {
			WordQuality::MissingLetters(missing_letters)
		} else if let Some(_) = webster::dictionary(&self.gamestate.word) {
			WordQuality::Valid(points)
		} else {
			WordQuality::Invalid
		}
	}

	fn draw(&mut self) {
		erase();

		getmaxyx(stdscr(), &mut self.term_size.y, &mut self.term_size.x);

		mvaddstr(0, 0, " [F4] quit\n");

		self.draw_score();
		self.draw_input_box();
		self.draw_message();
		self.draw_letter_bank();
		self.draw_viewer_instructions();
		self.draw_word();

		refresh();
	}

	fn draw_score(&self) {
		let text = format!("| score {:6} |", self.gamestate.score);

		let mut line_pos = Vector::new();
		getyx(stdscr(), &mut line_pos.y, &mut line_pos.x);

		line_pos.x = self.term_size.x - text.len() as i32 - 4;

		mvaddstr(line_pos.y, line_pos.x, &text);
	}

	fn draw_input_box(&mut self) {
		#[rustfmt::skip]
		const BOX: [&str; 5] = [
			"        type a word         ",
			"|==========================|",
			"|                          |",
			"|==========================|",
			" [Tab] clear [Enter] accept ",
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
			WordQuality::Valid(points) => format!("valid word! {} points", points),
		};

		let mut line_pos = Vector::new();
		getyx(stdscr(), &mut line_pos.y, &mut line_pos.x);

		line_pos.x = (self.term_size.x - message.len() as i32) / 2;
		line_pos.y += 1;

		mvaddstr(line_pos.y, line_pos.x, &message);
		addch('\n'.into());
	}

	fn draw_letter_bank(&self) {
		#[rustfmt::skip]
		const BOX: [&str; 4] = [
			"/----\\",
			"|    |",
			"|    |",
			"\\----/",
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
			mvaddstr(
				line_pos.y + 2,
				line_pos.x + 1,
				&format!("{:4}", letter.points),
			);
		}

		mv(line_pos.y + 4, 0);
	}

	fn draw_viewer_instructions(&self) {
		const INSTRUCTIONS: [&str; 5] = [
			"for viewers:",
			"to swap letters in the bank, type this into your browser:",
			"http://<server address>:8000/replace/<what>/with/<what>",
			"for example:",
			"http://127.0.0.1:8000/replace/a/with/z",
		];

		let mut line_pos = Vector::new();
		getyx(stdscr(), &mut line_pos.y, &mut line_pos.x);

		line_pos.y += 2;

		for s in INSTRUCTIONS {
			line_pos.x = (self.term_size.x - s.len() as i32) / 2;

			mvaddstr(line_pos.y, line_pos.x, s);
			line_pos.y += 1;
		}
		addch('\n'.into());
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

pub fn run(command_rx: Receiver<Command>) {
	let mut b = Blasphemy::new(command_rx);
	loop {
		b.process_commands();
		if b.input() {
			break;
		}
		b.draw();
	}
}
