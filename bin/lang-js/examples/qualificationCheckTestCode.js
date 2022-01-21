// Encode this into Base64 with:
//
// ```sh
// cat examples/qualificationCheckTestCode.js | base64 | tr -d '\n'
// ```

function nameIsGood(component) {
  // For our purposes, a "good" name is one that is all lowercase
  if (component["name"].toLowerCase() == component["name"]) {
    return { qualified: true };
  } else {
    return {
      qualified: false,
      message: "name must be all lowercase",
    };
  }
}
