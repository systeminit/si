import {
  SiEntity,
  OpType,
  OpSource,
  OpSet,
  OpTombstone,
  OpUnset,
} from "../src/siEntity";
import _ from "lodash";

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

describe("getCodeDecorations", () => {
  test("returns code decorations for yaml", () => {
    const { entity } = setupTest();
    const stuff = entity.getCodeDecorations("baseline", []);
    expect(stuff.length).toBe(3);
  });
});

function setupNumberTest(): TestData {
  const entity = new SiEntity({ entityType: "yamlNumbers" });
  entity.name = "aerosmith";

  entity.addOpSet({
    op: OpType.Set,
    path: ["stringType"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["numberType"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["stringMap", "stringKey"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["numberMap", "numberKey"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["stringArray", "0"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["stringArray", "1"],
    value: "1984",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["numberArray", "0"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["numberArray", "1"],
    value: "1984",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["nestedObject", "objectString"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["nestedObject", "objectNumber"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["nestedObject", "objectStringMap", "eddie"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["nestedObject", "objectNumberMap", "alex"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["nestedObject", "objectStringArray", "0"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["nestedObject", "objectNumberArray", "0"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["nestedObject", "objectArrayArray", "0", "0", "deeplyNestedString"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: ["nestedObject", "objectArrayArray", "0", "0", "deeplyNestedNumber"],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.addOpSet({
    op: OpType.Set,
    path: [
      "nestedObject",
      "objectArrayArray",
      "0",
      "0",
      "deeplyNestedArrayNumber",
      "0",
    ],
    value: "5150",
    source: OpSource.Inferred,
    system: "baseline",
  });

  entity.setDefaultProperties();
  entity.computeProperties();
  return { entity };
}

describe("yamlNumberReplacer", () => {
  test("stringType", () => {
    const { entity } = setupNumberTest();
    const numberifiedProperties = entity.yamlNumberReplacer();

    const stringType = _.get(numberifiedProperties, ["baseline", "stringType"]);
    expect(stringType).toBe("5150");
  });

  test("numberType", () => {
    const { entity } = setupNumberTest();
    const numberifiedProperties = entity.yamlNumberReplacer();

    const numberType = _.get(numberifiedProperties, ["baseline", "numberType"]);
    expect(numberType).toBe(5150);
  });

  test("stringMap", () => {
    const { entity } = setupNumberTest();
    const numberifiedProperties = entity.yamlNumberReplacer();

    const stringMapValue = _.get(numberifiedProperties, [
      "baseline",
      "stringMap",
    ]);
    expect(stringMapValue).toStrictEqual({ stringKey: "5150" });
  });

  test("numberMap", () => {
    const { entity } = setupNumberTest();
    const numberifiedProperties = entity.yamlNumberReplacer();

    const numberMapValue = _.get(numberifiedProperties, [
      "baseline",
      "numberMap",
    ]);
    expect(numberMapValue).toStrictEqual({ numberKey: 5150 });
  });

  test("stringArray", () => {
    const { entity } = setupNumberTest();
    const numberifiedProperties = entity.yamlNumberReplacer();

    const stringArray = _.get(numberifiedProperties, [
      "baseline",
      "stringArray",
    ]);
    expect(stringArray).toStrictEqual(["5150", "1984"]);
  });

  test("numberArray", () => {
    const { entity } = setupNumberTest();
    const numberifiedProperties = entity.yamlNumberReplacer();

    const numberArray = _.get(numberifiedProperties, [
      "baseline",
      "numberArray",
    ]);
    expect(numberArray).toStrictEqual([5150, 1984]);
  });

  test("nestedObject", () => {
    const { entity } = setupNumberTest();
    const numberifiedProperties = entity.yamlNumberReplacer();

    const nestedObject = _.get(numberifiedProperties, [
      "baseline",
      "nestedObject",
    ]);
    expect(nestedObject).toStrictEqual({
      objectString: "5150",
      objectNumber: 5150,
      objectStringMap: {
        eddie: "5150",
      },
      objectNumberMap: {
        alex: 5150,
      },
      objectStringArray: ["5150"],
      objectNumberArray: [5150],
      objectArrayArray: [
        [
          {
            deeplyNestedString: "5150",
            deeplyNestedNumber: 5150,
            deeplyNestedArrayNumber: [5150],
          },
        ],
      ],
    });
  });
});
