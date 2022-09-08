# Delete GitHub Actions Runs

## Required Tooling

- [GitHub CLI](https://cli.github.com/)
- [jq](https://stedolan.github.io/jq/)

## Fetch All Workflow Runs

```sh
gh api /repos/{owner}/{repo}/actions/runs --paginate >runs.json
```

## List All Uniq Workflow Names That Have Runs

```sh
jq -r '.workflow_runs[] | .name' <runs.json  | sort | uniq
```

## Delete All Workflow Runs By Workflow Name

```sh
jq \
  --arg name "Test Changed Components" \
  '.workflow_runs[] | select(.name == $name) | .id' \
  <runs.json \
  | while read -r run_id; do
      echo "  - Deleting workflow run $run_id"
      gh api -X DELETE "/repos/{owner}/{repo}/actions/runs/$run_id"
    done
```
