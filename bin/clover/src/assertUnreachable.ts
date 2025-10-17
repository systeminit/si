import util from "node:util";
export function assertUnreachable(value: never): never {
  throw new Error(`Didn't expect to get here: ${util.inspect(value)}`);
}
