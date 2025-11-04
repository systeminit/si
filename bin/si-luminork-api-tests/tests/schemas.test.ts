/**
 * Schemas Tests
 *
 * Tests for the schemas API endpoints.
 */

import { assertEquals, assertExists } from 'https://deno.land/std@0.220.1/assert/mod.ts';
import { cleanupTestResources, ConfigError, createTestClient } from '../src/test-utils.ts';
import type { SchemaView } from '../src/api/schemas.ts';
import { LuminorkApi } from '../src/api/index.ts';

/**
 * Helper function to find the first installed schema in a change set
 */
async function findInstalledSchema(
  api: LuminorkApi,
  workspaceId: string,
  changeSetId: string,
): Promise<SchemaView | null> {
  const listSchemasResponse = await api.schemas.listSchemas(
    workspaceId,
    changeSetId,
  );

  assertEquals(listSchemasResponse.status, 200);
  assertExists(listSchemasResponse.data.schemas);

  if (listSchemasResponse.data.schemas.length === 0) {
    console.warn('No schemas found in workspace');
    return null;
  }

  const firstSchema = listSchemasResponse.data.schemas.find(
    (s) => s.installed,
  );
  if (!firstSchema) {
    console.warn('No installed schemas found');
    return null;
  }

  console.log(
    `Found installed schema: ${firstSchema.schemaName} (${firstSchema.schemaId})`,
  );

  return firstSchema;
}

/**
 * Helper function to unlock a schema
 */
async function unlockSchema(
  api: LuminorkApi,
  workspaceId: string,
  changeSetId: string,
  schemaId: string,
) {
  const unlockResponse = await api.schemas.unlockSchema(
    workspaceId,
    changeSetId,
    schemaId,
  );

  assertEquals(
    unlockResponse.status,
    200,
    'Unlock schema response status should be 200',
  );
  assertExists(unlockResponse.data.unlockedVariantId);
  assertEquals(
    unlockResponse.data.unlockedVariant.isLocked,
    false,
    'Unlocked variant should not be locked',
  );

  console.log(
    `Successfully unlocked schema, unlocked variant ID: ${unlockResponse.data.unlockedVariantId}`,
  );

  return unlockResponse.data;
}

/**
 * Helper function to update a schema variant with test values
 */
async function updateSchemaVariantWithTestValues(
  api: LuminorkApi,
  workspaceId: string,
  changeSetId: string,
  schemaId: string,
  variantId: string,
  displayName: string,
) {
  const newCode = `function main() { const asset = new AssetBuilder(); return asset.build(); }`;

  const updateResponse = await api.schemas.updateSchemaVariant(
    workspaceId,
    changeSetId,
    {
      schema_id: schemaId,
      schema_variant_id: variantId,
      request_body: {
        name: displayName,
        category: 'TestCategory',
        color: '#FF00FF',
        description: 'Test description for unlocked schema',
        link: 'https://example.com/test-schema',
        code: newCode,
      },
    },
  );

  assertEquals(
    updateResponse.status,
    200,
    'Update schema variant response status should be 200',
  );
  assertExists(updateResponse.data);

  // Verify the updated values
  assertEquals(
    updateResponse.data.category,
    'TestCategory',
    'Category should be updated',
  );
  assertEquals(
    updateResponse.data.color,
    '#FF00FF',
    'Color should be updated',
  );
  assertEquals(
    updateResponse.data.description,
    'Test description for unlocked schema',
    'Description should be updated',
  );
  assertEquals(
    updateResponse.data.link,
    'https://example.com/test-schema',
    'Link should be updated',
  );

  console.log(
    `Successfully updated unlocked schema variant with new category: ${updateResponse.data.category}`,
  );

  return updateResponse.data;
}

