PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE refinery_schema_history(
             version int4 PRIMARY KEY,
             name VARCHAR(255),
             applied_on VARCHAR(255),
             checksum VARCHAR(255));
INSERT INTO refinery_schema_history VALUES(1,'tables','2026-09-23T10:59:54.006488548Z','9780694832710557874');
INSERT INTO refinery_schema_history VALUES(2,'translation','2026-09-23T11:16:41.323472507Z','17652640622827435444');
CREATE TABLE kanji_reading (
    id integer primary key on conflict fail autoincrement,
    kanji char,
    reading varchar(30),
        unique (kanji, reading) on conflict fail
);
INSERT INTO kanji_reading VALUES(1,'時','とき');
INSERT INTO kanji_reading VALUES(2,'時','どき');
INSERT INTO kanji_reading VALUES(3,'時','ジ');
INSERT INTO kanji_reading VALUES(4,'間','カン');
INSERT INTO kanji_reading VALUES(5,'時','と');
INSERT INTO kanji_reading VALUES(6,'計','ケイ');
INSERT INTO kanji_reading VALUES(7,'当','トウ');
INSERT INTO kanji_reading VALUES(8,'弁','ベン');
INSERT INTO kanji_reading VALUES(9,'日','ニチ');
INSERT INTO kanji_reading VALUES(10,'日','ジツ');
INSERT INTO kanji_reading VALUES(11,'日','ひ');
INSERT INTO kanji_reading VALUES(12,'日','び');
INSERT INTO kanji_reading VALUES(13,'日','か');
INSERT INTO kanji_reading VALUES(14,'本','ホン');
INSERT INTO kanji_reading VALUES(15,'本','もと');
INSERT INTO kanji_reading VALUES(16,'語','ゴ');
INSERT INTO kanji_reading VALUES(17,'語','かたる');
INSERT INTO kanji_reading VALUES(18,'語','かたらう');
INSERT INTO kanji_reading VALUES(19,'英','エイ');
INSERT INTO kanji_reading VALUES(20,'英','はなぶさ');
INSERT INTO kanji_reading VALUES(21,'何','カ');
INSERT INTO kanji_reading VALUES(22,'何','なに');
INSERT INTO kanji_reading VALUES(23,'何','なん');
INSERT INTO kanji_reading VALUES(24,'日','二');
INSERT INTO kanji_reading VALUES(27,'字','ジ');
INSERT INTO kanji_reading VALUES(34,'問','モン');
INSERT INTO kanji_reading VALUES(35,'質','シツ');
INSERT INTO kanji_reading VALUES(37,'漢','カン');
INSERT INTO kanji_reading VALUES(38,'道','ドウ');
INSERT INTO kanji_reading VALUES(39,'鉄','テツ');
INSERT INTO kanji_reading VALUES(40,'下','カ');
INSERT INTO kanji_reading VALUES(41,'地','チ');
INSERT INTO kanji_reading VALUES(42,'生','セイ');
INSERT INTO kanji_reading VALUES(43,'先','セン');
INSERT INTO kanji_reading VALUES(44,'達','だち');
INSERT INTO kanji_reading VALUES(45,'友','とも');
CREATE TABLE word_component (
    id integer primary key on conflict fail autoincrement,
    word_id integer,
    word_position integer,
    kanji_reading_id references kanji_reading (id) on delete cascade on update cascade,
        unique (word_id, word_position) on conflict fail
);
INSERT INTO word_component VALUES(1,1,1,3);
INSERT INTO word_component VALUES(2,1,2,4);
INSERT INTO word_component VALUES(3,2,1,5);
INSERT INTO word_component VALUES(4,2,2,6);
INSERT INTO word_component VALUES(5,3,1,7);
INSERT INTO word_component VALUES(6,3,2,3);
INSERT INTO word_component VALUES(7,4,1,8);
INSERT INTO word_component VALUES(10,5,2,14);
INSERT INTO word_component VALUES(11,5,3,16);
INSERT INTO word_component VALUES(12,6,1,19);
INSERT INTO word_component VALUES(13,6,2,16);
INSERT INTO word_component VALUES(14,7,1,23);
INSERT INTO word_component VALUES(15,7,2,16);
INSERT INTO word_component VALUES(25,8,2,34);
INSERT INTO word_component VALUES(26,8,1,35);
INSERT INTO word_component VALUES(27,9,2,27);
INSERT INTO word_component VALUES(28,9,1,37);
INSERT INTO word_component VALUES(29,10,4,38);
INSERT INTO word_component VALUES(30,10,3,39);
INSERT INTO word_component VALUES(31,10,2,40);
INSERT INTO word_component VALUES(32,10,1,41);
INSERT INTO word_component VALUES(33,5,1,24);
INSERT INTO word_component VALUES(34,4,2,7);
INSERT INTO word_component VALUES(35,11,2,42);
INSERT INTO word_component VALUES(36,11,1,43);
INSERT INTO word_component VALUES(37,12,2,44);
INSERT INTO word_component VALUES(38,12,1,45);
CREATE TABLE word_translation (
    id integer primary key on conflict fail autoincrement,
    word_id references word_component (word_id) on delete cascade on update cascade,
    translation varchar(200)
);
INSERT INTO word_translation VALUES(1,1,'время');
INSERT INTO word_translation VALUES(2,1,'час');
INSERT INTO word_translation VALUES(3,1,'период');
INSERT INTO word_translation VALUES(4,2,'часы (наручные, настенные, и т.д.)');
INSERT INTO word_translation VALUES(5,3,'в то время');
INSERT INTO word_translation VALUES(6,3,'в те дни');
INSERT INTO word_translation VALUES(7,4,'японский обед (еда) в коробке');
INSERT INTO word_translation VALUES(8,10,'метро');
INSERT INTO word_translation VALUES(9,9,'японский иероглиф');
INSERT INTO word_translation VALUES(10,8,'вопрос');
INSERT INTO word_translation VALUES(13,6,'английский язык');
INSERT INTO word_translation VALUES(14,5,'японский язык');
INSERT INTO word_translation VALUES(16,11,'учитель');
INSERT INTO word_translation VALUES(17,12,'друг');
PRAGMA writable_schema=ON;
CREATE TABLE IF NOT EXISTS sqlite_sequence(name,seq);
DELETE FROM sqlite_sequence;
INSERT INTO sqlite_sequence VALUES('kanji_reading',45);
INSERT INTO sqlite_sequence VALUES('word_component',38);
INSERT INTO sqlite_sequence VALUES('word_translation',17);
PRAGMA writable_schema=OFF;
COMMIT;
