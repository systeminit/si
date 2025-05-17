
# Manually Creating PG Restore Points

How do I refresh the restore points or create new restore points for tests? 
1. Refresh/clean out your local database
2. Launch Stack Locally
3. Once the migrations have completed, create a single changeset
4. Disable all services
5. Dump the Database Content 
a) pg_dump -h 0.0.0.0 -p 7432 -U si -d si_layer_db -f si_layer_db_backup.sql
b) pg_dump -h 0.0.0.0 -p 7432 -U si -d si -f si_db_backup.sql 
6. Use these are your new restore points
7. If you want to test how a service handles a set of procedures, simply disable the service and record the nats messages, and use that nats message content to drive the individual service.

# For si_layer_cache database

This restores the a database back to the original pre-test state

PGPASSWORD=bugbear psql -h 0.0.0.0 -p 7432 -U si -d postgres <<EOF
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'si_layer_db' AND pid <> pg_backend_pid();
DROP DATABASE IF EXISTS si_layer_db;
CREATE DATABASE si_layer_db;
EOF

PGPASSWORD=bugbear psql -h 0.0.0.0 -p 7432 -U si -d si_layer_db -f si_layer_db_backup.sql

# For si database

PGPASSWORD=bugbear psql -h 0.0.0.0 -p 7432 -U si -d postgres <<EOF
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'si' AND pid <> pg_backend_pid();
DROP DATABASE IF EXISTS si;
CREATE DATABASE si;
EOF

PGPASSWORD=bugbear psql -h 0.0.0.0 -p 7432 -U si -d si -f si_db_backup.sql




# Manually Tracking NATS Messages
To record a new sequence of test messages from an established start point, you need to do the following:

1. Set up passive copy/mirror - 
(a) nats --server 0.0.0.0 stream add REBASER_REQUESTS_AUDIT --source REBASER_REQUESTS --description "Passive copy of REBASER_REQUESTS for recording" --storage file --retention limits --max-msgs 10000
2. Set up consumer
(a) nats --server 0.0.0.0 consumer add REBASER_REQUESTS_AUDIT my-observer --deliver all --ack none --replay instant
3. Clear the stream messages if it's not already empty
(a) nats --server 0.0.0.0 stream purge REBASER_REQUESTS_AUDIT
4. Set up recorder for 2(?) mins to record the test you want to perform
(a) nats consumer next REBASER_REQUESTS_AUDIT my-observer --count 10 --timeout 120s > recorded_messages.txt # I.e. record for 10 rebase requests
5. Transform the recorded messages in recorded_messages.txt into json format and place into the relevant sequence.json
(a) python3 convert_recorded_messages_to_json.py # outputs sequence.json

