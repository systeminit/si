import { defineTests, expect, mockExec } from "file:///app/index.ts";

export default defineTests({
  "succeeds when query returns correct ImageId": {
    input: {
      domain: {
        region: "us-east-1",
        ImageId: "ami-123456",
        Filters: [{ Name: "name", Value: "ubuntu-*" }],
        Owners: "amazon",
        UseMostRecent: true,
      },
    },
    mocks: {
      exec: mockExec()
        .command("aws ec2 describe-images")
        .returns({
          stdout: JSON.stringify([
            { ImageId: "ami-123456" },
          ]),
          exitCode: 0,
        }),
    },
    expect: {
      validate: (result) => {
        const qual = result as { result: string; message: string };
        if (qual.result !== "success") {
          throw new Error(
            `Expected success, got ${qual.result}: ${qual.message}`,
          );
        }
        if (!qual.message.includes("correct image")) {
          throw new Error(`Unexpected message: ${qual.message}`);
        }
      },
    },
  },

  "fails when multiple images match and UseMostRecent is false": {
    input: {
      domain: {
        region: "us-east-1",
        ImageId: "ami-123456",
        Filters: [{ Name: "name", Value: "ubuntu-*" }],
        UseMostRecent: false,
      },
    },
    mocks: {
      exec: mockExec()
        .command("aws ec2 describe-images")
        .returns({
          stdout: JSON.stringify([
            { ImageId: "ami-123456" },
            { ImageId: "ami-789012" },
          ]),
          exitCode: 0,
        }),
    },
    expect: {
      validate: (result) => {
        const qual = result as { result: string; message: string };
        if (qual.result !== "failure") {
          throw new Error(`Expected failure, got ${qual.result}`);
        }
        if (!qual.message.includes("Multiple images")) {
          throw new Error(
            `Expected multiple images error, got: ${qual.message}`,
          );
        }
      },
    },
  },

  "fails when region is not specified": {
    input: {
      domain: {
        ImageId: "ami-123456",
      },
    },
    expect: {
      validate: (result) => {
        const qual = result as { result: string; message: string };
        if (qual.result !== "failure") {
          throw new Error(`Expected failure, got ${qual.result}`);
        }
        if (!qual.message.includes("must specify a region")) {
          throw new Error(`Expected region error, got: ${qual.message}`);
        }
      },
    },
  },
});
