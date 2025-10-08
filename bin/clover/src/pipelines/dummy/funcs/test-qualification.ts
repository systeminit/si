/**
 * Dummy test qualification function for exercising the generic override system
 */

export default function testQualification(domain: any) {
  // Always pass qualification for dummy provider
  return {
    result: "success",
    message: "Dummy qualification passed",
    details: {
      hasRequiredField: domain?.name !== undefined,
      timestamp: new Date().toISOString(),
    },
  };
}