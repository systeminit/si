import {
  SiEntity,
  OpType,
  OpSource,
  OpSet,
  OpTombstone,
  OpUnset,
} from "../src/siEntity";

interface TestData {
  entity: SiEntity;
}

function setupTest(): TestData {
  const entity = new SiEntity({ entityType: "k8sDeployment" });
  entity.name = "slayer";
  entity.addOpSet({
    op: OpType.Set,
    path: ["spec", "replicas"],
    value: 15,
    source: OpSource.Manual,
    system: "baseline",
  });
  entity.addOpSet({
    op: OpType.Set,
    path: ["metadata", "0", "again"],
    value: "riding",
    source: OpSource.Manual,
    system: "baseline",
  });
  entity.addOpSet({
    op: OpType.Set,
    path: ["metadata", "0", "me"],
    value: "15",
    source: OpSource.Inferred,
    system: "baseline",
  });
  entity.setDefaultProperties();
  entity.computeProperties();
  return { entity };
}

describe("_YAMLDocToPaths", () => {
  test("returns YamlDocPaths", () => {
    const { entity } = setupTest();
    const stuff = entity.getCodeDecorations("baseline");
    expect(stuff.length).toBe(3); // The two default values, plus [metadata, 0, me] - everything that's inferred
  });
});
