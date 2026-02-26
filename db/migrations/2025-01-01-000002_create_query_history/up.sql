CREATE TABLE query_history (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregation_type  TEXT        NOT NULL,
    date_from         TIMESTAMPTZ,
    date_to           TIMESTAMPTZ,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_query_history_created_at
    ON query_history (created_at DESC);
