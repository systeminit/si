# GCP Smoke Test

This document provides instructions for testing Google Cloud Platform components in System Initiative.

## Prerequisites

### 1. Regenerate and Upload Specs

Before running tests, regenerate the GCP specs and upload them:

```bash
# Generate specs
deno task run generate-specs --provider="google cloud"

# Upload specs (from the bin/clover directory)
# Create a single change set for all schema uploads
export UPLOAD_CS_ID=$(buck2 run //bin/si:si change-set create --name "GCP Schema Upload $(date -Iseconds)" | jq -r '.id')

# Upload all schemas to the same change set
buck2 run //bin/si:si schema upload --from-file si-specs/Google\ Cloud\ Compute\ Engine\ Networks.json --change-set-id "$UPLOAD_CS_ID"
buck2 run //bin/si:si schema upload --from-file si-specs/Google\ Cloud\ Pub.Sub\ Topics.json --change-set-id "$UPLOAD_CS_ID"
buck2 run //bin/si:si schema upload --from-file si-specs/Google\ Cloud\ Compute\ Engine\ Disks.json --change-set-id "$UPLOAD_CS_ID"
buck2 run //bin/si:si schema upload --from-file si-specs/Google\ Cloud\ Compute\ Engine\ Addresses.json --change-set-id "$UPLOAD_CS_ID"
buck2 run //bin/si:si schema upload --from-file si-specs/Google\ Cloud\ API\ Keys\ Keys.json --change-set-id "$UPLOAD_CS_ID"

# Apply the change set
buck2 run //bin/si:si change-set apply --change-set-id "$UPLOAD_CS_ID"
```

**Important**: Upload all schemas in a single change set, then apply it. This prevents cluttering the workspace with multiple open schema upload change sets.

If components using these schemas already exist in the workspace, you'll need to upgrade them after applying the schema change set to use the new schema versions.

### 2. Check for Credentials

Check if a `Google Cloud Credential` component exists in the workspace. If not, ask the user to create one before proceeding.

### 3. Gather Test Configuration

Prompt the user for:
- **Naming prefix**: A unique prefix for test resources (default: `si-smoke-{timestamp}-`)
- **Zone**: GCP zone for zonal resources (default: `us-central1-a`)
- **Region**: GCP region for regional resources (default: `us-central1`)

The project ID will be extracted from the credential's service account JSON.

---

## Test Assets

This smoke test covers 5 GCP assets that exercise different code paths:

| # | Schema Name | Scope | API Pattern | Tests Discover |
|---|-------------|-------|-------------|----------------|
| 1 | Google Cloud Compute Engine Networks | Global | Simple `{param}` | Yes |
| 2 | Google Cloud Pub/Sub Topics | Project | Reserved `{+name}`, wrapper field | Yes |
| 3 | Google Cloud Compute Engine Disks | Zonal | Simple `{param}` with zone | Yes |
| 4 | Google Cloud Compute Engine Addresses | Regional | Simple `{param}` with region | Yes |
| 5 | Google Cloud API Keys Keys | Project | Reserved `{+parent}` | Yes |

---

## Test Phases

Run all tests in a single change set named "GCP Smoke Test - {timestamp}".

### Phase 1: Create All Resources

Create all 5 resources in the change set, then apply and wait for all Create actions to succeed.

#### 1.1 Compute Engine Network (VPC)

```
Schema: Google Cloud Compute Engine Networks
Component Name: {prefix}vpc
Attributes:
  /domain/name: {prefix}vpc
  /domain/autoCreateSubnetworks: false
  /secrets/Google Cloud Credential: subscription to credential component
```

#### 1.2 Pub/Sub Topic

```
Schema: Google Cloud Pub/Sub Topics
Component Name: {prefix}topic
Attributes:
  /domain/name: projects/{project-id}/topics/{prefix}topic
  /secrets/Google Cloud Credential: subscription to credential component
```

**Note**: The `/domain/name` must be the full resource path including `projects/{project-id}/topics/`.

#### 1.3 Compute Engine Disk

```
Schema: Google Cloud Compute Engine Disks
Component Name: {prefix}disk
Attributes:
  /domain/name: {prefix}disk
  /domain/zone: {zone}
  /domain/sizeGb: "10"
  /domain/type: projects/{project-id}/zones/{zone}/diskTypes/pd-standard
  /secrets/Google Cloud Credential: subscription to credential component
```

**Note**: The `/domain/type` must be the full URL path, not just `pd-standard`.

#### 1.4 Compute Engine Address

```
Schema: Google Cloud Compute Engine Addresses
Component Name: {prefix}address
Attributes:
  /domain/name: {prefix}address
  /domain/region: {region}
  /secrets/Google Cloud Credential: subscription to credential component
```

#### 1.5 API Key

```
Schema: Google Cloud API Keys Keys
Component Name: {prefix}apikey
Attributes:
  /domain/displayName: {prefix}apikey
  /secrets/Google Cloud Credential: subscription to credential component
```

After creating all components, apply the change set and verify all 5 Create actions succeed.

---

### Phase 2: Update Resources

Create a new change set for updates. Make minor changes to resources that support updates.

**Note**: Only 3 of the 5 resources can be updated via the generic Update action.

#### 2.1 Update Network

```
Update: /domain/routingConfig/routingMode: "GLOBAL"
```

