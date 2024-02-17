export const partial = `
async function main(input: Input): Promise < Output > {
    if (input.domain?.extra) {
        delete input.domain.extra;
    }
  return {
    format: "json",
    code: JSON.stringify(input.domain || {}, null, 2),
  }
}
`
