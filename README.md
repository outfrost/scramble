# scramble

I am never touching ncurses again.

**scramble** is a scrabble-kind of thing, where Twitch chat can replace letters in the streamer's bank via HTTP, and the streamer has to create words. May involve thinking quickly enough for the letters to still be there to choose from.

## Building

```sh
cargo run
```

## How to play

You have a bank of letters. Make words using those letters.

Every time you enter a valid word, you get points, and the letters you've used from your bank are replaced with new, random ones.

## Notes

Yes, please do look at the code on stream. Chances for increased suffering unknown.
