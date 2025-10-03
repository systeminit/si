import { describe, it, expect, beforeAll } from 'vitest';
import { Configuration, SecretsApi, ChangeSetsApi, ComponentsApi } from 'system-initiative-api-client';

describe('Secrets API', () => {
  let secretsApi: SecretsApi;
  let changeSetsApi: ChangeSetsApi;
  let componentsApi: ComponentsApi;
  let workspaceId: string;
  let changeSetId: string;

  beforeAll(async () => {
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

    secretsApi = new SecretsApi(config);
    changeSetsApi = new ChangeSetsApi(config);
    componentsApi = new ComponentsApi(config);

    // Create a change set for testing
    const changeSetName = `test-secrets-${Date.now()}`;
    const changeSetResponse = await changeSetsApi.createChangeSet({
      workspaceId,
      createChangeSetV1Request: {
        changeSetName,
      },
    });

    changeSetId = changeSetResponse.data.changeSet.id;
  });

  it('should create an AWS credential secret', async () => {
    const response = await secretsApi.createSecret({
      workspaceId,
      changeSetId,
      createSecretV1Request: {
        name: 'my-aws-credentials',
        definitionName: 'AWS Credential',
        description: 'My AWS access credentials',
        rawData: {
          'accessKeyId': 'AKIAIOSFODNN7EXAMPLE',
          'secretAccessKey': 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY'
        }
      }
    });

    expect(response.status).toBe(200);
    expect(response.data.secret).toBeDefined();
    expect(response.data.secret.name).toBe('my-aws-credentials');
    expect(response.data.secret.definition).toBe('AWS Credential');
    expect(response.data.secret.description).toBe('My AWS access credentials');
  });

  it('should create an AWS credential secret and subscribe an EC2 instance to it', async () => {
    // Create the AWS credential secret
    const secretResponse = await secretsApi.createSecret({
      workspaceId,
      changeSetId,
      createSecretV1Request: {
        name: 'test-aws-ec2-credentials',
        definitionName: 'AWS Credential',
        description: 'AWS credentials for EC2 instance',
        rawData: {
          'accessKeyId': 'AKIAIOSFODNN7EXAMPLE',
          'secretAccessKey': 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY'
        }
      }
    });

    expect(secretResponse.status).toBe(200);
    expect(secretResponse.data.secret).toBeDefined();

    const secretId = secretResponse.data.secret.id;

    // Create an AWS EC2 Instance component
    const componentResponse = await componentsApi.createComponent({
      workspaceId,
      changeSetId,
      createComponentV1Request: {
        name: 'test-ec2-instance',
        schemaName: 'AWS::EC2::Instance',
        attributes: {
          "/secrets/AWS Credential": {
            component: secretId,
            path: "/secrets/AWS Credential",
          },
        }
      }
    });

    expect(componentResponse.status).toBe(200);
    expect(componentResponse.data.component).toBeDefined();
  }, 30000);

  it('should update a secret with new data', async () => {
    // Create the initial secret
    const createResponse = await secretsApi.createSecret({
      workspaceId,
      changeSetId,
      createSecretV1Request: {
        name: 'updatable-aws-credentials',
        definitionName: 'AWS Credential',
        description: 'Initial credentials',
        rawData: {
          'accessKeyId': 'AKIAIOSFODNN7EXAMPLE',
          'secretAccessKey': 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY'
        }
      }
    });

    expect(createResponse.status).toBe(200);
    expect(createResponse.data.secret).toBeDefined();

    const secretId = createResponse.data.secret.id;

    // Update the secret with new data
    const updateResponse = await secretsApi.updateSecret({
      workspaceId,
      changeSetId,
      secretId,
      updateSecretV1Request: {
        id: secretId,
        name: 'updated-aws-credentials',
        description: 'Updated credentials with new keys',
        rawData: {
          'accessKeyId': 'AKIAIOSFODNN7NEWKEY',
          'secretAccessKey': 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYNEWKEY'
        }
      }
    });

    expect(updateResponse.status).toBe(200);
    expect(updateResponse.data.secret).toBeDefined();
    expect(updateResponse.data.secret.id).toBe(secretId);
    expect(updateResponse.data.secret.name).toBe('updated-aws-credentials');
    expect(updateResponse.data.secret.description).toBe('Updated credentials with new keys');
  });

  it('should delete a secret', async () => {
    // Create a secret to delete
    const createResponse = await secretsApi.createSecret({
      workspaceId,
      changeSetId,
      createSecretV1Request: {
        name: 'deletable-aws-credentials',
        definitionName: 'AWS Credential',
        description: 'Credentials to be deleted',
        rawData: {
          'accessKeyId': 'AKIAIOSFODNN7EXAMPLE',
          'secretAccessKey': 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY'
        }
      }
    });

    expect(createResponse.status).toBe(200);
    expect(createResponse.data.secret).toBeDefined();

    const secretId = createResponse.data.secret.id;

    // Delete the secret
    const deleteResponse = await secretsApi.deleteSecret({
      workspaceId,
      changeSetId,
      secretId
    });

    expect(deleteResponse.status).toBe(200);
    expect(deleteResponse.data.success).toBe(true);
  });
});
