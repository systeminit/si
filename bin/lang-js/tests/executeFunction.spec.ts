import * as fs from "fs/promises";
import Joi from "joi";
import { describe, expect, fail, test, vi } from "vitest";
import { executeFunction, FunctionKind } from "../src/function";
import { AnyFunction, RequestCtx } from "../src/request";

let lastLog = "";
const consoleSpy = vi.spyOn(console, "log").mockImplementation((msg) => {
  console.dir(msg);
  lastLog = msg;
});

const FUNCS_FOLDER = "./tests/functions/";

type FuncOrFuncLocation = string | (() => unknown);

interface FuncScenario {
  name: string;
  kind: FunctionKind;
  funcSpec: AnyFunction;
  func?: FuncOrFuncLocation;
  before?: {
    handler: string;
    func: FuncOrFuncLocation;
    arg: Record<string, any>;
  }[];
  result?: any;
  timeout?: number;
}

const scenarios: FuncScenario[] = [
  {
    name: "Schema Variant with connection annotations",
    kind: FunctionKind.SchemaVariantDefinition,
    funcSpec: {
      value: {},
      handler: "main",
      codeBase64: "", // We rewrite this later
    },
    func: "schema-socket.ts",
  },
  {
    name: "Schema Variant with connection annotations",
    kind: FunctionKind.SchemaVariantDefinition,
    funcSpec: {
      value: {},
      handler: "main",
      codeBase64: "", // We rewrite this later
    },
    func: "schema-validation.ts",
  },
  {
    name: "Action Run",
    kind: FunctionKind.ActionRun,
    funcSpec: {
      value: {},
      handler: "workit",
      codeBase64: "", // We rewrite this later
    },
    func: "actionRun.ts",
  },
  {
    name: "Before funcs",
    kind: FunctionKind.ActionRun,
    funcSpec: {
      value: {},
      handler: "main",
      codeBase64: "", // We rewrite this later
    },
    func: "beforeFuncs.ts",
    before: [
      {
        handler: "before1",
        func: "beforeFuncs.ts",
        arg: { username: "name" },
      },
      {
        handler: "before2",
        func: "beforeFuncs.ts",
        arg: {},
      },
    ],
  },
  {
    name: "Joi Validation Number Success",
    kind: FunctionKind.Validation,
    funcSpec: {
      value: 1,
      validationFormat: JSON.stringify(Joi.number().describe()),
      handler: "",
      codeBase64: "",
    },
  },
  {
    name: "Joi Validation Number Failure",
    kind: FunctionKind.Validation,
    funcSpec: {
      value: "foobar",
      validationFormat: JSON.stringify(Joi.number().describe()),
      handler: "",
      codeBase64: "",
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
      handler: "",
      codeBase64: "",
    },
  },
  {
    name: "Joi Validation String Failure",
    kind: FunctionKind.Validation,
    funcSpec: {
      value: 1,
      validationFormat: JSON.stringify(Joi.string().describe()),
      handler: "",
      codeBase64: "",
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
      handler: "",
      codeBase64: "",
    },
    result: {
      protocol: "result",
      status: "failure",
      error: {
        kind: {
          UserCodeException: "JoiValidationJsonParsingError",
        },
      },
    },
  },
  {
    name: "Joi Validation Bad Format",
    kind: FunctionKind.Validation,
    funcSpec: {
      value: 1,
      validationFormat: JSON.stringify("test"),
      handler: "",
      codeBase64: "",
    },
    result: {
      protocol: "result",
      status: "failure",
      error: {
        kind: {
          UserCodeException: "JoiValidationFormatError",
        },
        message: "validationFormat must be of type object",
      },
    },
  },
  {
    name: "Will Timeout",
    kind: FunctionKind.ActionRun,
    funcSpec: {
      value: {},
      handler: "main",
      codeBase64: "", // We rewrite this later
    },
    func: "willTimeout.ts",
    timeout: 1,
  },
];

// This is the test suite timeout in seconds.
const testSuiteTimeout = 30;

describe("executeFunction", () => {
  test("Name", () => {
    const format = Joi.number().integer().min(0).max(2)
      .required();
    const string = JSON.stringify(format.describe());
    console.log(string);
  });

  test.each(scenarios)(
    "$name",
    async (scenario) => {
      consoleSpy.mockClear();
      lastLog = "";
      let codeBase64: string;

      if (scenario.func) {
        if (typeof scenario.func === "function") {
          // If we get a function from the scenario object we need to get its
          // string representation and make it a valid function definition
          // function.toString() is a wild thing :)
          const rawCode = scenario.func.toString();

          let code: string;
          if (rawCode.startsWith("func()")) {
            code = `function ${rawCode}`;
          } else {
            code = `const ${scenario.funcSpec.handler} = ${rawCode}`;
          }

          codeBase64 = Buffer.from(code).toString("base64");
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

      const before = [];

      for (const { func, handler, arg } of scenario.before ?? []) {
        before.push({
          handler,
          codeBase64: await base64FromFile(FUNCS_FOLDER + func),
          arg,
        });
      }

      if (scenario.timeout) {
        try {
          await executeFunction(scenario.kind, {
            ...ctx,
            ...funcObj,
            before,
          }, scenario.timeout);
          fail("expected function to hit timeout, but no error was thrown");
        } catch (error) {
          expect(error.message).toBe(
            `function timed out after ${scenario.timeout} seconds`,
          );
        }
      } else {
        await executeFunction(scenario.kind, {
          ...ctx,
          ...funcObj,
          before,
        }, testSuiteTimeout * 1000);
        const parsedLog = JSON.parse(lastLog);
        expect(parsedLog).toMatchObject(
          scenario.result ?? {
            protocol: "result",
            status: "success",
          },
        );
      }
    },
  );
}, testSuiteTimeout * 1000);

async function base64FromFile(path: string) {
  const buffer = await fs.readFile(path);
  return buffer.toString("base64");
}