#### 2.2 Update Pub/Sub Topic

```
Update: /domain/labels/smoke-test: "true"
```

#### 2.3 Disk

**Skipped**: Compute Disks use a separate `setLabels` API method that isn't supported by the generic Update action.

#### 2.4 Address

**Skipped**: Compute Addresses do not have an Update action - they are immutable after creation.

#### 2.5 Update API Key

```
Update: /domain/displayName: {prefix}apikey-updated
```

Apply the change set and verify all 3 Update actions succeed (Network, Topic, API Key).

---

### Phase 3: Discover Resources

Create a new change set for discovery. Test discover on all 5 schemas.

#### 3.1 Discover Networks

Run discover for `Google Cloud Compute Engine Networks`. Verify the test VPC is found among the discovered resources.

#### 3.2 Discover Pub/Sub Topics

Run discover for `Google Cloud Pub/Sub Topics`. Verify the test topic is found among the discovered resources.

#### 3.3 Discover Disks

Run discover for `Google Cloud Compute Engine Disks` with zone filter set to `{zone}`. Verify the test disk is found.

#### 3.4 Discover Addresses

Run discover for `Google Cloud Compute Engine Addresses` with region filter set to `{region}`. Verify the test address is found.

#### 3.5 Discover API Keys

Run discover for `Google Cloud API Keys Keys`. Verify the test API key is found among the discovered resources.

Abandon this change set after verifying discovery works (don't merge discovered components).

---

### Phase 4: Import Resources

Create a new change set for import testing. Import each resource by its resource ID.

For each resource:
1. Get the resource ID from the component's `/resource_value` or from the Create action result
2. Create a new component using component-import with that resource ID
3. Verify the imported component has the correct attributes

#### Resource IDs

| Resource | Resource ID Location |
|----------|---------------------|
| Network | Numeric ID (e.g., `6029756069415537918`) |
| Pub/Sub Topic | Full path (e.g., `projects/{project}/topics/{name}`) |
| Disk | Numeric ID (e.g., `7547182325923367760`) |
| Address | Numeric ID from resource |
| API Key | Full path (e.g., `projects/{project}/locations/global/keys/{uid}`) |

#### Import Requirements

**Important**: Zonal and regional resources require location attributes during import:

- **Disk Import**: Must set `/domain/zone` attribute (e.g., `"us-central1-a"`)
- **Address Import**: Must set `/domain/region` attribute (e.g., `"us-central1"`)

Without these attributes, the import will fail with "Cannot build API URL - unresolved parameters" error.

Abandon this change set after verifying imports work.

---

### Phase 5: Delete All Resources

Create a new change set for deletion. Delete all 5 test components, apply, and verify all Destroy actions succeed.

After successful deletion, verify only the credential component remains in HEAD.

---

## Known Issues

### Disk Type Requires Full URL

The `/domain/type` attribute for Compute Engine Disks requires the full URL path:

- **Wrong**: `pd-standard`
- **Correct**: `projects/{project}/zones/{zone}/diskTypes/pd-standard`

The GCP API returns error 400 "Invalid value for field 'resource.type': The URL is malformed" if you use the short form.

### Pub/Sub Topic Name Requires Full Path

The `/domain/name` attribute for Pub/Sub Topics must include the full resource path:

- **Wrong**: `my-topic`
- **Correct**: `projects/{project-id}/topics/my-topic`

---

## Expected Errors

### Success Indicators

- Create/Update/Delete actions return `funcRunState: "Success"` with `status: "ok"` in the result
- Discover returns a message like "Discovered N resources"
- Import returns `message: "Imported Resource"`

### Common Failure Patterns

| Error | Likely Cause |
|-------|--------------|
| `404 Not Found` on refresh | Resource was deleted outside SI or never created |
| `400 Bad Request` with "URL is malformed" | Missing full URL path (see Known Issues) |
| `403 Forbidden` | Credential lacks required IAM permissions |
| `409 Conflict` | Resource already exists with that name |
| `429 Too Many Requests` | Rate limited - will auto-retry |
| Unresolved parameters in URL | Required property not set (zone, region, parent, etc.) |

### Checking Failures

When an action fails, use `func-run-get` with `logs: true` and `result: true` to see the actual error from the GCP API.

---

## Test Results Template

After running the smoke test, record results:

```
Date: 
Credential: 
Prefix Used: 

Phase 1 - Create:
  [ ] Network: 
  [ ] Pub/Sub Topic: 
  [ ] Disk: 
  [ ] Address: 
  [ ] API Key: 

Phase 2 - Update:
  [ ] Network: 
  [ ] Pub/Sub Topic: 
  [N/A] Disk: (uses separate setLabels API)
  [N/A] Address: (immutable - no Update action)
  [ ] API Key: 

Phase 3 - Discover:
  [ ] Networks: 
  [ ] Pub/Sub Topics: 
  [ ] Disks: 
  [ ] Addresses: 
  [ ] API Keys: 

Phase 4 - Import:
  [ ] Network: 
  [ ] Pub/Sub Topic: 
  [ ] Disk: 
  [ ] Address: 
  [ ] API Key: 

Phase 5 - Delete:
  [ ] Network: 
  [ ] Pub/Sub Topic: 
  [ ] Disk: 
  [ ] Address: 
  [ ] API Key: 

Issues Found:
```
