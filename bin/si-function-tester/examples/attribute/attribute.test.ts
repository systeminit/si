import { defineTests, expect, mockExec } from "file:///app/index.ts";

export default defineTests({
  "returns most recent AMI matching filters": {
    input: {
      region: "us-east-1",
      Filters: [
        { Name: "name", Value: "ubuntu-*" },
        { Name: "architecture", Value: "x86_64" },
      ],
      Owners: "amazon",
    },
    mocks: {
      exec: mockExec()
        .command("aws ec2 describe-images")
        .returns({
          stdout: JSON.stringify([
            { ImageId: "ami-newest123" },
            { ImageId: "ami-older456" },
          ]),
          exitCode: 0,
        }),
    },
    expect: {
      validate: (result) => {
        if (result !== "ami-newest123") {
          throw new Error(`Expected ami-newest123, got ${result}`);
        }
      },
    },
  },

  "returns empty string when no filters specified": {
    input: {
      region: "us-east-1",
    },
    expect: {
      validate: (result) => {
        if (result !== "") {
          throw new Error(`Expected empty string, got ${result}`);
        }
      },
    },
  },
});
