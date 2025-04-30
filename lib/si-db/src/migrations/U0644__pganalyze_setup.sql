DO LANGUAGE PLPGSQL
$pganalyze_setup$
BEGIN
    IF EXISTS (
        SELECT FROM pg_catalog.pg_roles
        WHERE rolname = 'pganalyze'
    ) THEN
        RAISE NOTICE 'Role ''pganalyze'' already exists. Skipping pganalyze setup.';
    ELSE
        IF NOT EXISTS (
            SELECT FROM pg_catalog.pg_roles
            WHERE rolname = 'rds_superuser'
        ) OR pg_has_role(current_user, 'rds_superuser') THEN
            CREATE USER pganalyze WITH PASSWORD 'tSYwfHHdBBlfWlxa' CONNECTION LIMIT 5;
            GRANT pg_monitor TO pganalyze;

            CREATE SCHEMA IF NOT EXISTS pganalyze;
            GRANT USAGE ON SCHEMA pganalyze TO pganalyze;
            GRANT USAGE ON SCHEMA public TO pganalyze;

            CREATE OR REPLACE FUNCTION pganalyze.get_stat_replication() RETURNS SETOF pg_stat_replication AS
            $$
                /* pganalyze-collector */ SELECT * FROM pg_catalog.pg_stat_replication;
            $$ LANGUAGE sql VOLATILE SECURITY DEFINER;

            CREATE OR REPLACE FUNCTION pganalyze.get_column_stats() RETURNS SETOF pg_stats AS
            $$
                /* pganalyze-collector */ SELECT schemaname, tablename, attname, inherited, null_frac, avg_width,
                n_distinct, NULL::anyarray, most_common_freqs, NULL::anyarray, correlation, NULL::anyarray,
                most_common_elem_freqs, elem_count_histogram
                FROM pg_catalog.pg_stats;
            $$ LANGUAGE sql VOLATILE SECURITY DEFINER;

            -- For pganalyze sequence report
            CREATE OR REPLACE FUNCTION pganalyze.get_sequence_oid_for_column(table_name text, column_name text) RETURNS oid AS
            $$
            /* pganalyze-collector */ SELECT pg_get_serial_sequence(table_name, column_name)::regclass::oid;
            $$ LANGUAGE sql VOLATILE SECURITY DEFINER;

            -- The following is needed for Postgres 10+:
            CREATE OR REPLACE FUNCTION pganalyze.get_sequence_state(schema_name text, sequence_name text) RETURNS TABLE(
                last_value bigint, start_value bigint, increment_by bigint,
                max_value bigint, min_value bigint, cache_size bigint, cycle boolean
            ) AS
            $$
            /* pganalyze-collector */ SELECT last_value, start_value, increment_by, max_value, min_value, cache_size, cycle
                FROM pg_sequences WHERE schemaname = schema_name AND sequencename = sequence_name;
            $$ LANGUAGE sql VOLATILE SECURITY DEFINER;

            -- For pganalyze buffer cache report
            CREATE EXTENSION IF NOT EXISTS pg_buffercache WITH SCHEMA public;
            CREATE OR REPLACE FUNCTION pganalyze.get_buffercache() RETURNS SETOF public.pg_buffercache AS
            $$
            /* pganalyze-collector */ SELECT * FROM public.pg_buffercache;
            $$ LANGUAGE sql VOLATILE SECURITY DEFINER;

        ELSE
            RAISE NOTICE 'Current user does not have superuser-like privileges. Skipping pganalyze setup.';
        END IF;
    END IF;
END;
$pganalyze_setup$;

CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
-- SELECT calls, query FROM pg_stat_statements LIMIT 1;
