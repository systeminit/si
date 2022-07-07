import { Context } from "../context";

export async function greet(ctx: Context, name: string): Promise<string> {
  return `Hello, ${name}!`;
}
