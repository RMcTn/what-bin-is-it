CREATE TABLE IF NOT EXISTS emails (
	id          INTEGER PRIMARY KEY,
	email       TEXT NOT NULL,
	postcode    TEXT NOT NULL,
	address     TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS EmailsUniqueIndexOnEmails ON emails (email);
