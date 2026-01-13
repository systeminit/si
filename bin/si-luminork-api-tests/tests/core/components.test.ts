/**
 * Components Tests
 *
 * Tests for the components API endpoints.
 */

import { assertEquals, assertExists } from 'https://deno.land/std@0.220.1/assert/mod.ts';
import {
  cleanupTestResources,
  ConfigError,
  createTestClient,
  generateTestName,
} from '../../src/test-utils.ts';
import { ApiError } from '../../src/client.ts';

Deno.test('Components API - Create and Update Components', async () => {
  try {
    const { api, config } = await createTestClient();

    const createdChangeSetIds: string[] = [];

    try {
      // First create a change set to work with
      const createChangeSetResponse = await api.changeSets.createChangeSet(
        config.workspaceId,
        {
          changeSetName: generateTestName('component_test_changeset'),
        },
      );

      assertEquals(createChangeSetResponse.status, 200);
      assertExists(createChangeSetResponse.data.changeSet);
      const changeSetId = createChangeSetResponse.data.changeSet.id;
      createdChangeSetIds.push(changeSetId);

      console.log(`Created change set with ID: ${changeSetId}`);

      // Find a schema to use for the component
      // Note: Assumes there's at least one schema in the workspace
      const schemasResponse = await api.schemas.listSchemas(config.workspaceId);
      assertEquals(schemasResponse.status, 200);

      // Use AWS::EC2::VPC as the schema name as requested
      const schemaName = 'AWS::EC2::VPC';
      console.log(`Using schema: ${schemaName}`);

      // Create a component with the correct payload format
      const componentName = generateTestName('test_component');
      const createComponentResponse = await api.components.createComponent(
        config.workspaceId,
        changeSetId,
        {
          name: componentName,
          schemaName: schemaName,
        },
      );

      assertEquals(createComponentResponse.status, 200);
      assertExists(createComponentResponse.data.component);
      assertExists(createComponentResponse.data.component.id);

      const componentId = createComponentResponse.data.component.id;
      console.log(`Created component with ID: ${componentId}`);

      // Get the component
      const getComponentResponse = await api.components.getComponent(
        config.workspaceId,
        changeSetId,
        componentId,
      );

      assertEquals(getComponentResponse.status, 200);
      assertExists(getComponentResponse.data.component);
      assertEquals(getComponentResponse.data.component.id, componentId);
      assertEquals(getComponentResponse.data.component.name, componentName);

      // Update the component
      const updatedName = `${componentName}_updated`;
      const updateComponentResponse = await api.components.updateComponent(
        config.workspaceId,
        changeSetId,
        componentId,
        {
          name: updatedName,
        },
      );

      assertEquals(updateComponentResponse.status, 200);
      assertExists(updateComponentResponse.data.component);
      assertEquals(updateComponentResponse.data.component.name, updatedName);

      // List components
      const listComponentsResponse = await api.components.listComponents(
        config.workspaceId,
        changeSetId,
      );

      assertEquals(listComponentsResponse.status, 200);
      assertExists(listComponentsResponse.data.componentDetails);

      // Verify our component is in the list
      const isComponentInList = listComponentsResponse.data.componentDetails.some((c) => {
        return c.componentId === componentId;
      });
      assertEquals(
        isComponentInList,
        true,
        `Component ${componentId} not found in the list`,
      );

      // Find component by name test
      console.log(`Testing find component by name: ${updatedName}`);

      // The API is expecting 'component' as the query parameter name instead of 'name'
      // This is handled in our findComponent method
      const findComponentResponse = await api.components.findComponent(
        config.workspaceId,
        changeSetId,
        {
          name: updatedName, // This gets mapped to 'component' query param in the findComponent method
        },
      );
      assertEquals(
        findComponentResponse.status,
        200,
        'Find component response status should be 200',
      );
      assertEquals(
        findComponentResponse.data.component.name,
        updatedName,
        'Find component names should be equal',
      );

      // Delete the component
      const deleteComponentResponse = await api.components.deleteComponent(
        config.workspaceId,
        changeSetId,
        componentId,
      );

      assertEquals(deleteComponentResponse.status, 200);

      // Verify component was deleted
      try {
        // Try to get the component after deletion
        const response = await api.components.getComponent(
          config.workspaceId,
          changeSetId,
          componentId,
        );

        // If we get here, the delete didn't work - we shouldn't be able to get the component
        console.error('Component still exists after deletion:', response);
        throw new Error(
          'Component should have been deleted - was still able to retrieve it',
        );
      } catch (error) {
        // Check if this is an ApiError with 404 status (expected case)
        if (error instanceof ApiError && error.status === 404) {
          console.log(
            `Verified component was deleted successfully - got expected 404 Not Found response`,
          );
        } // The API might return error differently than expected - check error message for 404 references
        else if (
          error instanceof Error &&
          (error.message.includes('404') || error.message.includes('Not Found'))
        ) {
          console.log(
            'Successfully deleted component - error message contains 404 Not Found reference',
          );
        } // For any other errors, re-throw it with a descriptive message
        else {
          console.error("Error wasn't a 404 Not Found response:", error);
          throw error;
        }
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
