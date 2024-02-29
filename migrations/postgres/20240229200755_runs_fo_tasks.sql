---
-- Stores the raw runs submitted to the service
---

CREATE TABLE submitted_runs (
        ident UUID NOT NULL PRIMARY KEY,
        raw JSON NOT NULL,
        task UUID REFERENCES tasks(ident) NOT NULL,
        tags HSTORE,
        created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
        updated  TIMESTAMP WITH TIME ZONE,
        completed TIMESTAMP WITH TIME ZONE
);

CREATE TRIGGER submitted_runs_modification_trigger
        AFTER INSERT OR UPDATE ON submitted_runs
        FOR EACH ROW EXECUTE PROCEDURE send_notice();
