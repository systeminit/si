/**
 * Change Sets Tests
 *
 * Tests for the change sets API endpoints.
 */

import { assertEquals, assertExists } from 'https://deno.land/std@0.220.1/assert/mod.ts';
import {
  cleanupTestResources,
  ConfigError,
  createTestClient,
  generateTestName,
} from '../src/test-utils.ts';
import { ApiError } from '../src/client.ts';

Deno.test('Change Sets API - CRUD Operations', async () => {
  try {
    const { api, config } = await createTestClient();

    const createdChangeSetIds: string[] = [];

    try {
      // Create a change set
      const createResponse = await api.changeSets.createChangeSet(config.workspaceId, {
        changeSetName: generateTestName('test_change_set'),
      });

      assertEquals(createResponse.status, 200);
      assertExists(createResponse.data.changeSet);
      assertExists(createResponse.data.changeSet.id);

      const changeSetId = createResponse.data.changeSet.id;
      createdChangeSetIds.push(changeSetId);

      console.log(`Created change set with ID: ${changeSetId}`);

      // Get the change set
      const getResponse = await api.changeSets.getChangeSet(config.workspaceId, changeSetId);

      assertEquals(getResponse.status, 200);
      assertExists(getResponse.data.changeSet);
      assertEquals(getResponse.data.changeSet.id, changeSetId);

      // List change sets
      const listResponse = await api.changeSets.listChangeSets(config.workspaceId);

      assertEquals(listResponse.status, 200);
      assertExists(listResponse.data.changeSets);

      // Verify our change set is in the list
      const foundChangeSet = listResponse.data.changeSets.find((cs) => cs.id === changeSetId);
      assertExists(foundChangeSet);

      console.log(`Verified change set exists in list`);

      // Delete the change set
      const deleteResponse = await api.changeSets.deleteChangeSet(config.workspaceId, changeSetId);
      assertEquals(deleteResponse.status, 200);

      // Ensure the change set is abandoned
      const deletedChangeSetResponse = await api.changeSets.getChangeSet(
        config.workspaceId,
        changeSetId,
      );
      assertEquals(deletedChangeSetResponse.data.changeSet.status, 'Abandoned');
    } finally {
      // Clean up any change sets that weren't deleted by the test
      await cleanupTestResources(api, config.workspaceId, createdChangeSetIds);
    }
  } catch (error) {
    if (error instanceof ConfigError) {
      console.warn(`Skipping test due to configuration error: ${error.message}`);
      return;
    }
    throw error;
  }
});

Deno.test('Change Sets API - Create Multiple and Purge', async () => {
  try {
    const { api, config } = await createTestClient();

    const createdChangeSetIds: string[] = [];

    try {
      // Create multiple change sets
      for (let i = 0; i < 3; i++) {
        const createResponse = await api.changeSets.createChangeSet(config.workspaceId, {
          changeSetName: generateTestName(`purge_test_${i}`),
        });

        assertEquals(createResponse.status, 200);
        assertExists(createResponse.data.changeSet);
        assertExists(createResponse.data.changeSet.id);

        createdChangeSetIds.push(createResponse.data.changeSet.id);
      }

      // Verify they were created
      const listResponse = await api.changeSets.listChangeSets(config.workspaceId);
      assertEquals(listResponse.status, 200);
      assertExists(listResponse.data.changeSets);

      let foundCount = 0;
      for (const id of createdChangeSetIds) {
        if (listResponse.data.changeSets.some((cs) => cs.id === id)) {
          foundCount++;
        }
      }

      assertEquals(foundCount, createdChangeSetIds.length);

      // Purge all open change sets
      // Note: This might affect other change sets in the workspace, so be careful in shared environments
      const purgeResponse = await api.changeSets.purgeOpenChangeSets(config.workspaceId);
      assertEquals(purgeResponse.status, 200);

      // Verify they're gone
      const listAfterPurge = await api.changeSets.listChangeSets(config.workspaceId);
      assertExists(listAfterPurge.data.changeSets);

      foundCount = 0;
      for (const id of createdChangeSetIds) {
        if (listAfterPurge.data.changeSets.some((cs) => cs.id === id)) {
          foundCount++;
        }
      }

      assertEquals(foundCount, 0, 'All test change sets should be purged');
    } catch (error: unknown) {
      if (error instanceof Error) {
        console.error('Test failed:', error.message);
      } else {
        console.error('Test failed with unknown error');
      }
      throw error;
    } finally {
      // Attempt to clean up any change sets that weren't purged
      await cleanupTestResources(api, config.workspaceId, createdChangeSetIds);
    }
  } catch (error: unknown) {
    if (error instanceof ConfigError) {
      console.warn(`Skipping test due to configuration error: ${error.message}`);
      return;
    }
    throw error;
  }
});