Deno.test('Schemas API - List and Find Schemas', async () => {
  try {
    const { api, config } = await createTestClient();

    const createdChangeSetIds: string[] = [];

    try {
      // Create a change set for testing
      const createChangeSetResponse = await api.changeSets.createChangeSet(
        config.workspaceId,
        {
          changeSetName: 'schema_test_changeset',
        },
      );

      assertEquals(createChangeSetResponse.status, 200);
      assertExists(createChangeSetResponse.data.changeSet);
      const changeSetId = createChangeSetResponse.data.changeSet.id;
      createdChangeSetIds.push(changeSetId);

      console.log(`Created change set with ID: ${changeSetId}`);

      // List all schemas
      const listSchemasResponse = await api.schemas.listSchemas(
        config.workspaceId,
        changeSetId,
      );

      assertEquals(listSchemasResponse.status, 200);
      assertExists(listSchemasResponse.data.schemas);

      if (listSchemasResponse.data.schemas.length === 0) {
        console.warn('No schemas found in workspace, test is limited');
        return;
      }

      // Get the first schema for further testing
      const firstSchema = listSchemasResponse.data.schemas[0];
      assertExists(firstSchema.schemaId);
      assertExists(firstSchema.schemaName);

      console.log(
        `Found schema with ID: ${firstSchema.schemaId} and name: ${firstSchema.schemaName}`,
      );
      console.log(`Testing find schema by name: ${firstSchema.schemaName}`);

      const findSchemaResponse = await api.schemas.findSchema(
        config.workspaceId,
        { name: firstSchema.schemaName },
        changeSetId,
      );

      assertEquals(
        findSchemaResponse.status,
        200,
        'Find schema response status should be 200',
      );

      // Check if our schema is in the results
      const schemaFound = findSchemaResponse.data.schemaId === firstSchema.schemaId;

      assertEquals(
        schemaFound,
        true,
        `Schema ${firstSchema.schemaId} not found in search results`,
      );

      // Ensure we install the schema if it doesn't exist so we can guarantee we can get the schema details
      await api.components.createComponent(config.workspaceId, changeSetId, {
        name: 'Ensuring Schema Installed',
        schemaName: firstSchema.schemaName,
      });

      // Get specific schema
      const getSchemaResponse = await api.schemas.getSchema(
        config.workspaceId,
        changeSetId,
        firstSchema.schemaId,
      );
      assertEquals(getSchemaResponse.status, 200);
      assertEquals(getSchemaResponse.data.name, firstSchema.schemaName); // May need to adjust this based on actual response

      if (
        getSchemaResponse.data.variantIds &&
        getSchemaResponse.data.variantIds.length > 0
      ) {
        const variantId = getSchemaResponse.data.variantIds[0];

        // Retry logic for handling 202 responses
        let getVariantResponse;
        let attempts = 0;
        const maxAttempts = 5;
        const retryDelayMs = 1000; // 1 second

        do {
          getVariantResponse = await api.schemas.getSchemaVariant(
            config.workspaceId,
            changeSetId,
            firstSchema.schemaId,
            variantId,
          );

          if (getVariantResponse.status === 200) {
            break;
          } else if (getVariantResponse.status === 202) {
            attempts++;
            if (attempts < maxAttempts) {
              console.log(`Got 202 response, retrying in ${retryDelayMs}ms (attempt ${attempts}/${maxAttempts})`);
              await new Promise(resolve => setTimeout(resolve, retryDelayMs));
            }
          } else {
            break; // Exit on unexpected status codes
          }
        } while (getVariantResponse.status === 202 && attempts < maxAttempts);

        assertEquals(getVariantResponse.status, 200);
        assertEquals(getVariantResponse.data.variantId, variantId);

        console.log(
          `Successfully retrieved schema variant with ID: ${variantId}`,
        );
      }
    } finally {
      // Clean up any change sets
      await cleanupTestResources(api, config.workspaceId, createdChangeSetIds);
    }
  } catch (error: unknown) {
    if (error instanceof ConfigError) {
      console.warn(
        `Skipping test due to configuration error: ${error.message}`,
      );
      return;
    }
    throw error;
  }
});

