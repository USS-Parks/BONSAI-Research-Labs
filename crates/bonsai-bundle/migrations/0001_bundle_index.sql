PRAGMA application_id = 1112429385;

CREATE TABLE index_metadata (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    format TEXT NOT NULL CHECK (format = 'bonsai.bundle-index/v1')
) STRICT;

INSERT INTO index_metadata(singleton, format)
VALUES (1, 'bonsai.bundle-index/v1');

CREATE TABLE event_segments (
    sequence_decimal TEXT PRIMARY KEY
        CHECK (length(sequence_decimal) BETWEEN 1 AND 20),
    relative_path TEXT NOT NULL UNIQUE,
    frame_count_decimal TEXT NOT NULL
        CHECK (length(frame_count_decimal) BETWEEN 1 AND 20),
    maximum_frame_size INTEGER NOT NULL
        CHECK (maximum_frame_size > 0 AND maximum_frame_size <= 16777216),
    sha256 TEXT NOT NULL
        CHECK (length(sha256) = 64),
    byte_length_decimal TEXT NOT NULL
        CHECK (length(byte_length_decimal) BETWEEN 1 AND 20)
) STRICT;

CREATE TABLE derived_artifacts (
    sha256 TEXT PRIMARY KEY
        CHECK (length(sha256) = 64),
    relative_path TEXT NOT NULL UNIQUE,
    byte_length_decimal TEXT NOT NULL
        CHECK (length(byte_length_decimal) BETWEEN 1 AND 20)
) STRICT;

PRAGMA user_version = 1;
