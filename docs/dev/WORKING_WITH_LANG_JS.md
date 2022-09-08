# Working with `lang-js`

This document contains information related to working with [`lang-js`](../../bin/lang-js).
You should check out the [`lang-js` README](../../bin/lang-js/README.md) as well.

## Quickstart

While [dal integration tests](../../lib/dal/tests/integration.rs) are useful for testing new functions and workflows
that leverage [`lang-js`](../../bin/lang-js), it can be helpful to run `lang-js` directly for an efficient
developer feedback loop.

In the [`lang-js` directory](../../bin/lang-js), let's look at an example.
First, let's author a function and save it to the [examples directory](../../bin/lang-js/examples) directory.

```js
function fail() {
    throw new Error("wheeeeeeeeeeeeeeee");
}
```

Now, let's base64 encode this function and save the result to our clipboard.

```bash
cat examples/commandRunFailCode.js | base64 | tr -d '\n'
```

Then, we can create a `json` file in the same directory that's in a format that `lang-js` expects.

```json
{
  "executionId": "fail",
  "handler": "fail",
  "codeBase64": "ZnVuY3Rpb24gZmFpbCgpIHsKICAgIHRocm93IG5ldyBFcnJvcigid2hlZWVlZWVlZWVlZWVlZWVlIik7Cn0K"
}
```

Finally, we can run our function in `lang-js` directly.

> Ensure that `lang-js` has been built by running the following `make` target in the repository root:
>
> ```bash
> make build//bin/lang-js
> ```

When we run our function in `lang-js`, let's set the debug flag to see what's going on!

```bash
cat examples/commandRunFail.json | DEBUG=* target/lang-js commandRun
```