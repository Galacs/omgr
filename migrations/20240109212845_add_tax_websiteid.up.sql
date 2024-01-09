DELETE FROM tax;

ALTER TABLE tax
   DROP CONSTRAINT tax_tax_key;
ALTER TABLE tax
    ADD website_id VARCHAR PRIMARY KEY NOT NULL UNIQUE;


INSERT INTO tax(tax, rate, website_id) VALUES ('withdraw', 5, '1');
INSERT INTO tax(tax, rate, website_id) VALUES ('withdraw', 5, '2');
INSERT INTO tax(tax, rate, website_id) VALUES ('withdraw', 5, '3');
INSERT INTO tax(tax, rate, website_id) VALUES ('withdraw', 5, '4');
INSERT INTO tax(tax, rate, website_id) VALUES ('withdraw', 5, '5');
INSERT INTO tax(tax, rate, website_id) VALUES ('withdraw', 5, '6');
INSERT INTO tax(tax, rate, website_id) VALUES ('withdraw', 5, '7');
