#!/bin/bash

echo "STARTING WEB"
buck2 run app/web:web -- --host 0.0.0.0 &

sudo apt-get update -y
sudo apt-get install -y --no-install-recommends wget

echo "DOWNLOADING SDF"
wget -q https://artifacts.systeminit.com/sdf/stable/omnibus/linux/$(arch)/sdf-stable-omnibus-linux-$(arch).tar.gz -O - | sudo tar -xzf - -C / >/dev/null 2>&1

echo "DOWNLOADING EDDA"
wget -q https://artifacts.systeminit.com/edda/stable/omnibus/linux/$(arch)/edda-stable-omnibus-linux-$(arch).tar.gz -O - | sudo tar -xzf - -C / >/dev/null 2>&1

echo "DOWNLOADING BEDROCK"
wget -q https://artifacts.systeminit.com/bedrock/stable/omnibus/linux/$(arch)/bedrock-stable-omnibus-linux-$(arch).tar.gz -O - | sudo tar -xzf - -C / >/dev/null 2>&1

echo "Starting Bedrock"
max_attempts=10
attempt=0
success=0

while [ "$attempt" -lt "$max_attempts" ]; do
    ((attempt++))
    echo "Starting bedrock (attempt $attempt)..."

    bedrock --nats-url 0.0.0.0:4222 -vvvv &
    pid=$!

    sleep 5

    if kill -0 "$pid" 2>/dev/null; then
    echo "bedrock is running and appears healthy (PID $pid)"
    success=1
    break
    else
    echo "bedrock crashed early, retrying..."
    wait "$pid" 2>/dev/null
    fi
done

if [ "$success" -ne 1 ]; then
    echo "bedrock failed after $attempt attempts, giving up."
    exit 1
fi

sleep 5
curl --location http://localhost:3020/prepare \
    --header "Content-Type: application/json" \
    --data '{
    "recording_id": "ptlw-50-01JXZH5RFKTXMSTZJVZ55TPET3-100-01JXZH7DHBRX6GZCRC6T4QRVGF-300-01JXZHC9DR5P2K95KPRG895RRX",
    "parameters": {},
    "executionParameters": {}
    }'

echo "Starting other services"
sdf &
edda &
sleep 5
echo "Pre-reqs are all setup"

cd app/web
pnpm i
npx cypress install
