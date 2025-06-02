/**
 * System Status Tests
 * 
 * Tests for the system status endpoint to verify API availability.
 */

import { assertEquals } from 'https://deno.land/std@0.220.1/assert/mod.ts';
import { createTestClient, ConfigError } from '../src/test-utils.ts';

Deno.test("API Server System Status", async () => {
  try {
    const { api } = await createTestClient();
    
    // Check system status
    const response = await api.getSystemStatus();
    
    // Verify response
    assertEquals(response.status, 200);
    
    // Check that the response contains expected fields
    const data = response.data;
    assertEquals(typeof data["API Documentation"], 'string');
    
    console.log("System status test passed!");
  } catch (error: unknown) {
    if (error instanceof ConfigError) {
      console.warn(`Skipping test due to configuration error: ${error.message}`);
      return;
    }
    throw error;
  }
});

Deno.test("Authentication - Whoami", async () => {
  try {
    const { api } = await createTestClient();
    
    // Test the whoami endpoint to verify authentication
    const response = await api.whoami();
    
    // Verify response
    assertEquals(response.status, 200);
    
    // Check that the response contains expected fields
    assertEquals(typeof response.data.userId, 'string');
    assertEquals(typeof response.data.userEmail, 'string');
    assertEquals(typeof response.data.workspaceId, 'string');
    assertEquals(typeof response.data.token, 'object');
    assertEquals(typeof response.data.token.userId, 'string');
    
    console.log("Whoami test passed!");
  } catch (error: unknown) {
    if (error instanceof ConfigError) {
      console.warn(`Skipping test due to configuration error: ${error.message}`);
      return;
    }
    throw error;
  }
});