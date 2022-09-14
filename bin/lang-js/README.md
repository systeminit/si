# Lang JS

This directory contains `lang-js`.

## Testing Locally

Here is an example of testing `lang-js` locally:

```bash
DEBUG=* npm run dev -- qualificationcheck < examples/qualificationCheckTest.json
```

## Encoding the Code

Here is an example of "encoding the code" locally:

```bash
cat examples/commandRunFailCode.js | base64 | tr -d '\n'
```

## More Information

Check out [WORKING_WITH_LANG_JS](../../docs/dev/WORKING_WITH_LANG_JS.md) for more information.