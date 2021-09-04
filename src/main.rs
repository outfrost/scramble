use ncurses;

fn main() {
    ncurses::initscr();
    ncurses::addstr("Hello, world!");
    ncurses::refresh();
    ncurses::getch();
    ncurses::endwin();
}
