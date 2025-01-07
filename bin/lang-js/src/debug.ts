import process from "node:process";

export type Debugger = (msg: unknown) => void;

export function Debug(namespace: string): Debugger {
  const debugActive = process.env.SI_LANG_JS_LOG || process.env.SI_LOG;
  return (msg: unknown) => {
    if (debugActive) {
      try {
        const safeStringify = (obj: unknown): string => {
          const seen = new WeakSet();
          return JSON.stringify(obj, (_, value) => {
            if (typeof value === "function") {
              return value.toString();
            }

            if (typeof value === "object" && value !== null) {
              if (seen.has(value)) {
                return "[Circular]";
              }
              seen.add(value);
            }

            if (value instanceof Error) {
              return {
                name: value.name,
                message: value.message,
                stack: value.stack,
              };
            }

            return value;
          }, 2);
        };

        const pretty_json = safeStringify(msg);
        for (const line of pretty_json.split("\n")) {
          process.stderr.write(`${namespace} ${line}\n`);
        }
      } catch {
        process.stderr.write(
          `${namespace} [Debug Error: Unable to stringify message]\n`,
        );
      }
    }
  };
}
