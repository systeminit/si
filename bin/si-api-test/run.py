#!/usr/bin/env python3

import requests
import subprocess
import json
import time
import os
import random


### This script makes it easy to execute and run api tests with various levels
### of parallelism. 

## Steps to use: 
# 1. Add your Workspace ID to ws_ids
# 2. Update the SDF_API_URL and AUTH_API_URL (if running locally)
# 3. Set up credentials, you've got 2 options: 
#    1. Use the Tech Ops User (get the password out of 1pass and plug into PASSWORD below). 
#       You'll need to add the Tech Ops user to your workspace if they're not already an authorized user
#    2. Swipe your bearer token from the front end, and plug into USER_TOKEN below
#    If the token is set, we'll prefer that over the username/password
# 4. Configure which test to run. If running an E2E Cron test, set TEST = test_name (example below). 
#    If running a benchmark test, set TEST = "benchmark/{test_name}" (example below)
# 5. Configure concurrency - set BATCH = # of concurrent tests you want to kick off 
#    Beware concurrent tests impacting each other if they're applying to head!   
# 6. Configure Honeycomb Key: If running tests against a live environment, get the Honeycomb key out of 1pass
#    This lets us create Honeycomb markers for the start and end of tests to make gathering telemtry easier   
# 7. Have fun! :)  


ws_ids = [
  #"01JKRT3BB747W4Z74RVZ1XFX4K", # tools.systeminit.com Load Testing 2k25 workspace
]

SDF_API_URL = "https://tools.systeminit.com/api"
#SDF_API_URL = "http://localhost:8080/api" # local
AUTH_API_URL = "https://auth-api.systeminit.com"
USER_ID = "technical-operations+synthetics@systeminit.com"
USER_TOKEN = ""

PASSWORD = ""
HONEYCOMB_KEY = ""
#TEST = "7-emulate_authoring" ## E2E Cron Test Run
TEST = "benchmark/func_stampede" ## Benchmark Test Run
BATCH = "30" ## how many concurrent test runs



PROFILE = json.dumps({"maxDuration": 150, "rate": 2500, "useJitter": True})

def get_token():
    if USER_TOKEN == "":
        response = requests.post(
            f"{AUTH_API_URL}/auth/login",
            headers={"Content-Type": "application/json"},
            json={"email": USER_ID, "password": PASSWORD, "workspaceId": ws_ids[0]}
        )
        return response.json().get("token")
    return USER_TOKEN

    

def post_to_honeycomb(message, event_type):
    if HONEYCOMB_KEY != "":
        data = {"message": message, "type": event_type}
        requests.post(
            "https://api.honeycomb.io/1/markers/sdf",
            headers={"X-Honeycomb-Team": HONEYCOMB_KEY},
            json=data
    )

def run_task(id, token):
    env_vars = os.environ.copy()
    env_vars["SDF_API_URL"] = SDF_API_URL
    env_vars["AUTH_API_URL"] = AUTH_API_URL
    cmd = [
        "deno", "task", "run",
        "--workspaceId", id,
        "--tests", TEST,
        "--token", token,
        "--reportFile", f"reports/{id}.json",
        "-b", BATCH,
    ]
    return subprocess.Popen(cmd, env=env_vars)

def clear_reports():
    if os.path.exists('./reports'):
        for file in os.listdir('./reports'):
            os.remove(os.path.join('./reports', file))

def process_reports():
    reports = []
    for file in os.listdir('./reports'):
        with open(f'./reports/{file}', 'r') as f:
            reports.append(json.load(f))

    flattened_reports = [item for sublist in reports for item in sublist]
    total_success = len([r for r in flattened_reports if r.get("test_result") == "success"])
    total_failure = len([r for r in flattened_reports if r.get("test_result") == "failure"])
    average_duration = sum(
        [int(r.get("test_duration").replace("ms", "")) for r in flattened_reports]
    ) / len(flattened_reports) if flattened_reports else 0

    failures = [r.get("message") for r in flattened_reports if r.get("test_result") == "failure"]

    with open('./failures', 'w') as f:
        for failure in failures:
            f.write(failure + '\n')

    return total_success, total_failure, average_duration

def main():
    clear_reports()
    start = time.time()
    token = get_token()
    processes = []
    post_to_honeycomb(f"running load test - {TEST}", "")
    for id in ws_ids:
        process = run_task(id, token)
        processes.append(process)
        time.sleep(random.randint(1, 4))

    for process in processes:
        process.wait()

    end = time.time()
    runtime = end - start

    total_success, total_failure, average_duration = process_reports()
    post_to_honeycomb(f"finished running load test - {TEST}", "")
    print(f"With batch size: {BATCH}")
    print(f"Total Success: {total_success}")
    print(f"Total Failure: {total_failure}")
    print(f"Average Duration: {average_duration} ms")
    print(f"Ran for {runtime} seconds")

if __name__ == "__main__":
    main()