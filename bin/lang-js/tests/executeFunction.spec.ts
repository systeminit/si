import * as fs from "fs/promises";
import { executeFunction, FunctionKind } from "../src/function";
import { AnyFunction, RequestCtx } from "../src/request";

let lastLog = "";
const consoleSpy = jest.spyOn(console, "log").mockImplementation((msg) => {
  console.dir(msg);
  lastLog = msg;
});

const FUNCS_FOLDER = "./tests/functions/";

type FuncOrFuncLocation = string | (() => unknown);

interface FuncScenario {
  kind: FunctionKind;
  funcSpec: AnyFunction;
  func: FuncOrFuncLocation;
  before?: { handler: string; func: FuncOrFuncLocation }[];
}

const scenarios: FuncScenario[] = [
  {
    kind: FunctionKind.ActionRun,
    funcSpec: {
      value: {},
      handler: "workit",
      codeBase64: "", // We rewrite this later
    },
    func: "actionRun.ts",
  },
  {
    kind: FunctionKind.Validation,
    funcSpec: {
      value: {},
      handler: "main",
      codeBase64: "", // We rewrite this later
    },
    func: "validation.ts",
  },
  {
    kind: FunctionKind.Validation,
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
      },
      {
        handler: "before2",
        func: "beforeFuncs.ts",
      },
    ],
  },
];

describe("executeFunction", () => {
  scenarios.forEach((scenario, i) => {
    test(`Scenario ${i}`, async () => {
      consoleSpy.mockClear();
      lastLog = "";
      let codeBase64: string;

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

      const ctx: RequestCtx = {
        executionId: "",
      };

      const funcObj: AnyFunction = {
        ...scenario.funcSpec,
        codeBase64,
      };

      const before = [];

      for (const b of scenario.before ?? []) {
        before.push({
          handler: b.handler,
          codeBase64: await base64FromFile(FUNCS_FOLDER + b.func),
        });
      }

      await executeFunction(scenario.kind, {
        ...ctx,
        ...funcObj,
        before,
      });

      const parsedLog = JSON.parse(lastLog);

      // If there's a test that necessitates an execution failure
      // we could bring status from the scenario
      expect(parsedLog).toMatchObject({
        protocol: "result",
        status: "success",
      });
    });
  });
});

async function base64FromFile(path: string) {
  return (await fs.readFile(path)).toString("base64");
}