Deno.test("Schemas API - Ensure we don't get a 202 on finding and getting a schema", async () => {
  try {
    const { api, config } = await createTestClient();

    const createdChangeSetIds: string[] = [];

    try {
      // Create a change set for testing
      const createChangeSetResponse = await api.changeSets.createChangeSet(
        config.workspaceId,
        {
          changeSetName: 'schema_test_changeset',
        },
      );

      assertEquals(createChangeSetResponse.status, 200);
      assertExists(createChangeSetResponse.data.changeSet);
      const changeSetId = createChangeSetResponse.data.changeSet.id;
      createdChangeSetIds.push(changeSetId);

      console.log(`Created change set with ID: ${changeSetId}`);

      const findSchemaResponse = await api.schemas.findSchema(
        config.workspaceId,
        { name: 'AWS::Neptune::DBCluster' },
        changeSetId,
      );

      assertEquals(
        findSchemaResponse.status,
        200,
        'Find schema response status should be 200',
      );
      assertEquals(
        findSchemaResponse.data.schemaName,
        'AWS::Neptune::DBCluster',
      );
      assertEquals(findSchemaResponse.data.category, 'AWS::Neptune');
      assertEquals(findSchemaResponse.data.installed, false);

      const getSchemaResponse = await api.schemas.getSchema(
        config.workspaceId,
        changeSetId,
        findSchemaResponse.data.schemaId,
      );
      assertEquals(
        getSchemaResponse.status,
        200,
        'Get schema response status should be 200',
      );
      assertEquals(
        getSchemaResponse.data.name,
        'AWS::Neptune::DBCluster',
        'Ensure we have a schema name that matches',
      );

      const getDefaultSchemaResponse = await api.schemas.getDefaultSchemaVariant(
        config.workspaceId,
        changeSetId,
        findSchemaResponse.data.schemaId,
      );
      assertEquals(
        getDefaultSchemaResponse.status,
        200,
        'Get schema response status should be 200',
      );
    } finally {
      // Clean up any change sets
      await cleanupTestResources(
        api,
        config.workspaceId,
        createdChangeSetIds,
      );
    }
  } catch (error: unknown) {
    if (error instanceof ConfigError) {
      console.warn(
        `Skipping test due to configuration error: ${error.message}`,
      );
      return;
    }
    throw error;
  }
});

Deno.test('Schemas API - Unlocking a schema', async () => {
  try {
    const { api, config } = await createTestClient();

    const createdChangeSetIds: string[] = [];

    try {
      // Create a change set for testing
      const createChangeSetResponse = await api.changeSets.createChangeSet(
        config.workspaceId,
        {
          changeSetName: 'unlock_schema_test_changeset',
        },
      );

      assertEquals(createChangeSetResponse.status, 200);
      assertExists(createChangeSetResponse.data.changeSet);
      const changeSetId = createChangeSetResponse.data.changeSet.id;
      createdChangeSetIds.push(changeSetId);

      console.log(`Created change set with ID: ${changeSetId}`);

      // Find an installed schema
      const schema = await findInstalledSchema(api, config.workspaceId, changeSetId);
      if (!schema) {
        console.warn('Test is limited - no installed schemas found');
        return;
      }

      // Get the schema details to find its default variant
      const getSchemaResponse = await api.schemas.getSchema(
        config.workspaceId,
        changeSetId,
        schema.schemaId,
      );
      assertEquals(getSchemaResponse.status, 200);
      assertExists(getSchemaResponse.data.defaultVariantId);

      const defaultVariantId = getSchemaResponse.data.defaultVariantId;

      // Get the default variant to check if it's locked
      const getVariantResponse = await api.schemas.getSchemaVariant(
        config.workspaceId,
        changeSetId,
        schema.schemaId,
        defaultVariantId,
      );

      assertEquals(getVariantResponse.status, 200);
      assertExists(getVariantResponse.data);

      console.log(
        `Default variant is locked: ${getVariantResponse.data.isLocked}`,
      );

      // Unlock the schema
      const unlockedData = await unlockSchema(
        api,
        config.workspaceId,
        changeSetId,
        schema.schemaId,
      );

      assertEquals(
        unlockedData.schemaId,
        schema.schemaId,
        'Schema ID should match',
      );

      // If the original variant was locked, verify we got a new variant
      if (getVariantResponse.data.isLocked) {
        assertEquals(
          unlockedData.unlockedVariantId !== defaultVariantId,
          true,
          'Unlocked variant should be different from locked default variant',
        );
      } else {
        // If it was already unlocked, we should get the same variant back
        assertEquals(
          unlockedData.unlockedVariantId,
          defaultVariantId,
          'Should return existing unlocked variant',
        );
      }
    } finally {
      // Clean up any change sets
      await cleanupTestResources(api, config.workspaceId, createdChangeSetIds);
    }
  } catch (error: unknown) {
    if (error instanceof ConfigError) {
      console.warn(
        `Skipping test due to configuration error: ${error.message}`,
      );
      return;
    }
    throw error;
  }
});

