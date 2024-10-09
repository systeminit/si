export function commandWithPager(cmd: string, args: string[]): string {
  const command = new Deno.Command(cmd, { args, env: { "PAGER": "cat" }, stdin: "null" });
  const { stdout } = command.outputSync();
  return new TextDecoder().decode(stdout);
}
