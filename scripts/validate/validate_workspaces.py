#!/usr/bin/env python3
"""
Validate System Initiative workspaces by checking all change sets for snapshot issues.
"""

import argparse
import os
import sqlite3
import sys
import time

import requests


def get_headers(token: str) -> dict:
    return {
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json',
        'Cache-Control': 'no-cache',
        'User-Agent': 'si/validate-workspaces (support@systeminit.com)'
    }


def request_with_retry(url: str, headers: dict, max_retries: int = 11, initial_delay: float = 1.0) -> requests.Response:
    """Make a GET request with exponential backoff retry on 502/503 errors and read timeouts."""
    delay = initial_delay
    last_exception = None

    for attempt in range(max_retries + 1):
        try:
            response = requests.get(url, headers=headers)
        except requests.exceptions.ReadTimeout as e:
            last_exception = e
            if attempt < max_retries:
                print(f"      Read timed out, retrying in {delay:.1f}s (attempt {attempt + 1}/{max_retries})...")
                time.sleep(delay)
                delay *= 2
                continue
            raise last_exception

        if response.status_code != 503 and response.status_code != 502:
            response.raise_for_status()
            return response

        if response.status_code == 502:
            last_exception = requests.HTTPError(f"502 Bad Gateway", response=response)
        else:
            last_exception = requests.HTTPError(f"503 Service Unavailable", response=response)

        if attempt < max_retries:
            print(f"      Got {response.status_code}, retrying in {delay:.1f}s (attempt {attempt + 1}/{max_retries})...")
            time.sleep(delay)
            delay *= 2

    if last_exception is not None:
        raise last_exception

    raise Exception("How did i get here?")


def get_workspace_ids(auth_api_url: str, headers: dict) -> list[str]:
    """Fetch all workspace IDs from the admin API."""
    url = f"{auth_api_url}/list-workspace-ids"
    print(url);
    response = requests.get(url, headers=headers)
    response.raise_for_status()
    data = response.json()
    return data.get('workspaces', [])


def get_change_sets(api_url: str, headers: dict, workspace_id: str) -> dict:
    """Fetch all change sets for a workspace."""
    url = f"{api_url}/api/v2/admin/workspaces/{workspace_id}/change_sets"
    response = request_with_retry(url, headers)
    data = response.json()
    return data.get('changeSets', {})


def validate_snapshot(api_url: str, headers: dict, workspace_id: str, change_set_id: str) -> dict:
    """Validate a snapshot for a specific change set."""
    url = f"{api_url}/api/v2/admin/workspaces/{workspace_id}/change_sets/{change_set_id}/validate_snapshot"
    response = request_with_retry(url, headers)
    return response.json()


def insert_result(cursor: sqlite3.Cursor, conn: sqlite3.Connection, result: dict):
    """Insert a validation result into the database."""
    cursor.execute('''
        INSERT INTO validation_results (
            workspace_id, change_set_id, change_set_name, change_set_status,
            validation_status, issue_count, issues
        ) VALUES (
            :workspace_id, :change_set_id, :change_set_name, :change_set_status,
            :validation_status, :issue_count, :issues
        )
    ''', result)
    conn.commit()


