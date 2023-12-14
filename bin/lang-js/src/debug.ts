export type Debugger = (msg: any) => void;

export function Debug(namespace: string): Debugger {
  const debugActive = process.env.SI_LANG_JS_LOG || process.env.SI_LOG;

  return (msg: any) => {
    if (debugActive) {
      const pretty_json = JSON.stringify(msg, null, 2);
      for (const line of pretty_json.split("\n")) {
        process.stderr.write(`${namespace} ${line}\n`);
      }
    }
  };
}
