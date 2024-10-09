export function readWebPage(url: string): string {
  const command = new Deno.Command("w3m", { args: ["-dump", url], stdin: "null" });
  const result = command.outputSync();
  if (!result.success) {
    console.error("Failed to load web page");
    console.error(new TextDecoder().decode(result.stderr));
    Deno.exit(5);
  }
  return new TextDecoder().decode(result.stdout);
}
