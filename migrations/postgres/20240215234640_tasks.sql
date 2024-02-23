---
-- The `tasks` table is the primary data store for placementd
---
CREATE EXTENSION hstore;

CREATE TYPE placementd_state AS ENUM (
        'planned',
        'provisioning',
        'running',
        'completed',
        'finalized'
);

CREATE TABLE tasks (
        ident UUID NOT NULL PRIMARY KEY,
        state placementd_state NOT NULL DEFAULT 'planned',
        tags HSTORE,
        created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
        updated  TIMESTAMP WITH TIME ZONE,
        completed TIMESTAMP WITH TIME ZONE
);
