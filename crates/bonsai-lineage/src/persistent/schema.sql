PRAGMA application_id=1112429388;
PRAGMA user_version=1;
CREATE TABLE metadata(singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 commits INTEGER NOT NULL, events INTEGER NOT NULL, live_artifacts INTEGER NOT NULL,
 sessions INTEGER NOT NULL);
INSERT INTO metadata VALUES(1,0,0,0,1);
CREATE TABLE artifacts(id BLOB PRIMARY KEY CHECK(length(id)=16),
 state BLOB NOT NULL, sha256 BLOB NOT NULL CHECK(length(sha256)=32),terminal INTEGER NOT NULL) WITHOUT ROWID;
CREATE TABLE revisions(id BLOB PRIMARY KEY CHECK(length(id)=16),
 owner BLOB NOT NULL CHECK(length(owner)=16)) WITHOUT ROWID;
CREATE TABLE history(id BLOB PRIMARY KEY CHECK(length(id)=16)) WITHOUT ROWID;
CREATE TABLE parents(child BLOB NOT NULL,parent BLOB NOT NULL,PRIMARY KEY(child,parent)) WITHOUT ROWID;
CREATE TABLE traversal(id BLOB PRIMARY KEY,done INTEGER NOT NULL) WITHOUT ROWID;
CREATE INDEX traversal_pending ON traversal(done,id);
CREATE TABLE commits(sequence INTEGER PRIMARY KEY,directory TEXT NOT NULL UNIQUE,
 events INTEGER NOT NULL,checksum BLOB NOT NULL CHECK(length(checksum)=32));
