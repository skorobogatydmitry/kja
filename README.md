# Console application to learn 漢字 reading through chains of words

The application shows a (random) word and expects you to recollect its kanji's readings (furigana).
You can reveal the reading and translations to check yourself.  
If you click on / select a kanji or a reading, the app goes to the different word with the same kanji or reading.

## Adding a word

All the words are stored in database you can fill it through the adding screen or direct inserts.

## Internals
The app is backed by a sqlite database (see `migrations/`) stored in `~/.local/kja/database.sql` by default. this file is the only thing to save / share between devices.

The app works on android through termux.

### Not implemented / TODO
- Words database browsing screen
- Words deletions
- Shared remove database

### Queries

1. get word by ID
`select kanji, reading, word_position from word_component wc join kanji_reading kr on kr.id = wc.kanji_reading_id where word_id = 4;`

2. get word's translations
`select * from word_translation where word_id = 3;`

3. dump whole database
`sqlite3 ~/.local/share/kja/database.sqlite '.dump' > dump.sql`

4. refresh default database
```sh
sqlite3 ~/.local/share/kja/database.sqlite \
  '.dump --data-only word word_translation word_component kanji_reading' > res/default-database.sql
```
