---
-- The `tasks` table is the primary data store for placementd
---
CREATE TABLE tasks (
        ident UUID NOT NULL PRIMARY KEY,
        state TEXT NOT NULL DEFAULT 'planned',
        tags HSTORE,
        created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
        updated  TIMESTAMP WITH TIME ZONE,
        completed TIMESTAMP WITH TIME ZONE
);

-- Using a check rather than an enum to make marshalling in and out of rust easier
ALTER TABLE tasks ADD CHECK (state IN ('planned', 'provisioning', 'running', 'completed', 'finalized'));

CREATE TRIGGER tasks_modification_trigger
        AFTER INSERT OR UPDATE ON tasks
        FOR EACH ROW EXECUTE FUNCTION send_notice();

