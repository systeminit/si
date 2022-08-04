// Encode this into Base64 with:
//
// ```sh
// cat examples/resolverFunctionTestCode.js | base64 | tr -d '\n'
// ```

// The request.handle maps to the name of the function to invoke--we can ever
// define a few and neglect to call them--or use them--it's just javascript,
// right?
function calcBigObject(component) {
  // we can still console.log which emits OutputLines
  console.log("you love me");
  console.log("I know you do");

  // Parameters can now be used and defaults to {}
  const total =
    component.data.parameters["left"] + component.data.parameters["right"];

  const f = {
    poop: {
      canoe: {
        who: "fletcher",
      },
      pair: true,
      mapperton: {
        slow: "moving increments",
        pressure: "is crushing me",
      },
      arraymonster: ["foo", "bar", "baz"],
    },
    name: component.data.name,
    parents: component.parents.length,
    total,
  };

  // The return of the function is mapped to result.data
  return f;
}

// The name of the first argument doesn't matter, but it's populated by the
// lang server
function setString(component) {
  return component.value;
}

function stringJoin(component) {
  // More complicated code can be used
  const items = component.data.parameters.items;
  // Including maybe optional or important parameters? If you need to bail,
  // then throw an exception--the lang server interprets this as an execution
  // failure and reports the error upstack
  if (!items) {
    throw Error(`missing: component.data.parameters["value"]`);
  }
  // Console debugging also supported--it sets the level to debug and sets the
  // output stream to stderr
  console.debug("going to join", { items });

  // How about a little lodash?
  return _.join(items, ", ");
}
