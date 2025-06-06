# `lang-js`

This directory contains `lang-js`, the primary executor for functions in SI.

## Local Testing Guide

This is a guide for testing `lang-js` locally by using functions in the
[examples](examples) directory.

> [!NOTE]
> While [dal integration tests](../../lib/dal/tests/integration.rs) are useful
> for testing new functions and workflows that leverage `lang-js`, it can be
> helpful to run `lang-js` directly for an efficient developer feedback loop.

### 1) Writing the Function

Before writing a payload file, we will want to write a function to be executed.
We can do this anywhere, but for this guide, we will write the file to the
[examples](examples) directory.

> [!TIP]
> You can write the function in JavaScript (`.js`) or TypeScript (`.ts`). You
> can also write an `async` function or a regular, synchronous one.

Here is an example function:

```js
function fail() {
  throw new Error("wheeeeeeeeeeeeeeee");
}
```

### 2) Encoding the Function

With our new function written, we need to "base64 encode" it for the payload
file. You can do that by executing the following:

```bash
cat examples/<code-file>.<js-or-ts> | base64 | tr -d '\n'
```

You'll want to copy the result to your clipboard. On macOS, you can execute the
following to do it in one step:

```bash
cat examples/<code-file>.<js-or-ts> | base64 | tr -d '\n' | pbcopy
```

### 3) Preparing the Payload File

With the encoded function in our clipboard, we can create a payload file to send
to `lang-js`. The payload file will be a `json` file in the same directory.

At the time of writing the guide
([PR #4259](https://github.com/systeminit/si/pull/4259)), the shape the the
`json` file has drifted from what it used to be, so developers will need to read
the source code to learn its shape.

### 4) Running the Function via the Payload File

When we run our function in `lang-js`, let's set the debug flag to see what's
going on!

```bash
cat examples/<payload-file>.json | SI_LANG_JS_LOG=* buck2 run :lang-js -- <function-kind>
```
