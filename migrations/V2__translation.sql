create table if not exists word_translation (
    id integer primary key on conflict fail autoincrement,
    word_id references word_component (word_id) on delete cascade on update cascade,
    translation varchar(200)
);
