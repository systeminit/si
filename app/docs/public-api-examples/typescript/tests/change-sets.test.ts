import { describe, it, expect, beforeAll } from 'vitest';
import { Configuration, ChangeSetsApi, CreateChangeSetV1Request } from 'system-initiative-api-client';

describe('Change Sets API', () => {
  let changeSetsApi: ChangeSetsApi;
  let workspaceId: string;

  beforeAll(() => {
    const apiToken = process.env.SI_API_TOKEN;
    const basePath = process.env.SI_API_BASE_PATH || 'https://api.systeminit.com';
    workspaceId = process.env.SI_WORKSPACE_ID || '';

    if (!apiToken) {
      throw new Error('SI_API_TOKEN environment variable is required');
    }

    if (!workspaceId) {
      throw new Error('SI_WORKSPACE_ID environment variable is required');
    }

    const config = new Configuration({
      basePath,
      baseOptions: {
        headers: {
          Authorization: `Bearer ${apiToken}`,
        },
      }
    });

    changeSetsApi = new ChangeSetsApi(config);
  });

  it('should create a new change set', async () => {
    const changeSetName = `test-changeset-${Date.now()}`;
    const request: CreateChangeSetV1Request = {
      changeSetName,
    };

    const response = await changeSetsApi.createChangeSet({
      workspaceId,
      createChangeSetV1Request: request,
    });

    expect(response.status).toBe(200);
    expect(response.data.changeSet).toBeDefined();
    expect(response.data.changeSet.name).toBe(changeSetName);
    expect(response.data.changeSet.id).toBeDefined();

    console.log('Created change set:', {
      id: response.data.changeSet.id,
      name: response.data.changeSet.name,
      status: response.data.changeSet.status,
    });
  });

  it('should list all change sets', async () => {
    const response = await changeSetsApi.listChangeSets({
      workspaceId,
    });

    expect(response.status).toBe(200);
    expect(response.data.changeSets).toBeDefined();
    expect(Array.isArray(response.data.changeSets)).toBe(true);

    console.log(`Found ${response.data.changeSets.length} change set(s)`);
  });
});
