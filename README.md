# scramble

I am never touching ncurses again.

**scramble** is a scrabble-kind of thing, where Twitch chat can replace letters in the streamer's bank via HTTP, and the streamer has to create words. May involve thinking quickly enough for the letters to still be there to choose from.

## Building

```sh
cargo run
```

Requires `libncurses`. Tested with `libncurses6`, but 5 should theoretically work too.

## How to play

You have a bank of letters. Make words using those letters.

Every time you enter a valid word, you get points, and the letters you've used from your bank are replaced with new, random ones.

**Give your viewers** the address of your server. Tell them to send HTTP GET requests of the form:

```
http://<server address>:8000/replace/a/with/z
```

They can help you, or make you suffer :)

Make sure your terminal size is at least 90x29 or something like that. Enjoy!

## Notes

Feel free to look at the code on stream. Chances for increased suffering unknown.
