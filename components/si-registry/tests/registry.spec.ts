import { findProp } from "../src/registry";

describe("registry", () => {
  describe("findProperty", () => {
    test("simple property", () => {
      const simpleString = findProp(["leftHandPath", "simpleString"]);
      expect(simpleString).not.toBeUndefined();
      expect(simpleString.name).toBe("simpleString");
      expect(simpleString.type).toBe("string");
    });
    test("nested property", () => {
      const simpleString = findProp(["leftHandPath", "party", "poop"]);
      expect(simpleString).not.toBeUndefined();
      expect(simpleString.name).toBe("poop");
      expect(simpleString.type).toBe("string");
    });
    test("array property with no index", () => {
      const prop = findProp(["leftHandPath", "abnormal", "illusion"]);
      expect(prop).toEqual(
        expect.objectContaining({
          name: "illusion",
          type: "string",
        }),
      );
    });
    test("array property with index", () => {
      const prop = findProp(["leftHandPath", "abnormal", "0", "illusion"]);
      expect(prop).toEqual(
        expect.objectContaining({
          name: "illusion",
          type: "string",
        }),
      );
    });
    test("array property with trailing index", () => {
      const prop = findProp(["leftHandPath", "abnormal", "0"]);
      expect(prop).toEqual(
        expect.objectContaining({
          name: "abnormal",
          type: "array",
        }),
      );
    });
  });
});
