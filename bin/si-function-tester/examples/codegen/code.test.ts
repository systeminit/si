import { defineTests } from "file:///app/index.ts";

export default defineTests({
  "generates CloudControl JSON payload": {
    input: {
      domain: {
        extra: {
          AwsResourceType: "AWS::EC2::VPC",
          PropUsageMap: JSON.stringify({
            createOnly: ["CidrBlock"],
            updatable: ["Tags"],
            secrets: [],
          }),
        },
        CidrBlock: "10.0.0.0/16",
        Tags: [{ Key: "Name", Value: "test-vpc" }],
        UnusedProp: "should-be-removed",
      },
    },
    expect: {
      validate: (result) => {
        const codegen = result as { format: string; code: string };

        if (codegen.format !== "json") {
          throw new Error(`Expected format json, got ${codegen.format}`);
        }

        const payload = JSON.parse(codegen.code);

        if (payload.TypeName !== "AWS::EC2::VPC") {
          throw new Error(
            `Expected TypeName AWS::EC2::VPC, got ${payload.TypeName}`,
          );
        }

        if (!payload.DesiredState) {
          throw new Error("Missing DesiredState in payload");
        }

        if (payload.DesiredState.CidrBlock !== "10.0.0.0/16") {
          throw new Error(
            `Expected CidrBlock 10.0.0.0/16, got ${payload.DesiredState.CidrBlock}`,
          );
        }

        if (!Array.isArray(payload.DesiredState.Tags)) {
          throw new Error("Expected Tags array in DesiredState");
        }

        // UnusedProp should be filtered out
        if (payload.DesiredState.UnusedProp) {
          throw new Error(
            "UnusedProp should have been filtered out but is present",
          );
        }
      },
    },
  },

  "includes secrets from requestStorage": {
    input: {
      domain: {
        extra: {
          AwsResourceType: "AWS::EC2::Subnet",
          PropUsageMap: JSON.stringify({
            createOnly: ["VpcId", "CidrBlock"],
            updatable: [],
            secrets: [
              {
                secretKey: "vpcId",
                propPath: ["VpcId"],
              },
            ],
          }),
        },
        CidrBlock: "10.0.1.0/24",
      },
    },
    mocks: {
      storage: {
        vpcId: "vpc-secret-123",
      },
    },
    expect: {
      validate: (result) => {
        const codegen = result as { format: string; code: string };
        const payload = JSON.parse(codegen.code);

        if (payload.DesiredState.VpcId !== "vpc-secret-123") {
          throw new Error(
            `Expected VpcId from secret, got ${payload.DesiredState.VpcId}`,
          );
        }
      },
    },
  },

  "throws error for malformed PropUsageMap": {
    input: {
      domain: {
        extra: {
          AwsResourceType: "AWS::EC2::VPC",
          PropUsageMap: JSON.stringify({
            createOnly: "not-an-array",
            updatable: [],
            secrets: [],
          }),
        },
        CidrBlock: "10.0.0.0/16",
      },
    },
    expect: {
      validate: (result) => {
        // The function should throw an error, which gets caught by runner
        // and returned as an error result
        const errorResult = result as any;

        // Check if there's an error message about malformed propUsageMap
        const hasError = errorResult.status === "error" ||
          (errorResult.message &&
            errorResult.message.includes("malformed propUsageMap"));

        if (!hasError) {
          throw new Error(
            "Expected error for malformed PropUsageMap, but got success",
          );
        }
      },
    },
  },
});
