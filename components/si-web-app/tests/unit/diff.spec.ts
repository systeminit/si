import { expect } from "chai";
import { diffEntity } from "@/utils/diff";
import _ from "lodash";

const emptyEntity = null;
const startEntity = {
  id: "whatever",
  name: "james",
  description: "ageofapes",
  siStorable: {
    typeName: "application",
    changeSetId: "",
  },
  siProperties: {
    billingAccountId: "foo",
  },
  properties: {
    kind: "Application",
    replicas: 55,
    favorites: ["death", "entombed", "at the gates"],
    friends: [
      {
        name: "alex",
      },
      {
        name: "mahir",
        nice: "person",
      },
      {
        name: "donald trump",
      },
      {
        name: "fletcher",
      },
    ],
    tags: [
      {
        key: "b",
        value: "two",
      },
      {
        key: "c",
        value: "three",
      },
      {
        key: "d",
        value: "four",
      },
    ],
    containers: [
      {
        name: "adam",
        path: "/poop",
        ports: [
          {
            internalPort: 55,
          },
        ],
      },
    ],
  },
  constraints: {
    apeKind: "chimp",
  },
  implicitConstraints: {
    ape: true,
  },
};

describe("diff", () => {
  describe("new entity", () => {
    it("returns no entries and a count of 0", () => {
      const result = diffEntity(emptyEntity, startEntity);
      expect(result)
        .to.have.property("entries")
        .and.lengthOf(0);
      expect(result.count).eql(0);
    });
  });

  describe("left vs right", () => {
    it("is identical", () => {
      const result = diffEntity(startEntity, startEntity);
      expect(result)
        .to.have.property("entries")
        .and.lengthOf(0);
      expect(result.count).eql(0);
    });

    it("is renamed", () => {
      const renamedEntity = _.cloneDeep(startEntity);
      renamedEntity.name = "paul";
      const result = diffEntity(startEntity, renamedEntity);
      expect(result)
        .to.have.property("entries")
        .and.lengthOf(1);
      expect(result.count).eql(1);
      expect(result.entries[0]).to.eql({
        path: ["name"],
        before: "james",
        after: "paul",
        kind: "edit",
      });
    });

    it("has a new description", () => {
      const changedEntity = _.cloneDeep(startEntity);
      changedEntity.description = "snake pliskin";
      const result = diffEntity(startEntity, changedEntity);
      expect(result)
        .to.have.property("entries")
        .and.lengthOf(1);
      expect(result.count).eql(1);
      expect(result.entries[0]).to.eql({
        path: ["description"],
        before: "ageofapes",
        after: "snake pliskin",
        kind: "edit",
      });
    });

    it("checks properties that are strings", () => {
      const changedEntity = _.cloneDeep(startEntity);
      changedEntity.properties.kind = "dillinger escape plan";
      const result = diffEntity(startEntity, changedEntity);
      expect(result)
        .to.have.property("entries")
        .and.lengthOf(1);
      expect(result.count).eql(1);
      expect(result.entries[0]).to.eql({
        path: ["properties", "kind"],
        before: "Application",
        after: "dillinger escape plan",
        kind: "edit",
      });
    });

    it("checks properties that are numbers", () => {
      const changedEntity = _.cloneDeep(startEntity);
      changedEntity.properties.replicas = 1;
      const result = diffEntity(startEntity, changedEntity);
      expect(result)
        .to.have.property("entries")
        .and.lengthOf(1);
      expect(result.count).eql(1);
      expect(result.entries[0]).to.eql({
        path: ["properties", "replicas"],
        before: 55,
        after: 1,
        kind: "edit",
      });
    });
    it("checks properties that are arrays", () => {
      const changedEntity = _.cloneDeep(startEntity);
      changedEntity.properties.favorites = [
        "death",
        "alice in chains",
        "at the gates",
        "van halen",
      ];
      const result = diffEntity(startEntity, changedEntity);
      expect(result)
        .to.have.property("entries")
        .and.lengthOf(2);
      expect(result.count).eql(2);
      expect(result.entries[0]).to.eql({
        path: ["properties", "favorites", 1],
        before: "entombed",
        after: "alice in chains",
        kind: "edit",
      });
      expect(result.entries[1]).to.eql({
        path: ["properties", "favorites", 3],
        before: undefined,
        after: "van halen",
        kind: "add",
      });
    });
    it("checks properties that are arrays and have deleted elements", () => {
      const changedEntity = _.cloneDeep(startEntity);
      changedEntity.properties.favorites = [];
      const result = diffEntity(startEntity, changedEntity);
      expect(result)
        .to.have.property("entries")
        .and.lengthOf(3);
      expect(result.count).eql(3);
      expect(result.entries[0]).to.eql({
        path: ["properties", "favorites", 0],
        before: "death",
        after: undefined,
        kind: "delete",
      });
      expect(result.entries[1]).to.eql({
        path: ["properties", "favorites", 1],
        before: "entombed",
        after: undefined,
        kind: "delete",
      });
      expect(result.entries[2]).to.eql({
        path: ["properties", "favorites", 2],
        before: "at the gates",
        after: undefined,
        kind: "delete",
      });
    });
    it("checks properties that are arrays of objects", () => {
      const changedEntity = _.cloneDeep(startEntity);
      changedEntity.properties.friends[2].name = "joe biden";
      changedEntity.properties.friends.push({
        name: "sunil",
      });

      const result = diffEntity(startEntity, changedEntity);
      expect(result)
        .to.have.property("entries")
        .and.lengthOf(2);
      expect(result.count).eql(2);
      expect(result.entries[0]).to.eql({
        path: ["properties", "friends", 2, "name"],
        before: "donald trump",
        after: "joe biden",
        kind: "edit",
      });
      expect(result.entries[1]).to.eql({
        path: ["properties", "friends", 4, "name"],
        before: undefined,
        after: "sunil",
        kind: "add",
      });
    });
    it("checks properties that are arrays of objects and deeply nested", () => {
      const changedEntity = _.cloneDeep(startEntity);
      changedEntity.properties.containers[0].ports[0].internalPort = 65;
      changedEntity.properties.containers.push({
        name: "snoopy",
        path: "/canoe",
        ports: [
          {
            internalPort: 42,
          },
        ],
      });

      const result = diffEntity(startEntity, changedEntity);
      expect(result)
        .to.have.property("entries")
        .and.lengthOf(4);
      expect(result.count).eql(4);
      expect(result.entries[0]).to.eql({
        path: ["properties", "containers", 1, "name"],
        before: undefined,
        after: "snoopy",
        kind: "add",
      });
      expect(result.entries[1]).to.eql({
        path: ["properties", "containers", 1, "path"],
        before: undefined,
        after: "/canoe",
        kind: "add",
      });
      expect(result.entries[2]).to.eql({
        path: ["properties", "containers", 0, "ports", 0, "internalPort"],
        before: 55,
        after: 65,
        kind: "edit",
      });
      expect(result.entries[3]).to.eql({
        path: ["properties", "containers", 1, "ports", 0, "internalPort"],
        before: undefined,
        after: 42,
        kind: "add",
      });
    });
    it("checks properties that are maps with a stable sort order", () => {
      const changedEntity = _.cloneDeep(startEntity);
      changedEntity.properties.tags = [
        {
          key: "b",
          value: "two",
        },
        {
          key: "d",
          value: "something new",
        },
        {
          key: "c",
          value: "three",
        },
        {
          key: "a",
          value: "one",
        },
      ];
      const result = diffEntity(startEntity, changedEntity);
      expect(result)
        .to.have.property("entries")
        .and.lengthOf(3);
      expect(result.count).eql(3);
      expect(result.entries[0]).to.eql({
        path: ["properties", "tags", 0, "key"],
        before: undefined,
        after: "a",
        kind: "add",
      });
      expect(result.entries[1]).to.eql({
        path: ["properties", "tags", 0, "value"],
        before: undefined,
        after: "one",
        kind: "add",
      });
      expect(result.entries[2]).to.eql({
        path: ["properties", "tags", 3, "value"],
        before: "four",
        after: "something new",
        kind: "edit",
      });
    });
  });
});
