# Application to learn 漢字 reading through chain of words

1. The application shows a word and expects you to recollect its reading
2. You can reveal the reading and check yourself
3. You can reveal word's translation and check it too
4. After revealing the reading, you can　choose any of the readings or the 漢字　-s to show another word with the same 漢字　or reading
5. All the words are stored in database you can fill through the settings menu

Other features:
- it's solely depends on the database's content, but 音読み　is expected to be in カタカナ and 訓読み　- in ひらがな.

# Not implemented among the above
- browsing database:
  - delete word

# Queries to browse database

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
