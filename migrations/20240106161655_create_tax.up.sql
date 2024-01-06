CREATE TABLE tax (
    tax VARCHAR NOT NULL UNIQUE,
    rate REAL NOT NULL
);

INSERT INTO tax(tax, rate) VALUES ('withdraw', 5);