CREATE TABLE stock (
    website_id VARCHAR PRIMARY KEY NOT NULL UNIQUE,
    stock INTEGER NOT NULL DEFAULT 0
);

INSERT INTO stock(website_id) VALUES ('1');
INSERT INTO stock(website_id) VALUES ('2');
INSERT INTO stock(website_id) VALUES ('3');
INSERT INTO stock(website_id) VALUES ('4');
INSERT INTO stock(website_id) VALUES ('5');