import { executeRemoteFunction } from "../src/remote_function";

describe("remoteFunction", () => {
  describe("executeRemoteFunction", () => {
    test("fails when an exception is returned", () => {
      const result = executeRemoteFunction({
        kind: "resolver",
        code: 'throw new Error("wtf")',
        containerImage: "poop",
        containerTag: "tree",
      });
      expect(result).toEqual(
        expect.objectContaining({
          error: {
            message: "wtf",
            name: "Error",
          },
          kind: "resolver",
          status: "failure",
        })
      );
    });
    test("fails when a function is returned", () => {
      const result = executeRemoteFunction({
        kind: "resolver",
        code: "let f = function ape(p) { return p; }; f",
        containerImage: "poop",
        containerTag: "tree",
      });
      expect(result).toEqual(
        expect.objectContaining({
          error: {
            message:
              "Only strings, numbers, booleans, objects, arrays and null are allowed!",
            name: "InvalidReturnType",
          },
          kind: "resolver",
          status: "failure",
        })
      );
    });
    test("when undefined is returned, the value is unset", () => {
      const result = executeRemoteFunction({
        kind: "resolver",
        code: "undefined",
        containerImage: "poop",
        containerTag: "tree",
      });
      expect(result).toEqual(
        expect.objectContaining({
          kind: "resolver",
          status: "success",
          unset: true,
        })
      );
    });
    test("numbers can be returned successfully, and the value is set", () => {
      const result = executeRemoteFunction({
        kind: "resolver",
        code: "1",
        containerImage: "poop",
        containerTag: "tree",
      });
      expect(result).toEqual(
        expect.objectContaining({
          kind: "resolver",
          status: "success",
          data: 1,
          unset: false,
        })
      );
    });
    test("strings can be returned successfully, and the value is set", () => {
      const result = executeRemoteFunction({
        kind: "resolver",
        code: "'hardSkool'",
        containerImage: "poop",
        containerTag: "tree",
      });
      expect(result).toEqual(
        expect.objectContaining({
          kind: "resolver",
          status: "success",
          data: "hardSkool",
          unset: false,
        })
      );
    });
    test("booleans can be returned successfully, and the value is set", () => {
      const result = executeRemoteFunction({
        kind: "resolver",
        code: "true",
        containerImage: "poop",
        containerTag: "tree",
      });
      expect(result).toEqual(
        expect.objectContaining({
          kind: "resolver",
          status: "success",
          data: true,
          unset: false,
        })
      );
    });
    test("objects can be returned successfully, and the value is set", () => {
      const result = executeRemoteFunction({
        kind: "resolver",
        code: "f = { love: { you: 'more' } }",
        containerImage: "poop",
        containerTag: "tree",
      });
      expect(result).toEqual(
        expect.objectContaining({
          kind: "resolver",
          status: "success",
          data: { love: { you: "more" } },
          unset: false,
        })
      );
    });
    test("arrays can be returned successfully, and the value is set", () => {
      const result = executeRemoteFunction({
        kind: "resolver",
        code: "f = ['p']",
        containerImage: "poop",
        containerTag: "tree",
      });
      expect(result).toEqual(
        expect.objectContaining({
          kind: "resolver",
          status: "success",
          data: ["p"],
          unset: false,
        })
      );
    });
  });
});