def main():
    parser = argparse.ArgumentParser(
        description='Validate System Initiative workspace snapshots'
    )
    parser.add_argument(
        '--sdf-api-url',
        default=os.environ.get('SDF_API_URL'),
        help='SDF API URL (or set SDF_API_URL env var)'
    )
    parser.add_argument(
        '--auth-api-url',
        default=os.environ.get('SI_AUTH_API_URL'),
        help='Auth API URL (or set SI_AUTH_API_URL env var)'
    )
    parser.add_argument(
        '--bearer-token',
        default=os.environ.get('BEARER_TOKEN'),
        help='Bearer token for authentication (or set BEARER_TOKEN env var)'
    )
    parser.add_argument(
        '--auth-bearer-token',
        default=os.environ.get('AUTH_BEARER_TOKEN'),
        help='Bearer token for the auth api (or set AUTH_BEARER_TOKEN env var)'
    )
    parser.add_argument(
        '--output',
        '-o',
        default='validation_results.db',
        help='Output SQLite database file path (default: validation_results.db)'
    )
    parser.add_argument(
        '--delay',
        '-d',
        action='store_true',
        help='Add a 0.5 second delay between validating each change set'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Fetch workspaces and change sets but skip actual validation API calls'
    )

    args = parser.parse_args()

    if not args.sdf_api_url:
        print("Error: SDF_API_URL must be set via --sdf-api-url or environment variable", file=sys.stderr)
        sys.exit(1)

    if not args.auth_api_url:
        print("Error: SI_AUTH_API_URL must be set via --auth-api-url or environment variable", file=sys.stderr)
        sys.exit(1)

    if not args.bearer_token:
        print("Error: BEARER_TOKEN must be set via --bearer-token or environment variable", file=sys.stderr)
        sys.exit(1)

    if not args.auth_bearer_token:
        print("Error: AUTH_BEARER_TOKEN must be set via --auth-bearer-token or environment variable", file=sys.stderr)
        sys.exit(1)

    sdf_api_url = args.sdf_api_url.rstrip('/')
    auth_api_url = args.auth_api_url.rstrip('/')
    sdf_headers = get_headers(args.bearer_token)
    auth_headers = get_headers(args.auth_bearer_token)

    if args.dry_run:
        print("DRY RUN MODE - will not call validation endpoint")

    print(f"Fetching workspace IDs from {auth_api_url}...")
    try:
        workspace_ids = get_workspace_ids(auth_api_url, auth_headers)
    except requests.RequestException as e:
        print(f"Error fetching workspace IDs: {e}", file=sys.stderr)
        sys.exit(1)

    print(f"Found {len(workspace_ids)} workspaces")

    # Initialize database
    conn = sqlite3.connect(args.output)
    cursor = conn.cursor()
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS validation_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            workspace_id TEXT,
            change_set_id TEXT,
            change_set_name TEXT,
            change_set_status TEXT,
            validation_status TEXT,
            issue_count INTEGER,
            issues TEXT
        )
    ''')
    conn.commit()

    # Counters for summary
    total = 0
    valid_count = 0
    invalid_count = 0
    error_count = 0
    dry_run_count = 0

    for workspace_id in workspace_ids:
        print(f"Processing workspace {workspace_id}...")

        try:
            change_sets = get_change_sets(sdf_api_url, sdf_headers, workspace_id)
        except requests.RequestException as e:
            print(f"  Error fetching change sets: {e}", file=sys.stderr)
            insert_result(cursor, conn, {
                'workspace_id': workspace_id,
                'change_set_id': '',
                'change_set_name': '',
                'change_set_status': '',
                'validation_status': 'error',
                'issue_count': 0,
                'issues': str(e)
            })
            total += 1
            error_count += 1
            continue

        print(f"  Found {len(change_sets)} change sets")

        for change_set_id, change_set_info in change_sets.items():
            change_set_name = change_set_info.get('name', '')
            change_set_status = change_set_info.get('status', '')

            # Only validate change sets with status "Open"
            if change_set_status != 'Open':
                continue

            if args.dry_run:
                print(f"    [DRY RUN] Would validate change set {change_set_id} ({change_set_name}, status: {change_set_status})")
                insert_result(cursor, conn, {
                    'workspace_id': workspace_id,
                    'change_set_id': change_set_id,
                    'change_set_name': change_set_name,
                    'change_set_status': change_set_status,
                    'validation_status': 'dry_run',
                    'issue_count': 0,
                    'issues': ''
                })
                total += 1
                dry_run_count += 1
            else:
                print(f"    Validating change set {change_set_id} ({change_set_name})...")

                try:
                    validation = validate_snapshot(sdf_api_url, sdf_headers, workspace_id, change_set_id)
                    issues = validation.get('issues', [])
                    issue_count = len(issues)

                    if issue_count == 0:
                        validation_status = 'valid'
                        issues_str = ''
                        valid_count += 1
                    else:
                        validation_status = 'invalid'
                        issues_str = '; '.join([
                            f"{issue.get('message', 'Unknown issue')}"
                            for issue in issues
                        ])
                        invalid_count += 1

                    insert_result(cursor, conn, {
                        'workspace_id': workspace_id,
                        'change_set_id': change_set_id,
                        'change_set_name': change_set_name,
                        'change_set_status': change_set_status,
                        'validation_status': validation_status,
                        'issue_count': issue_count,
                        'issues': issues_str
                    })
                    total += 1

                    if issue_count > 0:
                        print(f"      Found {issue_count} issues")
                    else:
                        print("      Valid")

                except requests.RequestException as e:
                    print(f"      Error validating: {e}", file=sys.stderr)
                    insert_result(cursor, conn, {
                        'workspace_id': workspace_id,
                        'change_set_id': change_set_id,
                        'change_set_name': change_set_name,
                        'change_set_status': change_set_status,
                        'validation_status': 'error',
                        'issue_count': 0,
                        'issues': str(e)
                    })
                    total += 1
                    error_count += 1

                if args.delay:
                    time.sleep(0.5)

    conn.close()

    print(f"\nResults written to {args.output}")

    # Summary
    print("\nSummary:")
    print(f"  Total change sets: {total}")
    if args.dry_run:
        print(f"  Would validate: {dry_run_count}")
    else:
        print(f"  Valid: {valid_count}")
        print(f"  Invalid: {invalid_count}")
        print(f"  Errors: {error_count}")


if __name__ == '__main__':
    main()
