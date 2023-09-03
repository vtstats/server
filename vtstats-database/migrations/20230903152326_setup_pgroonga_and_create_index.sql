CREATE EXTENSION IF NOT EXISTS pgroonga;

CREATE INDEX pgroonga_stream_title_index ON streams USING pgroonga (title) WITH (
    tokenizer = 'TokenNgram("report_source_location", true, "loose_blank", true)'
);

CREATE TABLE pgroonga_synonyms (term text PRIMARY KEY, synonyms text []);

CREATE INDEX pgroonga_synonyms_search_index ON pgroonga_synonyms USING pgroonga (term pgroonga_text_term_search_ops_v2);

INSERT INTO
    pgroonga_synonyms (term, synonyms)
VALUES
    ('hololive', ARRAY ['hololive', 'ホロライブ']);

INSERT INTO
    pgroonga_synonyms (term, synonyms)
VALUES
    ('nijisanji', ARRAY ['nijisanji', 'にじさんじ']);

INSERT INTO
    pgroonga_synonyms (term, synonyms)
VALUES
    ('minecraft', ARRAY ['minecraft', 'マイクラ']);