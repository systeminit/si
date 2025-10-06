import {
  assertObjectMatch,
  assertRejects,
} from "https://deno.land/std@0.224.0/assert/mod.ts";
import { join } from "https://deno.land/std@0.224.0/path/mod.ts";
import { describe, it } from "https://deno.land/std@0.224.0/testing/bdd.ts";
import Joi from "npm:joi";
import { executeFunction, FunctionKind } from "../src/function.ts";
import { AnyFunction, RequestCtx } from "../src/request.ts";

let lastLog = "";
console.log = (msg: string) => {
  console.dir(msg);
  lastLog = msg;
};

const FUNCS_FOLDER = "./bin/lang-js/tests/functions/";

type FuncOrFuncLocation = string | (() => unknown);

interface FuncScenario {
  name: string;
  kind: FunctionKind;
  funcSpec: AnyFunction;
  func?: FuncOrFuncLocation;
  before?: {
    arg: Record<string, unknown>;
    codeBase64: string;
    handler: string;
  }[];
  result?: unknown;
  timeout?: number;
}

const scenarios: FuncScenario[] = [
  {
    name: "Schema Variant with connection annotations",
    kind: FunctionKind.SchemaVariantDefinition,
    funcSpec: {
      value: {},
      codeBase64: "",
      handler: "main",
    },
    func: "schema-socket.ts",
  },
  {
    name: "Schema Variant with connection annotations",
    kind: FunctionKind.SchemaVariantDefinition,
    funcSpec: {
      value: {},
      codeBase64: "", // We rewrite this later
      handler: "main",
    },
    func: "schema-validation.ts",
  },
  {
    name: "Action Run",
    kind: FunctionKind.ActionRun,
    funcSpec: {
      value: {},
      codeBase64: "", // We rewrite this later
      handler: "main",
    },
    func: "actionRun.ts",
  },
  {
    name: "Before funcs",
    kind: FunctionKind.ActionRun,
    funcSpec: {
      value: {},
      codeBase64: "", // We rewrite this later
      handler: "main",
    },
    func: "beforeFuncs.ts",
    before: [
      {
        arg: { username: "name" },
        codeBase64: btoa(`
        function main(arg) {
          console.log("Running Before 1");
          console.log(\`My arg is \${JSON.stringify(arg)}\`);
          requestStorage.setEnv("b1", true);
          requestStorage.setEnv("b2", true);
        }
      `),
        handler: "main",
      },
      {
        arg: {},
        codeBase64: btoa(`
        function main(arg) {
          console.log("Running Before 2");
          console.log(\`My arg is \${JSON.stringify(arg)}\`);
          requestStorage.deleteEnv("b2");
          requestStorage.setEnv("b3", "I'm a string");
        }
      `),
        handler: "main",
      },
    ],
    result: {
      protocol: "result",
      status: "failure",
      error: {
        kind: "ActionFieldWrongType",
        message: "The message field type must be string when status is either \"warning\" or \"error\"",
      },
    },
  },
  {
    name: "Joi Validation Number Success",
    kind: FunctionKind.Validation,
    funcSpec: {
      value: 1,
      validationFormat: JSON.stringify(Joi.number().describe()),
      codeBase64: "",
      handler: "main",
    },
  },
  {
    name: "Joi Validation Number Failure",
    kind: FunctionKind.Validation,
    funcSpec: {
      value: "foobar",
      validationFormat: JSON.stringify(Joi.number().describe()),
      codeBase64: "",
      handler: "main",
    },
    result: {
      protocol: "result",
      status: "success",
      error: '"value" must be a number',
    },
  },
  {
    name: "Joi Validation String Success",
    kind: FunctionKind.Validation,
    funcSpec: {
      value: "foobar",
      validationFormat: JSON.stringify(Joi.string().describe()),
      codeBase64: "",
      handler: "main",
    },
  },
  {
    name: "Joi Validation String Failure",
    kind: FunctionKind.Validation,
    funcSpec: {
      value: 1,
      validationFormat: JSON.stringify(Joi.string().describe()),
      codeBase64: "",
      handler: "main",
    },
    result: {
      protocol: "result",
      status: "success",
      error: '"value" must be a string',
    },
  },
  {
    name: "Joi Validation Bad JSON",
    kind: FunctionKind.Validation,
    funcSpec: {
      value: 1,
      validationFormat: "''",
      codeBase64: "",
      handler: "main",
    },
    result: {
      protocol: "result",
      status: "failure",
      error: {
        kind: {
          UserCodeException: "Error",
        },
        message: "Invalid JSON format",
      },
    },
  },
  {
    name: "Joi Validation Bad Format",
    kind: FunctionKind.Validation,
    funcSpec: {
      value: 1,
      validationFormat: JSON.stringify("test"),
      codeBase64: "",
      handler: "main",
    },
    result: {
      protocol: "result",
      status: "failure",
      error: {
        kind: {
          UserCodeException: "Error",
        },
        message: "validationFormat test is wrong: ValidationError: \"value\" must be of type object",
      },
    },
  },
];

// This is the test suite timeout in seconds.
const testSuiteTimeout = 30;

async function base64FromFile(path: string) {
  const buffer = await Deno.readFile(join(Deno.cwd(), path));
  return btoa(new TextDecoder().decode(buffer));
}

describe("executeFunction", () => {
  it("Name", () => {
    const format = Joi.number().integer().min(0).max(2).required();
    const string = JSON.stringify(format.describe());
    console.log(string);
  });

  for (const scenario of scenarios) {
    it(scenario.name, async () => {
      lastLog = "";
      let codeBase64: string;

      if (scenario.func) {
        if (typeof scenario.func === "function") {
          const rawCode = scenario.func.toString();
          const code = `function ${rawCode}`;
          codeBase64 = btoa(code);
        } else {
          codeBase64 = await base64FromFile(FUNCS_FOLDER + scenario.func);
        }
      } else {
        codeBase64 = scenario.funcSpec.codeBase64;
      }

      const ctx: RequestCtx = {
        executionId: "",
      };

      const funcObj: AnyFunction = {
        ...scenario.funcSpec,
        codeBase64,
      };

      await executeFunction({
        kind: scenario.kind,
        ...ctx,
        ...funcObj,
        before: scenario.before,
      }, testSuiteTimeout * 1000);

      const parsedLog = JSON.parse(lastLog);
      assertObjectMatch(
        parsedLog,
        (scenario.result ?? {
          protocol: "result",
          status: "success",
        }) as Record<PropertyKey, unknown>,
      );
    });
  }
});
