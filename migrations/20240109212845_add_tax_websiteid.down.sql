DELETE FROM tax;

ALTER TABLE tax
    DROP website_id;
ALTER TABLE tax
   ADD CONSTRAINT tax_tax_key UNIQUE (tax);