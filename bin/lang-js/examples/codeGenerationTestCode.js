// Encode this into Base64 with:
//
// ```sh
// cat examples/codeGenerationTestCode.js | base64 | tr -d '\n'
// ```

function generate(component) {
  console.log(JSON.stringify(component));
  console.log("generating something, right?");
  return { format: "json", code: JSON.stringify(component) };
}
