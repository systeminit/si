#!/usr/bin/env python3
"""
Validate System Initiative workspaces by checking all change sets for snapshot issues.
"""

import argparse
import csv
import os
import sys
import time

import requests


def get_headers(token: str) -> dict:
    return {
        'Authorization': f'Bearer {token}',
        'Content-Type': 'application/json',
        'Cache-Control': 'no-cache',
        'User-Agent': 'si.git/admin-tests (support@systeminit.com)'
    }


def get_workspace_ids(api_url: str, headers: dict) -> list[str]:
    """Fetch all workspace IDs from the admin API."""
    url = f"{api_url}/api/v2/admin/workspace_ids"
    response = requests.get(url, headers=headers)
    response.raise_for_status()
    data = response.json()
    return data.get('workspaces', [])


def get_change_sets(api_url: str, headers: dict, workspace_id: str) -> dict:
    """Fetch all change sets for a workspace."""
    url = f"{api_url}/api/v2/admin/workspaces/{workspace_id}/change_sets"
    response = requests.get(url, headers=headers)
    response.raise_for_status()
    data = response.json()
    return data.get('changeSets', {})


def validate_snapshot(api_url: str, headers: dict, workspace_id: str, change_set_id: str) -> dict:
    """Validate a snapshot for a specific change set."""
    url = f"{api_url}/api/v2/admin/workspaces/{workspace_id}/change_sets/{change_set_id}/validate_snapshot"
    response = requests.get(url, headers=headers)
    response.raise_for_status()
    return response.json()


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
        '--bearer-token',
        default=os.environ.get('BEARER_TOKEN'),
        help='Bearer token for authentication (or set BEARER_TOKEN env var)'
    )
    parser.add_argument(
        '--output',
        '-o',
        default='validation_results.csv',
        help='Output CSV file path (default: validation_results.csv)'
    )
    parser.add_argument(
        '--delay',
        '-d',
        action='store_true',
        help='Add a 0.5 second delay between validating each change set'
    )

    args = parser.parse_args()

    if not args.sdf_api_url:
        print("Error: SDF_API_URL must be set via --sdf-api-url or environment variable", file=sys.stderr)
        sys.exit(1)

    if not args.bearer_token:
        print("Error: BEARER_TOKEN must be set via --bearer-token or environment variable", file=sys.stderr)
        sys.exit(1)

    api_url = args.sdf_api_url.rstrip('/')
    headers = get_headers(args.bearer_token)

    print(f"Fetching workspace IDs from {api_url}...")
    try:
        workspace_ids = get_workspace_ids(api_url, headers)
    except requests.RequestException as e:
        print(f"Error fetching workspace IDs: {e}", file=sys.stderr)
        sys.exit(1)

    print(f"Found {len(workspace_ids)} workspaces")

    results = []

    for workspace_id in workspace_ids:
        print(f"Processing workspace {workspace_id}...")

        try:
            change_sets = get_change_sets(api_url, headers, workspace_id)
        except requests.RequestException as e:
            print(f"  Error fetching change sets: {e}", file=sys.stderr)
            results.append({
                'workspace_id': workspace_id,
                'change_set_id': '',
                'change_set_name': '',
                'change_set_status': '',
                'validation_status': 'error',
                'issue_count': 0,
                'issues': str(e)
            })
            continue

        print(f"  Found {len(change_sets)} change sets")

        for change_set_id, change_set_info in change_sets.items():
            change_set_name = change_set_info.get('name', '')
            change_set_status = change_set_info.get('status', '')

            print(f"    Validating change set {change_set_id} ({change_set_name})...")

            try:
                validation = validate_snapshot(api_url, headers, workspace_id, change_set_id)
                issues = validation.get('issues', [])
                issue_count = len(issues)

                if issue_count == 0:
                    validation_status = 'valid'
                    issues_str = ''
                else:
                    validation_status = 'invalid'
                    issues_str = '; '.join([
                        f"{issue.get('message', 'Unknown issue')}"
                        for issue in issues
                    ])

                results.append({
                    'workspace_id': workspace_id,
                    'change_set_id': change_set_id,
                    'change_set_name': change_set_name,
                    'change_set_status': change_set_status,
                    'validation_status': validation_status,
                    'issue_count': issue_count,
                    'issues': issues_str
                })

                if issue_count > 0:
                    print(f"      Found {issue_count} issues")
                else:
                    print("      Valid")

            except requests.RequestException as e:
                print(f"      Error validating: {e}", file=sys.stderr)
                results.append({
                    'workspace_id': workspace_id,
                    'change_set_id': change_set_id,
                    'change_set_name': change_set_name,
                    'change_set_status': change_set_status,
                    'validation_status': 'error',
                    'issue_count': 0,
                    'issues': str(e)
                })

            if args.delay:
                time.sleep(0.5)

    # Write results to CSV
    fieldnames = [
        'workspace_id',
        'change_set_id',
        'change_set_name',
        'change_set_status',
        'validation_status',
        'issue_count',
        'issues'
    ]

    with open(args.output, 'w', newline='') as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(results)

    print(f"\nResults written to {args.output}")

    # Summary
    total = len(results)
    valid = sum(1 for r in results if r['validation_status'] == 'valid')
    invalid = sum(1 for r in results if r['validation_status'] == 'invalid')
    errors = sum(1 for r in results if r['validation_status'] == 'error')

    print("\nSummary:")
    print(f"  Total change sets checked: {total}")
    print(f"  Valid: {valid}")
    print(f"  Invalid: {invalid}")
    print(f"  Errors: {errors}")


if __name__ == '__main__':
    main()
