/**
 * Tests for AWS CloudControl Create Function
 *
 * This demonstrates how to test a complex function with:
 * - Multiple AWS CLI calls (create-resource + get-resource-request-status)
 * - Retry logic with exponential backoff
 * - Rate limiting handling
 * - Status polling
 */

import { defineTests, expect, mockExec } from "file:///app/index.ts";

export default defineTests({
  // ========================================
  // SUCCESSFUL CREATE
  // ========================================

  "creates resource successfully on first attempt": {
    input: {
      properties: {
        code: {
          awsCloudControlCreate: {
            code: JSON.stringify({
              TypeName: "AWS::S3::Bucket",
              DesiredState: {
                BucketName: "test-bucket",
              },
            }),
          },
        },
        domain: {
          extra: {
            Region: "us-east-1",
            PropUsageMap: JSON.stringify({ secrets: [] }),
          },
        },
        resource: {
          // No payload means resource doesn't exist yet
        },
      },
    },
    mocks: {
      exec: mockExec()
        // First call: create-resource
        .command("aws cloudcontrol create-resource")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              RequestToken: "mock-token-123",
              OperationStatus: "IN_PROGRESS",
            },
          }),
          exitCode: 0,
        })
        // Second call: get-resource-request-status (success)
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              OperationStatus: "SUCCESS",
              Identifier: "test-bucket",
              RequestToken: "mock-token-123",
            },
          }),
          exitCode: 0,
        }),
    },
    expect: {
      status: "ok",
      resourceId: "test-bucket",
    },
    timeout: 10000, // Allow time for polling
  },

  "handles resource that requires multiple status polls": {
    input: {
      properties: {
        code: {
          awsCloudControlCreate: {
            code: JSON.stringify({
              TypeName: "AWS::EC2::Instance",
              DesiredState: { InstanceType: "t2.micro" },
            }),
          },
        },
        domain: {
          extra: {
            Region: "us-west-2",
            PropUsageMap: JSON.stringify({ secrets: [] }),
          },
        },
        resource: {},
      },
    },
    mocks: {
      exec: mockExec()
        // Create returns immediately
        .command("aws cloudcontrol create-resource")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              RequestToken: "token-456",
              OperationStatus: "IN_PROGRESS",
            },
          }),
          exitCode: 0,
        })
        // First status poll: still in progress
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              OperationStatus: "IN_PROGRESS",
              RequestToken: "token-456",
            },
          }),
          exitCode: 0,
        })
        // Second status poll: success!
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              OperationStatus: "SUCCESS",
              Identifier: "i-1234567890abcdef0",
              RequestToken: "token-456",
            },
          }),
          exitCode: 0,
        }),
    },
    expect: {
      status: "ok",
      resourceId: "i-1234567890abcdef0",
    },
    timeout: 15000,
  },

  // ========================================
  // VALIDATION / EARLY EXIT
  // ========================================

  "returns error when resource already exists": {
    input: {
      properties: {
        resource: {
          payload: { existing: "resource" }, // Resource exists!
        },
        code: {
          awsCloudControlCreate: {
            code: JSON.stringify({
              TypeName: "AWS::S3::Bucket",
              DesiredState: {},
            }),
          },
        },
        domain: { extra: { Region: "us-east-1", PropUsageMap: "{}" } },
      },
    },
    // No mocks needed - fails validation before any AWS calls
    expect: {
      status: "error",
      message: "Resource already exists",
    },
  },

  "returns error when awsCloudControlCreate code is missing": {
    input: {
      properties: {
        code: {
          // awsCloudControlCreate is missing
        },
        domain: { extra: { Region: "us-east-1", PropUsageMap: "{}" } },
        resource: {},
      },
    },
    expect: {
      status: "error",
      message: expect.stringContaining("Could not find awsCloudControlCreate"),
    },
  },

  // ========================================
  // RATE LIMITING / RETRY LOGIC
  // ========================================

  "retries create-resource when rate limited": {
    input: {
      properties: {
        code: {
          awsCloudControlCreate: {
            code: JSON.stringify({
              TypeName: "AWS::Lambda::Function",
              DesiredState: { FunctionName: "test" },
            }),
          },
        },
        domain: {
          extra: {
            Region: "us-east-1",
            PropUsageMap: JSON.stringify({ secrets: [] }),
          },
        },
        resource: {},
      },
    },
    mocks: {
      exec: mockExec()
        // First attempt: rate limited
        .command("aws cloudcontrol create-resource")
        .returns({
          stdout: "",
          stderr:
            "An error occurred (ThrottlingException) when calling the CreateResource operation: Rate exceeded",
          exitCode: 254,
        })
        // Second attempt: success!
        .command("aws cloudcontrol create-resource")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              RequestToken: "retry-token",
              OperationStatus: "IN_PROGRESS",
            },
          }),
          exitCode: 0,
        })
        // Status check: success (provide multiple in case of retries)
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              OperationStatus: "SUCCESS",
              Identifier: "test-function",
              RequestToken: "retry-token",
            },
          }),
          exitCode: 0,
        })
        // Extra status checks in case function polls multiple times
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              OperationStatus: "SUCCESS",
              Identifier: "test-function",
              RequestToken: "retry-token",
            },
          }),
          exitCode: 0,
        })
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              OperationStatus: "SUCCESS",
              Identifier: "test-function",
              RequestToken: "retry-token",
            },
          }),
          exitCode: 0,
        }),
    },
    expect: {
      status: "ok",
      resourceId: "test-function",
    },
    timeout: 10000, // Should be fast - only one retry with ~1s delay
  },

  "retries status poll when rate limited": {
    input: {
      properties: {
        code: {
          awsCloudControlCreate: {
            code: JSON.stringify({
              TypeName: "AWS::DynamoDB::Table",
              DesiredState: { TableName: "test-table" },
            }),
          },
        },
        domain: {
          extra: {
            Region: "us-east-1",
            PropUsageMap: JSON.stringify({ secrets: [] }),
          },
        },
        resource: {},
      },
    },
    mocks: {
      exec: mockExec()
        // Create succeeds
        .command("aws cloudcontrol create-resource")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              RequestToken: "table-token",
              OperationStatus: "IN_PROGRESS",
            },
          }),
          exitCode: 0,
        })
        // First status poll: rate limited
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: "",
          stderr:
            "An error occurred (Throttling) when calling the GetResourceRequestStatus operation",
          exitCode: 254,
        })
        // Second status poll: success
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              OperationStatus: "SUCCESS",
              Identifier: "test-table",
              RequestToken: "table-token",
            },
          }),
          exitCode: 0,
        })
        // Extra status checks in case function polls multiple times
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              OperationStatus: "SUCCESS",
              Identifier: "test-table",
              RequestToken: "table-token",
            },
          }),
          exitCode: 0,
        })
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              OperationStatus: "SUCCESS",
              Identifier: "test-table",
              RequestToken: "table-token",
            },
          }),
          exitCode: 0,
        }),
    },
    expect: {
      status: "ok",
      resourceId: "test-table",
    },
    timeout: 10000, // Should be fast - only one retry with ~1s delay
  },

  // ========================================
  // AWS OPERATION FAILURES
  // ========================================

  "handles AWS operation failure": {
    input: {
      properties: {
        code: {
          awsCloudControlCreate: {
            code: JSON.stringify({
              TypeName: "AWS::S3::Bucket",
              DesiredState: { BucketName: "invalid--bucket" },
            }),
          },
        },
        domain: {
          extra: {
            Region: "us-east-1",
            PropUsageMap: JSON.stringify({ secrets: [] }),
          },
        },
        resource: {},
      },
    },
    mocks: {
      exec: mockExec()
        // Create starts
        .command("aws cloudcontrol create-resource")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              RequestToken: "fail-token",
              OperationStatus: "IN_PROGRESS",
            },
          }),
          exitCode: 0,
        })
        // Status poll returns FAILED
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              OperationStatus: "FAILED",
              StatusMessage: "Invalid bucket name format",
              ErrorCode: "InvalidRequest",
              RequestToken: "fail-token",
            },
          }),
          exitCode: 0,
        }),
    },
    expect: {
      status: "error",
      message: "Invalid bucket name format",
    },
    timeout: 10000,
  },

  "handles operation cancellation": {
    input: {
      properties: {
        code: {
          awsCloudControlCreate: {
            code: JSON.stringify({
              TypeName: "AWS::RDS::DBInstance",
              DesiredState: { DBInstanceIdentifier: "test-db" },
            }),
          },
        },
        domain: {
          extra: {
            Region: "us-east-1",
            PropUsageMap: JSON.stringify({ secrets: [] }),
          },
        },
        resource: {},
      },
    },
    mocks: {
      exec: mockExec()
        .command("aws cloudcontrol create-resource")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              RequestToken: "cancel-token",
              OperationStatus: "IN_PROGRESS",
            },
          }),
          exitCode: 0,
        })
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              OperationStatus: "CANCEL_COMPLETE",
              RequestToken: "cancel-token",
            },
          }),
          exitCode: 0,
        }),
    },
    expect: {
      status: "error",
      message: "Operation Canceled by API or AWS.",
    },
    timeout: 10000,
  },

  // ========================================
  // ERROR HANDLING
  // ========================================

  "handles non-rate-limit AWS CLI errors": {
    input: {
      properties: {
        code: {
          awsCloudControlCreate: {
            code: JSON.stringify({
              TypeName: "AWS::S3::Bucket",
              DesiredState: {},
            }),
          },
        },
        domain: {
          extra: {
            Region: "invalid-region",
            PropUsageMap: JSON.stringify({ secrets: [] }),
          },
        },
        resource: {},
      },
    },
    mocks: {
      exec: mockExec()
        .command("aws cloudcontrol create-resource")
        .returns({
          stdout: "",
          stderr: "Could not connect to the endpoint URL",
          exitCode: 1,
        }),
    },
    expect: {
      status: "error",
      message: expect.stringContaining("AWS CLI 2 exited with non zero code"),
    },
  },

  "handles malformed JSON in AWS response": {
    input: {
      properties: {
        code: {
          awsCloudControlCreate: {
            code: JSON.stringify({
              TypeName: "AWS::S3::Bucket",
              DesiredState: {},
            }),
          },
        },
        domain: {
          extra: {
            Region: "us-east-1",
            PropUsageMap: JSON.stringify({ secrets: [] }),
          },
        },
        resource: {},
      },
    },
    mocks: {
      exec: mockExec()
        .command("aws cloudcontrol create-resource")
        .returns({
          stdout: JSON.stringify({
            ProgressEvent: {
              RequestToken: "bad-json-token",
              OperationStatus: "IN_PROGRESS",
            },
          }),
          exitCode: 0,
        })
        .command("aws cloudcontrol get-resource-request-status")
        .returns({
          stdout: "{ invalid json !!!", // Malformed JSON
          exitCode: 0,
        }),
    },
    expect: {
      status: "error",
      message: expect.stringContaining(
        "Unable to parse AWS CloudControl response",
      ),
    },
    timeout: 10000,
  },
});
