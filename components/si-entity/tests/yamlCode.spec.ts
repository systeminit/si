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
  entity.setDefaultProperties();
  entity.computeProperties();
  return { entity };
}

describe("_YAMLDocToPaths", () => {
  test("returns YamlDocPaths", () => {
    const { entity } = setupTest();
    const stuff = entity.getCodeDecorations("baseline", []);
    expect(stuff.length).toBe(3);
  });
});
