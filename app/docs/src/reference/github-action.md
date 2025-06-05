# GitHub Action Reference

The System Initiative [GitHub Action](https://github.com/systeminit/actions)
allows a user to be able to do the following:

- Open a change set in a workspace
- Set properties on a component
- Execute a management function on a component
- Merge a change set (or request approval if needed)
- Poll for the status of actions resultant of a change set merge

## Action Parameters

### Required Parameters

- **apiToken:** The API token. You can [generate an API token](../explanation/generate-a-workspace-api-token) on the workspace
  details page in the [auth portal](https://auth.systeminit.com). It is
  suggested to store this as a
  [secret for the GitHub Action](https://docs.github.com/en/actions/security-for-github-actions/security-guides/using-secrets-in-github-actions)

- **componentId:** The ID of the component to run the management function on.

- **changeSet:** The name of the change set to create to make changes in.

### Optional Parameters

- **workspaceId:** The id of the workspace in which to interact. If this is not
  specified, we will retrive the workspaceId from the specific token used. If
  the workspaceId doesn't match the workspace the token is generated for then we
  will return an error.

- **domain:** An object containing the domain properties to set for the
  management function component. You may specify property keys as names,
  property IDs, or paths (e.g. `{ "name/first": "John", "name/last": "Doe" }`).
  Each property's value will be replaced with the new value. Properties not
  specified will not be changed in the component. If not specified then no
  properties will be set.

- **managementFunction:** The name of the management function to execute.
  Optional if the component has a single management function.

- **view:** The name of view in which the component should be updated and the
  context in which the management function will execute. Optional if the
  component only exists in a singe view.

- **applyOnSuccess:** Whether to apply the change set to main after triggering
  the management function. Set to `force` to force apply (if your user has
  permission to do so). Available options are: `force`, `true`, or `false`.
  Default is `true`.

- **waitForApproval:** Whether to wait for approval before applying the
  changeset (and fail if it is rejected). By default, we do not wait. If
  `applyOnSuccess` is anything but `true` (the default), this is ignored. If
  this is true, we will also wait for actions to complete unless you explicitly
  set `waitForActions` to `false`. Default is `false`.

- **waitForActions:** Whether to wait for actions, in System Initiative, to
  complete (and fail the actions run if the actions fail). By default, we wait
  for actions if `applyOnSuccess: force` or `waitForApproval=true`, otherwise is
  ignored. Default is `false`.

- **pollIntervalSeconds:** Length of time (in seconds) between checks when
  polling for change set status. Default is `10` seconds.

### Action Outputs

- **managementFunctionLogs:** The logs from the management function.

- **workspaceId:** The ID of the workspace containing the component.

- **changeSetId:** The ID of the changeset created.

- **changeSetWebUrl:** The web URL to the changeset.

- **componentWebUrl:** componentWebUrl

## Example Usage

### Basic Usage

```yaml
- uses: systeminit/actions@v0
  with:
    changeSet: CI
    componentId: 01JH3DZW0QTMH69ZA45299GSWY
    domain: |
      region: us-east-1
      cidrBlock: "10.0.0.0/16"
      tag/purpose: demo
    apiToken: ${{ secrets.SI_API_TOKEN }}
```

### Advanced Usage

```yaml
- uses: systeminit/actions@v0
  with:
    changeSet: CI
    componentId: 01JH3DZW0QTMH69ZA45299GSWY
    domain: |
      region: us-east-1
      cidrBlock: "10.0.0.0/16"
      tag/purpose: demo
    managementFunction: "UpdateDevelopmentEnvironment"
    view: dev
    waitForApproval: force
    waitForActions: true
    pollIntervalSeconds: 30
    apiToken: ${{ secrets.SI_API_TOKEN }}
```
