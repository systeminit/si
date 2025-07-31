# Read through all files in current directory
import csv
import os
import re

def read_workspace_list(file: str):
    with open(file, 'r') as f:
        return f.read().splitlines()

deleted_workspaces = read_workspace_list('../deleted_workspaces.txt')
only_workspaces = read_workspace_list('../workspaces.txt')

totals = {
    'migrated': 0,
    'fixed_errors': 0,
    'unfixed_errors': 0,
}
workspace_summaries = {}
schema_summaries = {}
by_category = {}
for file in os.listdir('.'):
    if file.endswith('.csv'):
        match = re.search(r'^(.+)\.(\w+)\.(\w+)\.csv$', file)
        if match is None:
            raise ValueError(f"File {file} does not match expected pattern")

        workspace_name, workspace_id, change_set_id = match.groups()
        if workspace_id in deleted_workspaces:
            continue

        workspace_summaries[workspace_id] = {
            'name': workspace_name,
            'id': workspace_id,
            'migrated': 0,
            'fixed_errors': 0,
            'unfixed_errors': 0,
        }
        # Read the CSV file
        with open(file, 'r') as f:
            reader = csv.DictReader(f)
            rows = list(reader)

        for row in rows:
            category = row.get('ErrorCategory')
            if not category:
                category = f"{row['SourceSocket']} on {row['SourceSchema']} --> {row['DestSocket']} on {row['DestSchema']}: {row['Error']}"
            if row['SourceSchema'] not in schema_summaries:
                schema_summaries[row['SourceSchema']] = {
                    'fixed_errors': 0,
                    'unfixed_errors': 0,
                    'migrated': 0,
                }
            if row['DestSchema'] not in schema_summaries:
                schema_summaries[row['DestSchema']] = {
                    'fixed_errors': 0,
                    'unfixed_errors': 0,
                    'migrated': 0,
                }
            if category not in by_category:
                by_category[category] = {
                    'migrated': 0,
                    'errors': 0,
                }
            if row['Error'] and not row.get('ErrorCategory', '').startswith('FIXED-'):
                by_category[category]['errors'] += 1
                if row['ErrorCategory'] and (row['ErrorCategory'].startswith('ENG-') or row['ErrorCategory'].startswith('NEW:')):
                    totals['fixed_errors'] += 1
                    workspace_summaries[workspace_id]['fixed_errors'] += 1
                    schema_summaries[row['SourceSchema']]['fixed_errors'] += 1
                    schema_summaries[row['DestSchema']]['fixed_errors'] += 1
                else:
                    totals['unfixed_errors'] += 1
                    workspace_summaries[workspace_id]['unfixed_errors'] += 1
                    schema_summaries[row['SourceSchema']]['unfixed_errors'] += 1
                    schema_summaries[row['DestSchema']]['unfixed_errors'] += 1
            else:
                by_category[category]['migrated'] += 1
                totals['migrated'] += 1
                workspace_summaries[workspace_id]['migrated'] += 1
                schema_summaries[row['SourceSchema']]['migrated'] += 1
                schema_summaries[row['DestSchema']]['migrated'] += 1

for category, summary in by_category.items():
    print(f"category: {summary['errors']} errors, {summary['migrated']} migrated: {category}")

for workspace_id, summary in workspace_summaries.items():
    print(f"workspace: {summary['unfixed_errors']} unfixed errors, {summary['fixed_errors']} fixed errors, {summary['migrated']} migrated: {summary['name']} ({workspace_id})")

for schema, summary in schema_summaries.items():
    print(f"schema: {summary['unfixed_errors']} unfixed errors, {summary['fixed_errors']} fixed errors, {summary['migrated']} migrated: {schema}")

workspace_counts = {
    'unfixed_errors': len([ws for ws in workspace_summaries.values() if ws['unfixed_errors'] > 0]),
    'fixed_errors': len([ws for ws in workspace_summaries.values() if ws['unfixed_errors'] == 0 and ws['fixed_errors'] > 0]),
    'migrated': len([ws for ws in workspace_summaries.values() if ws['unfixed_errors'] == 0 and ws['fixed_errors'] == 0 and ws['migrated'] > 0]),
    'no_connections': len([ws for ws in workspace_summaries.values() if ws['unfixed_errors'] == 0 and ws['fixed_errors'] == 0 and ws['migrated'] == 0]),
}
print(f"workspace_counts: {workspace_counts['unfixed_errors']} with unfixed errors, {workspace_counts['fixed_errors']} with fixed errors, {workspace_counts['migrated']} migrated, {workspace_counts['no_connections']} with no connections, {len(workspace_summaries)} total")
print(f"total: {totals['unfixed_errors']} unfixed errors, {totals['fixed_errors']} fixed errors, {totals['migrated']} migrated, {totals['migrated'] + totals['fixed_errors'] + totals['unfixed_errors']} total")
