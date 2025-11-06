import { defineTests, mockExec } from "file:///app/index.ts";

export default defineTests({
  "successfully imports resource": {
    input: {
      thisComponent: {
        properties: {
          si: {
            resourceId: "vpc-12345",
          },
          domain: {
            extra: {
              Region: "us-east-1",
              AwsResourceType: "AWS::EC2::VPC",
            },
          },
        },
      },
    },
    mocks: {
      exec: mockExec()
        .command("aws cloudcontrol get-resource")
        .returns({
          stdout: JSON.stringify({
            ResourceDescription: {
              Properties: JSON.stringify({
                VpcId: "vpc-12345",
                CidrBlock: "10.0.0.0/16",
              }),
            },
          }),
          exitCode: 0,
        }),
    },
    expect: {
      validate: (result) => {
        const mgmt = result as {
          status: string;
          message: string;
          ops?: any;
        };

        if (mgmt.status !== "ok") {
          throw new Error(`Expected status ok, got ${mgmt.status}`);
        }

        if (!mgmt.message.includes("Imported Resource")) {
          throw new Error(`Unexpected message: ${mgmt.message}`);
        }

        if (!mgmt.ops?.update?.self?.properties) {
          throw new Error("Missing ops.update.self.properties");
        }

        if (!mgmt.ops.actions?.self?.remove?.includes("create")) {
          throw new Error("Expected 'create' in actions.remove");
        }
      },
    },
  },

  "returns error when resourceId is missing": {
    input: {
      thisComponent: {
        properties: {
          domain: {
            extra: {
              Region: "us-east-1",
              AwsResourceType: "AWS::EC2::VPC",
            },
          },
        },
      },
    },
    expect: {
      validate: (result) => {
        const mgmt = result as { status: string; message: string };

        if (mgmt.status !== "error") {
          throw new Error(`Expected status error, got ${mgmt.status}`);
        }

        if (!mgmt.message.includes("No resourceId")) {
          throw new Error(`Expected resourceId error, got: ${mgmt.message}`);
        }
      },
    },
  },

  "returns error when AWS command fails": {
    input: {
      thisComponent: {
        properties: {
          si: {
            resourceId: "vpc-invalid",
          },
          domain: {
            extra: {
              Region: "us-east-1",
              AwsResourceType: "AWS::EC2::VPC",
            },
          },
        },
      },
    },
    mocks: {
      exec: mockExec()
        .command("aws cloudcontrol get-resource")
        .returns({
          stdout: "",
          stderr: "Resource not found",
          exitCode: 1,
        }),
    },
    expect: {
      validate: (result) => {
        const mgmt = result as { status: string; message: string };

        if (mgmt.status !== "error") {
          throw new Error(`Expected status error, got ${mgmt.status}`);
        }

        if (!mgmt.message.includes("Import error")) {
          throw new Error(`Expected import error, got: ${mgmt.message}`);
        }
      },
    },
  },
});
