export async function sleep(ms: number) {
  const positive_ms = Math.max(0, Math.floor(ms));
  return new Promise((resolve) => {
    setTimeout(resolve, positive_ms);
  });
}
