/**
 * Dummy test action function for exercising the generic override system
 */

export default async function testAction() {
  return {
    status: "success",
    message: "Test action executed successfully",
    timestamp: new Date().toISOString(),
  };
}