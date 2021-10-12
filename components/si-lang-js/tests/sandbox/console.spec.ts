import { consoleObject } from "../../src/sandbox/console";

const consoleSpy = jest.spyOn(console, "log");

describe("console", () => {
  beforeEach(() => {
    consoleSpy.mockClear();
  });
  describe("log", () => {
    test("prints the output line as structured data", () => {
      consoleObject.log("rock and roll aint noise pollution", {
        acdc: "rules",
      });
      expect(console.log).toBeCalledTimes(1);
      const logLineJson = consoleSpy.mock.calls[0][0];
      const logLine = JSON.parse(logLineJson);
      expect(logLine).toEqual(
        expect.objectContaining({
          stream: "stdout",
          level: "info",
          group: "log",
          message: "rock and roll aint noise pollution",
          data: { acdc: "rules" },
          timestamp: expect.anything(),
        })
      );
    });
  });
});
