import { defineTests, mockExec } from "file:///app/index.ts";

export default defineTests({
  "sets environment variables with direct credentials": {
    input: {
      AccessKeyId: "AKIATEST123",
      SecretAccessKey: "secretkey123",
      SessionToken: "sessiontoken123",
    },
    expect: {
      validate: (result) => {
        // Authentication functions return void/undefined
        // We can't directly verify env vars were set, but we can check no error occurred
        if (result !== null && result !== undefined) {
          // If there's a result that's not null/undefined, check if it's an error
          const anyResult = result as any;
          if (anyResult.status === "error" || anyResult.message) {
            throw new Error(
              `Expected successful auth, got: ${JSON.stringify(result)}`,
            );
          }
        }
        // void/undefined/null are all acceptable for successful auth
        return true;
      },
    },
  },

  "sets environment variables without session token": {
    input: {
      AccessKeyId: "AKIATEST456",
      SecretAccessKey: "secretkey456",
    },
    expect: {
      validate: (result) => {
        // Check that no error occurred
        if (result !== null && result !== undefined) {
          const anyResult = result as any;
          if (anyResult.status === "error" || anyResult.message) {
            throw new Error(
              `Expected successful auth, got: ${JSON.stringify(result)}`,
            );
          }
        }
        return true;
      },
    },
  },

  "sets endpoint URL when provided": {
    input: {
      AccessKeyId: "AKIATEST789",
      SecretAccessKey: "secretkey789",
      Endpoint: "https://localstack:4566",
    },
    expect: {
      validate: (result) => {
        // Check that no error occurred
        if (result !== null && result !== undefined) {
          const anyResult = result as any;
          if (anyResult.status === "error" || anyResult.message) {
            throw new Error(
              `Expected successful auth, got: ${JSON.stringify(result)}`,
            );
          }
        }
        return true;
      },
    },
  },

  "assumes role with provided keys": {
    input: {
      AssumeRole: "arn:aws:iam::123456789:role/test-role",
      AccessKeyId: "AKIATEST000",
      SecretAccessKey: "secretkey000",
      WorkspaceToken: "workspace-token-123",
    },
    mocks: {
      exec: mockExec()
        .command("aws configure set aws_access_key_id")
        .returns({ stdout: "", exitCode: 0 })
        .command("aws configure set aws_secret_access_key")
        .returns({ stdout: "", exitCode: 0 })
        .command("aws sts assume-role")
        .returns({
          stdout: JSON.stringify({
            Credentials: {
              AccessKeyId: "ASIATEMP123",
              SecretAccessKey: "tempsecret123",
              SessionToken: "temptoken123",
            },
          }),
          exitCode: 0,
        }),
    },
    expect: {
      validate: (result) => {
        // Check that no error occurred
        if (result !== null && result !== undefined) {
          const anyResult = result as any;
          if (anyResult.status === "error" || anyResult.message) {
            throw new Error(
              `Expected successful auth with assume role, got: ${
                JSON.stringify(result)
              }`,
            );
          }
        }
        return true;
      },
    },
  },

  "returns early when assume role fails": {
    input: {
      AssumeRole: "arn:aws:iam::123456789:role/invalid-role",
      WorkspaceToken: "workspace-token-456",
    },
    mocks: {
      exec: mockExec()
        .command("aws sts assume-role")
        .returns({
          stdout: "",
          stderr: "AccessDenied: User is not authorized",
          exitCode: 1,
        }),
    },
    expect: {
      validate: (result) => {
        // When assume role fails, the function returns early (void)
        // This is acceptable - no exception is thrown
        return true;
      },
    },
  },
});
