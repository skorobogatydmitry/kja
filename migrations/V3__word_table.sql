-- table with word ids
create table if not exists word (
    id integer primary key on conflict fail autoincrement
);

-- fill table with existing words
insert into word select distinct(word_id) as id from word_component where true on conflict do nothing;

-- re-bind translations FK

create table if not exists word_translation_new (
    id integer primary key on conflict fail autoincrement,
    word_id references word (id) on delete cascade on update cascade,
    translation varchar(200)
);

insert into word_translation_new select * from word_translation;
drop table word_translation;
alter table word_translation_new rename to word_translation;

-- bind components to word_id
create table if not exists word_component_new (
    word_id integer references word (id) on delete cascade on update cascade not null,
    word_position integer not null,
    kanji_reading_id references kanji_reading (id) on delete cascade on update cascade not null,
        primary key (word_id, word_position)
);


insert into word_component_new select word_id, word_position, kanji_reading_id from word_component;
drop table word_component;
alter table word_component_new rename to word_component;
