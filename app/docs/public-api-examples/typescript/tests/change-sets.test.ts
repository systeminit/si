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

  it('should apply a change set', async () => {
    // Create a change set to apply
    const changeSetName = `test-apply-${Date.now()}`;
    const createResponse = await changeSetsApi.createChangeSet({
      workspaceId,
      createChangeSetV1Request: { changeSetName },
    });

    expect(createResponse.status).toBe(200);
    const changeSetId = createResponse.data.changeSet.id;

    console.log('Created change set to apply:', {
      id: changeSetId,
      name: changeSetName,
    });

    // Apply the change set
    const applyResponse = await changeSetsApi.forceApply({
      workspaceId,
      changeSetId,
    });

    expect(applyResponse.status).toBe(200);
    expect(applyResponse.data.success).toBe(true);

    console.log('Applied change set:', changeSetId);
  });

  it('should abandon a change set', async () => {
    // Create a change set to abandon
    const changeSetName = `test-abandon-${Date.now()}`;
    const createResponse = await changeSetsApi.createChangeSet({
      workspaceId,
      createChangeSetV1Request: { changeSetName },
    });

    expect(createResponse.status).toBe(200);
    const changeSetId = createResponse.data.changeSet.id;

    console.log('Created change set to abandon:', {
      id: changeSetId,
      name: changeSetName,
    });

    // Abandon the change set
    const abandonResponse = await changeSetsApi.abandonChangeSet({
      workspaceId,
      changeSetId,
    });

    expect(abandonResponse.status).toBe(200);
    expect(abandonResponse.data.success).toBe(true);

    console.log('Abandoned change set:', changeSetId);
  });
});
