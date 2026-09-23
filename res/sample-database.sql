PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE refinery_schema_history(
             version int4 PRIMARY KEY,
             name VARCHAR(255),
             applied_on VARCHAR(255),
             checksum VARCHAR(255));
INSERT INTO refinery_schema_history VALUES(1,'tables','2026-09-23T10:59:54.006488548Z','9780694832710557874'),
  (2,'translation','2026-09-23T11:16:41.323472507Z','17652640622827435444');
CREATE TABLE kanji_reading (
    id integer primary key on conflict fail autoincrement,
    kanji char,
    reading varchar(30),
        unique (kanji, reading) on conflict fail
);
INSERT INTO kanji_reading VALUES(1,'時','とき'),
  (2,'時','どき'),
  (3,'時','ジ'),
  (4,'間','カン'),
  (5,'時','と'),
  (6,'計','ケイ'),
  (7,'当','トウ'),
  (8,'弁','ベン');
CREATE TABLE word_component (
    id integer primary key on conflict fail autoincrement,
    word_id integer,
    word_position integer,
    kanji_reading_id references kanji_reading (id) on delete cascade on update cascade,
        unique (word_id, word_position) on conflict fail
);
INSERT INTO word_component VALUES(1,1,1,3),
  (2,1,2,4),
  (3,2,1,5),
  (4,2,2,6),
  (5,3,1,7),
  (6,3,2,3),
  (7,4,1,8),
  (8,4,2,7);
CREATE TABLE word_translation (
    id integer primary key on conflict fail autoincrement,
    word_id references word_component (word_id) on delete cascade on update cascade,
    translation varchar(200)
);
INSERT INTO word_translation VALUES(1,1,'время'),
  (2,1,'час'),
  (3,1,'период'),
  (4,2,'часы (наручные, настенные, и т.д.)'),
  (5,3,'в то время'),
  (6,3,'в те дни'),
  (7,4,'японский обед (еда) в коробке');
PRAGMA writable_schema=ON;
CREATE TABLE IF NOT EXISTS sqlite_sequence(name,seq);
DELETE FROM sqlite_sequence;
INSERT INTO sqlite_sequence VALUES('kanji_reading',8),
  ('word_component',8),
  ('word_translation',7);
PRAGMA writable_schema=OFF;
COMMIT;
