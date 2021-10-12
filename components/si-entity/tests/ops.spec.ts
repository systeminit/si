import { OpSet, OpSource, validatePathForEntityType } from "../src/ops";
import { validate as uuidValidate } from "uuid";
import { Inference } from "si-inference";

const mockInference: Inference = {
  kind: "lambda",
  name: "poop",
  from: [{ targetEntity: true, data: [{ name: true }] }],
  to: { name: true },
  code: "firstEntity.name",
};

const mockContext = [{ id: "1", entityType: "poop" }];

const mockProvenance = {
  inference: mockInference,
  context: mockContext,
};

describe("OpSet", () => {
  describe("validatePathForEntityType", () => {
    test("returns true when the path exists", () => {
      expect(
        validatePathForEntityType({
          path: ["standardString"],
          entityType: "torture",
        }),
      ).toBe(true);
    });
    test("returns false when the path does not exist", () => {
      expect(
        validatePathForEntityType({
          path: ["sloopJohnB"],
          entityType: "torture",
        }),
      ).toBe(false);
    });
  });

  describe("createManual", () => {
    test("returns a manual OpSet", () => {
      const opSet = OpSet.createManual({
        entityType: "torture",
        value: "death",
        path: ["standardString"],
        system: "baseline",
      });
      expect(uuidValidate(opSet.id)).toBe(true);
      expect(opSet.type).toBe("set");
      expect(opSet.path).toStrictEqual(["standardString"]);
      expect(opSet.value).toBe("death");
      expect(opSet.system).toBe("baseline");
      expect(opSet.source).toBe(OpSource.Manual);
    });

    test("throws exception on invalid path", () => {
      expect(() => {
        OpSet.createManual({
          entityType: "torture",
          value: "a",
          path: ["b"],
          system: "baseline",
        });
      }).toThrowError();
    });
  });

  describe("createInferred", () => {
    test("returns an inferred OpSet", () => {
      const opSet = OpSet.createInferred({
        entityType: "torture",
        value: "death",
        path: ["standardString"],
        system: "baseline",
        provenance: mockProvenance,
      });
      expect(uuidValidate(opSet.id)).toBe(true);
      expect(opSet.type).toBe("set");
      expect(opSet.path).toStrictEqual(["standardString"]);
      expect(opSet.value).toBe("death");
      expect(opSet.system).toBe("baseline");
      expect(opSet.source).toBe(OpSource.Inferred);
      expect(opSet.provenance).toStrictEqual(mockProvenance);
    });

    test("throws exception on invalid path", () => {
      expect(() => {
        OpSet.createManual({
          entityType: "torture",
          value: "a",
          path: ["b"],
          system: "baseline",
        });
      }).toThrowError();
    });
  });

  describe("isEqual", () => {
    test("ops are equal if the path, value, system and source are equal", () => {
      const left = OpSet.createInferred({
        entityType: "torture",
        value: "death",
        path: ["standardString"],
        system: "baseline",
        provenance: mockProvenance,
      });
      const right = OpSet.createInferred({
        entityType: "torture",
        value: "death",
        path: ["standardString"],
        system: "baseline",
        provenance: mockProvenance,
      });

      const result = OpSet.isEqual(left, right);
      expect(result).toBe(true);
    });

    test("ops are not equal if the value is not equal", () => {
      const left = OpSet.createInferred({
        entityType: "torture",
        value: "speaking",
        path: ["standardString"],
        system: "baseline",
        provenance: mockProvenance,
      });
      const right = OpSet.createInferred({
        entityType: "torture",
        value: "sparkling",
        path: ["standardString"],
        system: "baseline",
        provenance: mockProvenance,
      });

      const result = OpSet.isEqual(left, right);
      expect(result).toBe(false);
    });

    test("ops are not equal if the path is not equal", () => {
      const left = OpSet.createInferred({
        entityType: "torture",
        value: "speaking",
        path: ["standardString"],
        system: "baseline",
        provenance: mockProvenance,
      });
      const right = OpSet.createInferred({
        entityType: "torture",
        value: "speaking",
        path: ["biggerText"],
        system: "baseline",
        provenance: mockProvenance,
      });

      const result = OpSet.isEqual(left, right);
      expect(result).toBe(false);
    });

    test("ops are not equal if the system is not equal", () => {
      const left = OpSet.createInferred({
        entityType: "torture",
        value: "speaking",
        path: ["standardString"],
        system: "baseline",
        provenance: mockProvenance,
      });
      const right = OpSet.createInferred({
        entityType: "torture",
        value: "speaking",
        path: ["standardString"],
        system: "system:hello",
        provenance: mockProvenance,
      });

      const result = OpSet.isEqual(left, right);
      expect(result).toBe(false);
    });

    test("ops are not equal if the source is not equal", () => {
      const left = OpSet.createInferred({
        entityType: "torture",
        value: "speaking",
        path: ["standardString"],
        system: "baseline",
        provenance: mockProvenance,
      });
      const right = OpSet.createManual({
        entityType: "torture",
        value: "speaking",
        path: ["standardString"],
        system: "baseline:hello",
      });

      const result = OpSet.isEqual(left, right);
      expect(result).toBe(false);
    });
  });
});
