// Encode this into Base64 with:
//
// ```sh
// cat examples/resourceSyncTestCode.js | base64 | tr -d '\n'
// ```

function sync(component) {
  console.log(JSON.stringify(component));
  console.log("syncing something, right?");
  return {};
}
