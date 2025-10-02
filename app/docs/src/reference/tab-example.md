# Tab Component Example

This page demonstrates how to use the `DocTabs` and `TabPanel` components for organizing documentation by platform, environment, or context.

## Basic Usage

<DocTabs tabs="Web Application,AI Agent,Public API">

<TabPanel value="Web Application">

### Web Application Setup

The web application provides a visual interface for managing infrastructure.

To get started:

1. Navigate to https://app.systeminit.com
2. Create a new workspace
3. Start adding components to your canvas

```typescript
// Example: Connecting to the web socket
const ws = new WebSocket('wss://app.systeminit.com/ws');
ws.onmessage = (event) => {
  console.log('Received:', event.data);
};
```

</TabPanel>

<TabPanel value="AI Agent">

### AI Agent Integration

The AI Agent can help automate infrastructure tasks and provide intelligent suggestions.

Key features:
- Natural language infrastructure queries
- Automated component configuration
- Best practice recommendations

```typescript
// Example: Using the AI Agent API
const response = await fetch('https://api.systeminit.com/ai/query', {
  method: 'POST',
  headers: { 'Authorization': 'Bearer YOUR_TOKEN' },
  body: JSON.stringify({
    query: 'Create a VPC with 3 subnets'
  })
});
```

</TabPanel>

<TabPanel value="Public API">

### Public API Access

Use the Public API for programmatic access to System Initiative.

Authentication is required via API tokens:

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://api.systeminit.com/v1/workspaces
```

Available endpoints:
- `/v1/workspaces` - Manage workspaces
- `/v1/components` - Component operations
- `/v1/changesets` - Change set management

</TabPanel>

</DocTabs>

## Cloud Provider Example

<DocTabs tabs="AWS,Azure,GCP" defaultTab="AWS">

<TabPanel value="AWS">

### Amazon Web Services

Configure AWS credentials:

```bash
export AWS_ACCESS_KEY_ID="your-key"
export AWS_SECRET_ACCESS_KEY="your-secret"
export AWS_REGION="us-east-1"
```

</TabPanel>

<TabPanel value="Azure">

### Microsoft Azure

Configure Azure credentials:

```bash
az login
az account set --subscription "your-subscription-id"
```

</TabPanel>

<TabPanel value="GCP">

### Google Cloud Platform

Configure GCP credentials:

```bash
gcloud auth login
gcloud config set project your-project-id
```

</TabPanel>

</DocTabs>

## Syntax

```markdown
<DocTabs tabs="Tab 1,Tab 2,Tab 3" defaultTab="Tab 2">

<TabPanel value="Tab 1">

Content for Tab 1 with **markdown** support.

</TabPanel>

<TabPanel value="Tab 2">

Content for Tab 2.

</TabPanel>

<TabPanel value="Tab 3">

Content for Tab 3.

</TabPanel>

</DocTabs>
```

### Props

**DocTabs:**
- `tabs` (required): Comma-separated list of tab labels
- `defaultTab` (optional): Which tab to show by default (defaults to first tab)

**TabPanel:**
- `value` (required): Must match one of the tab labels from `DocTabs`

### Important Notes

- Leave blank lines before and after markdown content inside `<TabPanel>` tags
- The `value` in `TabPanel` is case-insensitive and spaces are converted to hyphens
- All standard markdown formatting works inside tab panels
