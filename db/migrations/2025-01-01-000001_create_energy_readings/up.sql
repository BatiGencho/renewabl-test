CREATE TABLE energy_readings (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    reading_time  TIMESTAMPTZ NOT NULL,
    quantity_kwh  NUMERIC(12, 4) NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- one reading per hour ??
CREATE UNIQUE INDEX idx_energy_readings_reading_time 
    ON energy_readings (reading_time);