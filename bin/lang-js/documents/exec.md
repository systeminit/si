# Executing Commands

This documents the `siExec` command.

## Arguments

1. The command to run
2. An array of arguments

## Examples

```ts
const child = await siExec.waitUntilEnd("aws", ["ec2", "describe-instances"]);
if (child.exitCode !== 0) {
  console.log(`Failed to run command, exit code: ${child.exitCode}`);
  console.log(child.stdout);
  console.error(child.stderr);
}
```
