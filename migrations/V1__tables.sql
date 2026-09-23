create table if not exists kanji_reading (
    id integer primary key on conflict fail autoincrement,
    kanji char,
    reading varchar(30),
        unique (kanji, reading) on conflict fail
);

create table if not exists word_component (
    id integer primary key on conflict fail autoincrement,
    word_id integer,
    word_position integer,
    kanji_reading_id references kanji_reading (id) on delete cascade on update cascade,
        unique (word_id, word_position) on conflict fail
);
