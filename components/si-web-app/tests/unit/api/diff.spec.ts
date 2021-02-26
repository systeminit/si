import { Entity, IEntity } from "@/api/sdf/model/entity";
import { diffEntity } from "@/api/diff";
import _ from "lodash";
import { SiChangeSetEvent } from "@/api/sdf/model/siChangeSet";

const baseEntity: IEntity = {
  id: "entity:2",
  name: "cinderella",
  description: "steam train",
  objectType: "funkyMonkey",
  expressionProperties: {},
  manualProperties: {},
  inferredProperties: {},
  properties: {
    __baseline: {
      tough: "snakes",
      songs: {
        "shake me": "dont break me",
      },
      artifacts: [
        {
          people: "make their way",
        },
        {
          nomore: "coming home",
        },
      ],
    },
  },
  nodeId: "node:2",
  head: true,
  base: false,
  siStorable: {
    typeName: "funkyMoney",
    objectId: "entity:2",
    billingAccountId: "ba:2",
    organizationId: "o:2",
    workspaceId: "w:2",
    tenantIds: ["ba:2"],
    updateClock: {
      epoch: 0,
      updateCount: 1,
    },
    deleted: false,
  },
  siChangeSet: {
    changeSetId: "cs:1",
    editSessionId: "es:1",
    event: SiChangeSetEvent.Create,
    orderClock: {
      epoch: 2,
      updateCount: 33,
    },
  },
};

describe("diff", () => {
  test("returns nothing if they are equal", async () => {
    let lhs = new Entity(baseEntity);
    let rhs = new Entity(baseEntity);
    let results = diffEntity(lhs, rhs);
    expect(results.count).toBe(0);
    expect(results.entries.length).toBe(0);
  });

  test("new fields result in add", async () => {
    let lhs = new Entity(_.cloneDeep(baseEntity));
    let rhs = new Entity(_.cloneDeep(baseEntity));
    rhs.properties.__baseline["poop"] = "pants";
    let results = diffEntity(lhs, rhs);
    expect(results.count).toBe(1);
    expect(results.entries.length).toBe(1);
    let diffEntry = _.find(results.entries, [
      "path",
      ["properties", "__baseline", "poop"],
    ]);
    expect(diffEntry).not.toBeUndefined();
    if (diffEntry) {
      expect(diffEntry.kind).toBe("add");
      expect(diffEntry.path).toStrictEqual([
        "properties",
        "__baseline",
        "poop",
      ]);
      expect(diffEntry.after).toBe("pants");
      expect(diffEntry.before).toBe(undefined);
    }
  });

  test("changed fields result in edit", async () => {
    let lhs = new Entity(_.cloneDeep(baseEntity));
    let rhs = new Entity(_.cloneDeep(baseEntity));
    rhs.properties.__baseline["tough"] = "love";
    let results = diffEntity(lhs, rhs);
    expect(results.count).toBe(1);
    expect(results.entries.length).toBe(1);
    let diffEntry = _.find(results.entries, [
      "path",
      ["properties", "__baseline", "tough"],
    ]);
    expect(diffEntry).not.toBeUndefined();
    if (diffEntry) {
      expect(diffEntry.kind).toBe("edit");
      expect(diffEntry.path).toStrictEqual([
        "properties",
        "__baseline",
        "tough",
      ]);
      expect(diffEntry.after).toBe("love");
      expect(diffEntry.before).toBe("snakes");
    }
  });

  test("removed fields result in delete", async () => {
    let lhs = new Entity(_.cloneDeep(baseEntity));
    let rhs = new Entity(_.cloneDeep(baseEntity));
    delete rhs.properties.__baseline["tough"];
    let results = diffEntity(lhs, rhs);
    console.log("test", { entries: results.entries });
    expect(results.count).toBe(1);
    expect(results.entries.length).toBe(1);
    let diffEntry = _.find(results.entries, [
      "path",
      ["properties", "__baseline", "tough"],
    ]);
    expect(diffEntry).not.toBeUndefined();
    if (diffEntry) {
      expect(diffEntry.kind).toBe("delete");
      expect(diffEntry.path).toStrictEqual([
        "properties",
        "__baseline",
        "tough",
      ]);
      expect(diffEntry.after).toBe(undefined);
      expect(diffEntry.before).toBe("snakes");
    }
  });
});
