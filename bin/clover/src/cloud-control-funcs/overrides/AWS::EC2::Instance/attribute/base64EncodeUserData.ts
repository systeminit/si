async function main(input: Input): Promise<Output> {
  const data = input.data;

  if (!data) return "";

  try {
    const decoded = Buffer.from(data, 'base64').toString('utf8');
    const reEncoded = Buffer.from(decoded, 'utf8').toString('base64');

    if (reEncoded === data) {
      return data;
    }
  } catch {}

  return Buffer.from(data, 'utf8').toString('base64');
}
