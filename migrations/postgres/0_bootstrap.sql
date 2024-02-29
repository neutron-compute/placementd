--
-- This file is the first mnigrat
--

CREATE EXTENSION hstore;

CREATE OR REPLACE FUNCTION send_notice()
        RETURNS trigger
        LANGUAGE plpgsql AS
$fun$
BEGIN
        PERFORM pg_notify(FORMAT('%s-modified', TG_TABLE_NAME), NEW.ident::text);
        RETURN NULL;
END
$fun$;

