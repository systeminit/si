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