#!/usr/bin/python3

import requests
import subprocess
import json
import time
import os
import random


ws_ids = [
    "01J7KJ0WHCDHZ4YMKQMPZSPEW3",
    "01J7KJ0YBSRWC57X0M48S94CX4",
    "01J7KJ0X5MYC76YJ2SP34N49PD",
    "01J7KJ0XSM4FWA76MW9M8Z70HE",
    "01J7KJ0YYR49T30N6VDBA17TFA",
    "01J7KJ105KPMG5Y4GRJBD6Q23T",
    "01J7KJ10SWAVJ5QQ139TQYFV3E",
    "01J7KJ11D9DQJCZ2ZFGYXEVBPZ",
    "01J7KJ149YFZ53PQTDT3BF5T7B",
    "01J7KJ16WHB71TNFKCDNTTNG0F",
    "01J7KJ191CBB3NCX82HNBB8CHG",
    "01J7KJ19RTX2YPVHTJVPXE5R7S",
    "01J7KJ0ZJDPJANCS7YZ6JY5YAC",
    "01J7KJ121DMFV7VEJHNWDAPCPF",
    "01J7KJ138DA6FV9T1GT1WQTDZ3",
    "01J7KJ15ARY8D6R833D9R6FMV4",
    "01J7KJ1633QCWFSNT1CDSTD369",
    "01J7KJ17N4KF96GTY18G9B6RD4",
    "01J7KJ18CQESWE6EDBJC1QCJQ1"
]

SDF_API_URL = "https://tools.systeminit.com/api"
AUTH_API_URL = "https://auth-api.systeminit.com"
USER_ID = "technical-operations+synthetics@systeminit.com"
PASSWORD = "xgv1pjc!RVU@bet5wft"
HONEYCOMB_KEY = "17aUCFVo9UqCv1fkie1lMF"
TEST = "emulate_paul_stack"
BATCHSIZE = "6"

# CHANGESET=""

PROFILE = json.dumps({"maxDuration": 300, "rate": 10000, "useJitter": True})


def get_token():
    response = requests.post(
        f"{AUTH_API_URL}/auth/login",
        headers={"Content-Type": "application/json"},
        json={"email": USER_ID, "password": PASSWORD, "workspaceId": ws_ids[0]}
    )
    return response.json().get("token")


def post_to_honeycomb(message, event_type):
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
        # "--changeSetId", CHANGESET,
        # "--profile", PROFILE,
        "--tests", TEST,
        "--token", token,
        "--reportFile", f"reports/{id}.json",
        "-b", BATCHSIZE,
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
    post_to_honeycomb("Beginning soak test locally.", "soak-test-start")
    start = time.time()
    token = get_token()

    processes = []
    for id in ws_ids:
        process = run_task(id, token)
        processes.append(process)
        time.sleep(random.randint(1, 4))

    for process in processes:
        process.wait()

    end = time.time()
    runtime = end - start
    post_to_honeycomb("Finishing soak test locally.", "soak-test-stop")

    total_success, total_failure, average_duration = process_reports()

    print(f"With profile: {PROFILE}")
    print(f"Total Success: {total_success}")
    print(f"Total Failure: {total_failure}")
    print(f"Average Duration: {average_duration} ms")
    print(f"Ran for {runtime} seconds")


if __name__ == "__main__":
    main()
