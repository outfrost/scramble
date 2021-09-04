use ncurses::*;

pub fn run() {
    initscr();
    addstr("Hello, world!");
    refresh();
    getch();
    endwin();
}
