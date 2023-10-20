import * as fs from "fs/promises";
import {executeFunction, FunctionKind} from "../src/function";
import {AnyFunction, RequestCtx} from "../src/request";
import {BeforeFunc} from "../src/function_kinds/before";

let lastLog = "";
const consoleSpy = jest.spyOn(console, "log").mockImplementation((msg) => {
  lastLog = msg
})

const FUNCS_FOLDER = './tests/functions/';

interface FuncScenario {
  kind: FunctionKind,
  funcSpec: AnyFunction,
  func: string | (() => unknown);
  before?: BeforeFunc[];
}

const scenarios: FuncScenario[] = [
  {
    kind: FunctionKind.Validation,
    funcSpec: {
      value: {},
      handler: "func",
      codeBase64: "" // We rewrite this later
    },
    func() {
      console.log("help");
      return {
        valid: true,
        message: "CIDR Blocks must be between /16 and /28 netmask",
      };
    }
  },
  {
    kind: FunctionKind.Validation,
    funcSpec: {
      value: {},
      handler: "main",
      codeBase64: "" // We rewrite this later
    },
    func: "validation.ts"
  },

]

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
          code = `function ${rawCode}`
        } else {
          code = `const ${scenario.funcSpec.handler} = ${rawCode}`
        }

        codeBase64 = Buffer.from(code).toString("base64");
      } else {
        codeBase64 = (await fs.readFile(FUNCS_FOLDER + scenario.func)).toString('base64');
      }

      const ctx: RequestCtx = {
        executionId: ""
      };

      const funcObj: AnyFunction = {
        ...scenario.funcSpec,
        codeBase64,
      }

      await executeFunction(FunctionKind.Validation, {
        ...ctx,
        ...funcObj,
      })

      const {protocol, status} = JSON.parse(lastLog);

      // If there's a test that necessitates an execution failure
      // we could bring status from the scenario
      expect(protocol).toBe("result")
      expect(status).toBe("success")
    })
  })


});
