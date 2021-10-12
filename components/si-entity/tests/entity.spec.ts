import { findProp } from "si-registry";
import {
  IEntity,
  Entity,
  emptyValueForContainerProp,
  updateOpSet,
  updateDependencies,
  removeDependencies,
  generateProperties,
  isTombstoned,
  isOpSetOverridden,
} from "../src/entity";
import { OpSource, OpSet } from "../src/ops";
import _ from "lodash";
import { Tombstone } from "../src/tombstone";
import { Inference } from "si-inference";

interface EntityCreateArgs {
  entityType: string;
}

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

function createFake({ entityType }: EntityCreateArgs): IEntity {
  return {
    id: "fake",
    entityType,
    nodeId: "fake",
    name: "fake",
    ops: [],
    tombstones: [],
    siStorable: {
      typeName: entityType,
      objectId: "fake",
      billingAccountId: "fake",
      organizationId: "fake",
      workspaceId: "fake",
      tenantIds: ["fake"],
      deleted: false,
    },
    dependencies: {},
    properties: {},
    code: {},
  };
}

describe("Entity", () => {
  describe("updateOpSet", () => {
    test("adds opSet to ops array if it does not exist", () => {
      const entity = createFake({ entityType: "torture" });
      const opSet = OpSet.createManual({
        entityType: "torture",
        value: "death",
        path: ["standardString"],
        system: "baseline",
      });
      updateOpSet(entity, opSet);
      expect(entity.ops).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            value: "death",
          }),
        ]),
      );
    });

    test("updates the existing opSet if the path and system match", () => {
      const entity = createFake({ entityType: "torture" });

      const opSet = OpSet.createManual({
        entityType: "torture",
        value: "death",
        path: ["standardString"],
        system: "baseline",
      });
      updateOpSet(entity, opSet);

      const mastodonOpSet = OpSet.createManual({
        entityType: "torture",
        value: "mastodon",
        path: ["standardString"],
        system: "baseline",
      });
      updateOpSet(entity, mastodonOpSet);

      expect(entity.ops).not.toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            value: "death",
          }),
        ]),
      );

      expect(entity.ops).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            value: "mastodon",
          }),
        ]),
      );
    });
  });

  describe("removeDependencies", () => {
    test("removes dependency array for given id", () => {
      const entity = createFake({ entityType: "torture" });
      updateDependencies(entity, { opSetId: "1", dependencies: ["2", "3"] });
      expect(entity.dependencies).toEqual(
        expect.objectContaining({
          "1": ["2", "3"],
        }),
      );
      removeDependencies(entity, "1");
      expect(entity.dependencies).not.toEqual(
        expect.objectContaining({
          "1": ["2", "3"],
        }),
      );
    });
  });

  describe("updateDependencies", () => {
    test("sets the dependencies", () => {
      const entity = createFake({ entityType: "torture" });
      updateDependencies(entity, { opSetId: "1", dependencies: ["2", "3"] });
      expect(entity.dependencies).toEqual(
        expect.objectContaining({
          "1": ["2", "3"],
        }),
      );
    });
  });

  describe("emptyValueForContainerProp", () => {
    test("returns {} for object", () => {
      const prop = findProp(["torture", "nestedTest"]);
      const value = emptyValueForContainerProp(prop);
      expect(value).toEqual({});
    });
    test("returns [] for arrays", () => {
      const prop = findProp(["torture", "complexArray"]);
      const value = emptyValueForContainerProp(prop);
      expect(value).toEqual([]);
    });
    test("returns {} for maps", () => {
      const prop = findProp(["torture", "mappy"]);

      const value = emptyValueForContainerProp(prop);
      expect(value).toEqual({});
    });
    test("throws an error on non container prop types", () => {
      const prop = findProp(["torture", "standardString"]);
      expect(() => {
        emptyValueForContainerProp(prop);
      }).toThrowErrorMatchingSnapshot();
    });
  });

  describe("findOpSetForPathAndSystem", () => {
    test("returns undefined if not found", () => {
      const entity = createFake({ entityType: "torture" });
      const result = Entity.findOpSetForPathAndSystem(entity, {
        path: ["shoes"],
        system: "baseline",
      });
      expect(result).toBe(undefined);
    });
    test("returns opSet if found", () => {
      const entity = createFake({ entityType: "torture" });
      Entity.setManualValue(entity, {
        path: ["standardString"],
        system: "baseline",
        value: "death",
      });
      const result = Entity.findOpSetForPathAndSystem(entity, {
        path: ["standardString"],
        system: "baseline",
      });
      expect(result).toEqual(expect.objectContaining({ value: "death" }));
    });
    test("returns a baseline opSet if found when asking for a specific system", () => {
      const entity = createFake({ entityType: "torture" });
      Entity.setManualValue(entity, {
        path: ["standardString"],
        system: "baseline",
        value: "death",
      });
      const result = Entity.findOpSetForPathAndSystem(entity, {
        path: ["standardString"],
        system: "verge",
      });
      expect(result).toEqual(expect.objectContaining({ value: "death" }));
    });
  });

  describe("setManualValue", () => {
    test("adds an op for the manual value", () => {
      const entity = createFake({ entityType: "torture" });
      const result = Entity.setManualValue(entity, {
        path: ["standardString"],
        system: "baseline",
        value: "death",
      });
      expect(result).toBe(entity);
      expect(result.ops).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            path: ["standardString"],
            system: "baseline",
            value: "death",
          }),
        ]),
      );
      const op = _.find(result.ops, (o) => o.value == "death");
      const depObj: Record<string, string[]> = {};
      depObj[op.id] = [];
      expect(result.dependencies).toEqual(expect.objectContaining(depObj));
    });
    describe("nested values", () => {
      describe("object with nested string", () => {
        test("adds an op for each missing nested value along with the final value", () => {
          const entity = createFake({ entityType: "torture" });
          const result = Entity.setManualValue(entity, {
            path: ["nestedTest", "squirrel", "name"],
            system: "baseline",
            value: "monkey",
          });
          expect(result.ops).toEqual(
            expect.arrayContaining([
              expect.objectContaining({
                path: ["nestedTest"],
                system: "baseline",
                value: {},
              }),
              expect.objectContaining({
                path: ["nestedTest", "squirrel"],
                system: "baseline",
                value: {},
              }),
              expect.objectContaining({
                path: ["nestedTest", "squirrel", "name"],
                system: "baseline",
                value: "monkey",
              }),
            ]),
          );
        });
      });
    });
  });

  describe("addTombstone", () => {
    test("adds tombstone when no matching tombstoned exists", () => {
      const entity = createFake({ entityType: "torture" });
      const tombstone: Tombstone = {
        path: ["standardString"],
        system: "baseline",
        source: OpSource.Manual,
      };
      Entity.addTombstone(entity, tombstone);
      expect(entity.tombstones).toEqual(
        expect.arrayContaining([expect.objectContaining(tombstone)]),
      );
    });

    test("does not add duplicate tombstones", () => {
      const entity = createFake({ entityType: "torture" });
      const tombstone: Tombstone = {
        path: ["standardString"],
        system: "baseline",
        source: OpSource.Manual,
      };
      Entity.addTombstone(entity, tombstone);
      Entity.addTombstone(entity, tombstone);
      Entity.addTombstone(entity, tombstone);
      expect(entity.tombstones).toEqual(
        expect.arrayContaining([expect.objectContaining(tombstone)]),
      );
      expect(entity.tombstones.length).toBe(1);
    });
  });

  describe("removeTombstone", () => {
    test("removes tombstone from the list", () => {
      const entity = createFake({ entityType: "torture" });
      const tombstone: Tombstone = {
        path: ["standardString"],
        system: "baseline",
        source: OpSource.Manual,
      };
      Entity.addTombstone(entity, tombstone);
      expect(entity.tombstones).toEqual(
        expect.arrayContaining([expect.objectContaining(tombstone)]),
      );
      Entity.removeTombstone(entity, tombstone);
      expect(entity.tombstones).not.toEqual(
        expect.arrayContaining([expect.objectContaining(tombstone)]),
      );
    });

    test("does nothing when tombstone is already gone", () => {
      const entity = createFake({ entityType: "torture" });
      const tombstone: Tombstone = {
        path: ["standardString"],
        system: "baseline",
        source: OpSource.Manual,
      };
      Entity.addTombstone(entity, tombstone);
      expect(entity.tombstones).toEqual(
        expect.arrayContaining([expect.objectContaining(tombstone)]),
      );
      Entity.removeTombstone(entity, tombstone);
      expect(entity.tombstones).not.toEqual(
        expect.arrayContaining([expect.objectContaining(tombstone)]),
      );
      Entity.removeTombstone(entity, tombstone);
      expect(entity.tombstones).not.toEqual(
        expect.arrayContaining([expect.objectContaining(tombstone)]),
      );
    });
  });

  describe("isTombstoned", () => {
    test("returns false it is not tombstoned", () => {
      const entity = createFake({ entityType: "torture" });
      const op = {
        path: ["standardString"],
        system: "baseline",
        value: "metallica",
      };
      Entity.setManualValue(entity, op);
      const result = isTombstoned(entity, { source: OpSource.Manual, ...op });
      expect(result).toBe(false);
    });

    test("returns false with tombstones that do not match", () => {
      const entity = createFake({ entityType: "torture" });
      const op = {
        path: ["standardString"],
        system: "baseline",
        value: "metallica",
      };
      const stringArrayTombstone = {
        ...op,
        source: OpSource.Manual,
        path: ["stringArray"],
      };
      Entity.addTombstone(entity, stringArrayTombstone);
      const inferredTombstone = { ...op, source: OpSource.Inferred };
      Entity.addTombstone(entity, inferredTombstone);
      Entity.setManualValue(entity, op);
      const result = isTombstoned(entity, { ...op, source: OpSource.Manual });
      expect(result).toBe(false);
    });

    test("returns true if tombstoned by exact path", () => {
      const entity = createFake({ entityType: "torture" });
      const op = {
        path: ["standardString"],
        system: "baseline",
        value: "metallica",
      };
      Entity.setManualValue(entity, op);
      const tombstone = { source: OpSource.Manual, ...op };
      Entity.addTombstone(entity, tombstone);
      const result = isTombstoned(entity, tombstone);
      expect(result).toBe(true);
    });

    test("returns true if tombstoned by sub path", () => {
      const entity = createFake({ entityType: "torture" });
      const op = {
        path: ["nestedTest", "squirrel", "name"],
        system: "baseline",
        value: "metallica",
      };
      Entity.setManualValue(entity, op);
      const tombstone = {
        source: OpSource.Manual,
        path: ["nestedTest"],
        ...op,
      };
      Entity.addTombstone(entity, tombstone);
      const result = isTombstoned(entity, tombstone);
      expect(result).toBe(true);
    });
  });

  describe("isOpSetOverriden", () => {
    describe("inferred", () => {
      describe("baseline", () => {
        test("is not overriden when no other values exist", () => {
          const entity = createFake({ entityType: "torture" });
          const op = {
            path: ["standardString"],
            system: "baseline",
            value: "metallica",
            provenance: mockProvenance,
          };
          Entity.setInferredValue(entity, op);
          const result = isOpSetOverridden(entity, {
            opSet: { ...op, source: OpSource.Inferred },
            system: "baseline",
          });
          expect(result).toBe(false);
        });
        test("is overriden when an inferred system value exists", () => {
          const entity = createFake({ entityType: "torture" });
          const ops = [
            {
              path: ["standardString"],
              system: "baseline",
              value: "metallica",
              provenance: mockProvenance,
            },
            {
              path: ["standardString"],
              system: "system:1",
              value: "cuba",
              provenance: mockProvenance,
            },
          ];
          for (const op of ops) {
            Entity.setInferredValue(entity, op);
          }

          const baselineResult = isOpSetOverridden(entity, {
            opSet: { ...ops[0], source: OpSource.Inferred },
            system: "baseline",
          });
          expect(baselineResult).toBe(false);

          const systemResult = isOpSetOverridden(entity, {
            opSet: { ...ops[0], source: OpSource.Inferred },
            system: "system:1",
          });
          expect(systemResult).toBe(true);
        });
        test("is overriden when a manual baseline value exists", () => {
          const entity = createFake({ entityType: "torture" });
          const ops = [
            {
              path: ["standardString"],
              system: "baseline",
              value: "metallica",
              provenance: mockProvenance,
            },
          ];
          for (const op of ops) {
            Entity.setInferredValue(entity, op);
          }
          const manualOp = {
            path: ["standardString"],
            system: "baseline",
            value: "cuba",
          };
          Entity.setManualValue(entity, manualOp);

          const baselineResult = isOpSetOverridden(entity, {
            opSet: { ...ops[0], source: OpSource.Inferred },
            system: "baseline",
          });
          expect(baselineResult).toBe(true);
        });
      });

      describe("system", () => {
        test("is not overriden when no other values exist", () => {
          const entity = createFake({ entityType: "torture" });
          const op = {
            path: ["standardString"],
            system: "system:1",
            value: "metallica",
            provenance: mockProvenance,
          };
          Entity.setInferredValue(entity, op);
          const result = isOpSetOverridden(entity, {
            opSet: { ...op, source: OpSource.Inferred },
            system: "system:1",
          });
          expect(result).toBe(false);
        });
        test("is not overriden when an inferred baseline value exists", () => {
          const entity = createFake({ entityType: "torture" });
          const baselineOp = {
            path: ["standardString"],
            system: "baseline",
            value: "alice",
            provenance: mockProvenance,
          };
          Entity.setInferredValue(entity, baselineOp);
          const op = {
            path: ["standardString"],
            system: "system:1",
            value: "metallica",
            provenance: mockProvenance,
          };
          Entity.setInferredValue(entity, op);
          const result = isOpSetOverridden(entity, {
            opSet: { ...op, source: OpSource.Inferred },
            system: "system:1",
          });
          expect(result).toBe(false);
        });
        test("is overriden when a manual value exists for the same system", () => {
          const entity = createFake({ entityType: "torture" });
          const manualOp = {
            path: ["standardString"],
            system: "system:1",
            value: "alice",
          };
          Entity.setManualValue(entity, manualOp);
          const op = {
            path: ["standardString"],
            system: "system:1",
            value: "metallica",
            provenance: mockProvenance,
          };
          Entity.setInferredValue(entity, op);

          const result = isOpSetOverridden(entity, {
            opSet: { ...op, source: OpSource.Inferred },
            system: "system:1",
          });
          expect(result).toBe(true);
        });
      });
    });
    describe.only("manual", () => {
      describe("baseline", () => {
        test("is not overriden when no other values exist", () => {
          const entity = createFake({ entityType: "torture" });
          const op = {
            path: ["standardString"],
            system: "baseline",
            value: "metallica",
          };
          Entity.setManualValue(entity, op);
          const result = isOpSetOverridden(entity, {
            opSet: { ...op, source: OpSource.Manual },
            system: "baseline",
          });
          expect(result).toBe(false);
        });
        test("is not overriden when inferred values also exist", () => {
          const entity = createFake({ entityType: "torture" });
          const op = {
            path: ["standardString"],
            system: "baseline",
            value: "metallica",
          };
          Entity.setManualValue(entity, op);

          const inferredOp = {
            path: ["standardString"],
            system: "baseline",
            value: "metallica",
            provenance: mockProvenance,
          };
          Entity.setInferredValue(entity, inferredOp);

          const result = isOpSetOverridden(entity, {
            opSet: { ...op, source: OpSource.Manual },
            system: "baseline",
          });
          expect(result).toBe(false);
        });
        test("is overriden when a manual system value exists", () => {
          const entity = createFake({ entityType: "torture" });
          const op = {
            path: ["standardString"],
            system: "baseline",
            value: "metallica",
          };
          Entity.setManualValue(entity, op);

          const manualOp = {
            path: ["standardString"],
            system: "system:1",
            value: "metallica",
          };
          Entity.setManualValue(entity, manualOp);

          const result = isOpSetOverridden(entity, {
            opSet: { ...op, source: OpSource.Manual },
            system: "system:1",
          });
          expect(result).toBe(true);
        });
        test("is overridden when an inferred system value exists", () => {
          const entity = createFake({ entityType: "torture" });
          const op = {
            path: ["standardString"],
            system: "baseline",
            value: "metallica",
          };
          Entity.setManualValue(entity, op);

          const inferredOp = {
            path: ["standardString"],
            system: "system:1",
            value: "metallica",
            provenance: mockProvenance,
          };
          Entity.setInferredValue(entity, inferredOp);

          const result = isOpSetOverridden(entity, {
            opSet: { ...op, source: OpSource.Manual },
            system: "system:1",
          });
          expect(result).toBe(true);
        });
      });
    });
  });

  //describe("generateProperties", () => {
  //  describe("no tombstones", () => {
  //    describe("baseline values", () => {
  //      test("sets a string value for a root path", () => {
  //        const entity = createFake({ entityType: "torture" });
  //        Entity.setManualValue(entity, {
  //          path: ["standardString"],
  //          system: "baseline",
  //          value: "metallica",
  //        });
  //        const result = generateProperties(entity);
  //        expect(result.properties).toEqual(
  //          expect.objectContaining({
  //            baseline: {
  //              standardString: "metallica",
  //            },
  //          }),
  //        );
  //      });
  //    });
  //  });
  //});
});