Deno.test('Schemas API - Editing an unlocked schema with no components', async () => {
  try {
    const { api, config } = await createTestClient();

    const createdChangeSetIds: string[] = [];

    try {
      // Create a change set for testing
      const createChangeSetResponse = await api.changeSets.createChangeSet(
        config.workspaceId,
        {
          changeSetName: 'edit_unlocked_schema_test_changeset',
        },
      );

      assertEquals(createChangeSetResponse.status, 200);
      assertExists(createChangeSetResponse.data.changeSet);
      const changeSetId = createChangeSetResponse.data.changeSet.id;
      createdChangeSetIds.push(changeSetId);

      console.log(`Created change set with ID: ${changeSetId}`);

      // Find an installed schema
      const schema = await findInstalledSchema(api, config.workspaceId, changeSetId);
      if (!schema) {
        console.warn('Test is limited - no installed schemas found');
        return;
      }

      // Unlock the schema
      const unlockedData = await unlockSchema(
        api,
        config.workspaceId,
        changeSetId,
        schema.schemaId,
      );

      // Update the unlocked variant with test values
      await updateSchemaVariantWithTestValues(
        api,
        config.workspaceId,
        changeSetId,
        schema.schemaId,
        unlockedData.unlockedVariantId,
        unlockedData.unlockedVariant.displayName,
      );
    } finally {
      // Clean up any change sets
      await cleanupTestResources(api, config.workspaceId, createdChangeSetIds);
    }
  } catch (error: unknown) {
    if (error instanceof ConfigError) {
      console.warn(
        `Skipping test due to configuration error: ${error.message}`,
      );
      return;
    }
    throw error;
  }
});

Deno.test('Schemas API - Editing an unlocked schema with components', async () => {
  try {
    const { api, config } = await createTestClient();

    const createdChangeSetIds: string[] = [];

    try {
      // Create a change set for testing
      const createChangeSetResponse = await api.changeSets.createChangeSet(
        config.workspaceId,
        {
          changeSetName: 'edit_unlocked_schema_with_components_changeset',
        },
      );

      assertEquals(createChangeSetResponse.status, 200);
      assertExists(createChangeSetResponse.data.changeSet);
      const changeSetId = createChangeSetResponse.data.changeSet.id;
      createdChangeSetIds.push(changeSetId);

      console.log(`Created change set with ID: ${changeSetId}`);

      // Find an installed schema
      const schema = await findInstalledSchema(api, config.workspaceId, changeSetId);
      if (!schema) {
        console.warn('Test is limited - no installed schemas found');
        return;
      }

      // Create three components before unlocking
      console.log('Creating 3 components before unlocking...');
      for (let i = 1; i <= 3; i++) {
        const createComponentResponse = await api.components.createComponent(
          config.workspaceId,
          changeSetId,
          {
            name: `Test Component Before Unlock ${i}`,
            schemaName: schema.schemaName,
          },
        );
        assertEquals(createComponentResponse.status, 200);
        console.log(
          `Created component ${i}: ${createComponentResponse.data.component.id}`,
        );
      }

      // Unlock the schema
      const unlockedData = await unlockSchema(
        api,
        config.workspaceId,
        changeSetId,
        schema.schemaId,
      );

      // Update the unlocked variant with test values
      await updateSchemaVariantWithTestValues(
        api,
        config.workspaceId,
        changeSetId,
        schema.schemaId,
        unlockedData.unlockedVariantId,
        unlockedData.unlockedVariant.displayName,
      );

      // Create three more components after editing using the working copy
      console.log('Creating 3 components after editing...');
      for (let i = 1; i <= 3; i++) {
        const createComponentResponse = await api.components.createComponent(
          config.workspaceId,
          changeSetId,
          {
            name: `Test Component After Edit ${i}`,
            schemaName: schema.schemaName,
            useWorkingCopy: true,
          },
        );
        assertEquals(createComponentResponse.status, 200);
        console.log(
          `Created component ${i}: ${createComponentResponse.data.component.id}`,
        );
      }

      console.log('Successfully created components before and after editing unlocked schema');
    } finally {
      // Clean up any change sets
      await cleanupTestResources(api, config.workspaceId, createdChangeSetIds);
    }
  } catch (error: unknown) {
    if (error instanceof ConfigError) {
      console.warn(
        `Skipping test due to configuration error: ${error.message}`,
      );
      return;
    }
    throw error;
  }
});
