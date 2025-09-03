/**
 * Schemas Tests
 *
 * Tests for the schemas API endpoints.
 */

import {
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.220.1/assert/mod.ts";
import {
  createTestClient,
  cleanupTestResources,
  ConfigError,
} from "../src/test-utils.ts";

Deno.test("Schemas API - List and Find Schemas", async () => {
  try {
    const { api, config } = await createTestClient();

    const createdChangeSetIds: string[] = [];

    try {
      // Create a change set for testing
      const createChangeSetResponse = await api.changeSets.createChangeSet(
        config.workspaceId,
        {
          changeSetName: "schema_test_changeset",
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
        console.warn("No schemas found in workspace, test is limited");
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
        "Find schema response status should be 200",
      );

      // Check if our schema is in the results
      const schemaFound =
        findSchemaResponse.data.schemaId === firstSchema.schemaId;

      assertEquals(
        schemaFound,
        true,
        `Schema ${firstSchema.schemaId} not found in search results`,
      );

      // Ensure we install the schema if it doesn't exist so we can guarantee we can get the schema details
      await api.components.createComponent(config.workspaceId, changeSetId, {
        name: "Ensuring Schema Installed",
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

      // Test getting schema variant if available
      if (
        getSchemaResponse.data.variants &&
        getSchemaResponse.data.variants.length > 0
      ) {
        const variant = getSchemaResponse.data.variants[0];

        const getVariantResponse = await api.schemas.getSchemaVariant(
          config.workspaceId,
          changeSetId,
          firstSchema.schemaId,
          variant.id,
        );

        assertEquals(getVariantResponse.status, 200);
        assertEquals(getVariantResponse.data.id, variant.id);
        assertEquals(getVariantResponse.data.schema_id, firstSchema.schemaId);

        console.log(
          `Successfully retrieved schema variant with ID: ${variant.id}`,
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

Deno.test(
  "Schemas API - Ensure we don't get a 202 on finding and getting a schema",
  async () => {
    try {
      const { api, config } = await createTestClient();

      const createdChangeSetIds: string[] = [];

      try {
        // Create a change set for testing
        const createChangeSetResponse = await api.changeSets.createChangeSet(
          config.workspaceId,
          {
            changeSetName: "schema_test_changeset",
          },
        );

        assertEquals(createChangeSetResponse.status, 200);
        assertExists(createChangeSetResponse.data.changeSet);
        const changeSetId = createChangeSetResponse.data.changeSet.id;
        createdChangeSetIds.push(changeSetId);

        console.log(`Created change set with ID: ${changeSetId}`);

        const findSchemaResponse = await api.schemas.findSchema(
          config.workspaceId,
          { name: "AWS::Neptune::DBCluster" },
          changeSetId,
        );

        assertEquals(
          findSchemaResponse.status,
          200,
          "Find schema response status should be 200",
        );
        assertEquals(
          findSchemaResponse.data.schemaName,
          "AWS::Neptune::DBCluster",
        );
        assertEquals(findSchemaResponse.data.category, "AWS::Neptune");
        assertEquals(findSchemaResponse.data.installed, false);

        const getSchemaResponse = await api.schemas.getSchema(
          config.workspaceId,
          changeSetId,
          findSchemaResponse.data.schemaId,
        );
        assertEquals(
          getSchemaResponse.status,
          200,
          "Get schema response status should be 200",
        );
        assertEquals(
          getSchemaResponse.data.name,
          "AWS::Neptune::DBCluster",
          "Ensure we have a schema name that matches",
        );

        const getDefaultSchemaResponse =
          await api.schemas.getDefaultSchemaVariant(
            config.workspaceId,
            changeSetId,
            findSchemaResponse.data.schemaId,
          );
        assertEquals(
          getDefaultSchemaResponse.status,
          200,
          "Get schema response status should be 200",
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
  },
);
