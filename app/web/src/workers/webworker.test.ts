/* eslint-disable no-useless-escape */
import * as Comlink from "comlink";
import { SqlValue } from "@sqlite.org/sqlite-wasm";
import {
  TabDBInterface,
  BustCacheFn,
  NOROW,
  WorkspacePatchBatch,
  AtomDocument,
  OutgoingConnections,
  WorkspaceIndexUpdate,
  MessageKind,
} from "@/workers/types/dbinterface";
import { WorkspacePk } from "@/newhotness/types";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import {
  AttributeTree,
  BifrostComponent,
  CachedDefaultVariant,
  ComponentInList,
  EddaComponent,
  EntityKind,
  IncomingConnections,
} from "./types/entity_kind_types";

// setup a few things
const workerUrl =
  import.meta.env.VITE_SI_ENV === "local"
    ? "/src/workers/webworker.ts"
    : "webworker.js";

const bustTanStackCache: BustCacheFn = (
  _workspaceId: string,
  _changeSetId: string,
  _kind: string,
  _id: string,
) => {};

/**
 * TEST OUTPUT
 */
const logElm = document.querySelector("#logs > ul");
const errElm = document.querySelector("#errors > ul");

const createMsg = (msg: string) => {
  const li = document.createElement("li");
  li.appendChild(document.createTextNode(msg));
  return li;
};

const log = (msg: string) => {
  logElm?.append(createMsg(msg));
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const assert = (value: any, msg: string) => {
  if (value) return; // if true, don't log anything
  errElm?.append(createMsg(msg));
};
const done = () => {
  const elm = document.getElementById("timestamp");
  const stamp = document.createTextNode(new Date().toString());
  elm?.appendChild(stamp);
};

const oneInOne = (rows: SqlValue[][]): SqlValue | typeof NOROW => {
  const first = rows[0];
  if (first) {
    const id = first[0];
    if (id || id === 0) return id;
  }
  return NOROW;
};

/**
 * THE TEST
 */
const workspaceId = "01HRFEV0S23R1G23RP75QQDCA7";
const changeSetId = "01JYPTEC5JM3T1Y4ECEPT9560J";

const fullDiagnosticTest = async (db: Comlink.Remote<TabDBInterface>) => {
  log("~~ DIAGNOSTIC STARTED ~~");
  // NOTE STARTING ON HEAD
  const coldstart = await db.niflheim(workspaceId, changeSetId);
  assert(coldstart, "initial niflheim failed");
  log("niflheim completed");

  const atoms = await db.exec({
    sql: "select count(*) from atoms",
    returnValue: "resultRows",
  });
  const atomCnt = oneInOne(atoms);
  assert(atomCnt === 106, `${atomCnt?.toString()} initial atoms`);
  const mtm = await db.exec({
    sql: "select count(*) from index_mtm_atoms",
    returnValue: "resultRows",
  });
  const mtmCnt = oneInOne(mtm);
  assert(mtmCnt === 106, `${mtmCnt?.toString()} initial mtm`);

  const changeSet = await db.exec({
    sql: "select count(*) from changesets",
    returnValue: "resultRows",
  });
  const changeSetCnt = oneInOne(changeSet);
  assert(changeSetCnt === 1, "1 initial change set");

  const indexes = await db.exec({
    sql: "select count(*) from indexes",
    returnValue: "resultRows",
  });
  const indexestCnt = oneInOne(indexes);
  assert(indexestCnt === 1, "1 initial index");
  log("initial data exists");

  const component = (await db.get(
    workspaceId,
    changeSetId,
    EntityKind.ComponentInList,
    "01JZK1VKDN40NZCCTYR350RDFD",
  )) as ComponentInList;
  assert(component.name === "si-1835", `${component.name} not named si-1835`);
  log("si-1835 exists");

  const componentListStr = await db.getList(
    workspaceId,
    changeSetId,
    EntityKind.ComponentList,
    workspaceId,
  );
  const componentList = JSON.parse(componentListStr) as ComponentInList[];
  assert(
    componentList.length === 11,
    `${componentList.length} != 11 components`,
  );
  log("11 components");

  db.exec({
    sql: "delete from atoms where kind = ? and args = ?",
    bind: [EntityKind.ComponentInList, "01JZK1VKDN40NZCCTYR350RDFD"],
  });

  const componentAgain = await db.get(
    workspaceId,
    changeSetId,
    EntityKind.ComponentInList,
    "01JZK1VKDN40NZCCTYR350RDFD",
  );
  assert(componentAgain === -1, "si-1835 wasn't deleted");

  db.exec({
    sql: "delete from index_mtm_atoms where index_checksum = ? and kind = ? and args = ? and checksum = ?",
    bind: [
      "8601f40a5bc4b73e90dba9d1a4126bfd",
      EntityKind.Component,
      "01K3GTCTNVYW3NGHP33G5F31XD",
      "2ea75387de1dec40e3ea2726cf718361",
    ],
  });

  const component2 = await db.get(
    workspaceId,
    changeSetId,
    EntityKind.Component,
    "01K3GTCTNVYW3NGHP33G5F31XD",
  );
  assert(component2 === -1, "gimmie an action wasn't deleted");

  await db.niflheim(workspaceId, changeSetId);

  const component2Back = (await db.get(
    workspaceId,
    changeSetId,
    EntityKind.Component,
    "01K3GTCTNVYW3NGHP33G5F31XD",
  )) as BifrostComponent;
  assert(
    component2Back.id === "01K3GTCTNVYW3NGHP33G5F31XD",
    "gimmie an action wasn't deleted",
  );

  const componentBack = (await db.get(
    workspaceId,
    changeSetId,
    EntityKind.ComponentInList,
    "01JZK1VKDN40NZCCTYR350RDFD",
  )) as ComponentInList;
  assert(componentBack.name === "si-1835", "not named si-1835");

  log("niflheim restored deleted atoms");

  // SOME QUICK HELPERS ON HEAD

  const oneView = await db.getComponentsInOnlyOneView(workspaceId, changeSetId);
  assert(
    Object.keys(oneView).length === 11,
    `should be 11 components in one view ${Object.keys(oneView).length}`,
  );

  const cDetails = await db.getComponentDetails(workspaceId, changeSetId);
  assert(
    cDetails["01JZK1VKDN40NZCCTYR350RDFD"]?.name === "si-1835",
    "details name isn't si-1835",
  );

  const search = new Set(
    (await db.queryAttributes(workspaceId, changeSetId, [
      { key: "schema", value: "AWS::EC2::VPC", op: "exact" },
    ])) as string[],
  );
  assert(search.size === 1, `didn't find 1 VPC: ${search}`);
  assert(search.has("01JZK1VKDN40NZCCTYR350RDFD"), "bad VPC id");

  // NOW WE'RE IN A DIFFERENT CHANGE SET
  const patch1 = JSON.parse(`
{
  "meta": {
    "workspaceId": "01HRFEV0S23R1G23RP75QQDCA7",
    "changeSetId": "01K4X1YZBFV7FW6WZ0594TY6TZ",
    "toIndexChecksum": "b54ef494932554dc664cfc66674f8b53",
    "fromIndexChecksum": "8601f40a5bc4b73e90dba9d1a4126bfd"
  },
  "kind": "PatchMessage",
  "patches": [
    {
      "kind": "AttributeTree",
      "id": "01JZND7T6F6VEWK5Q0FBNFYQBZ",
      "fromChecksum": "d803a3f1f77af406c40bd6a5a18690dd",
      "toChecksum": "37613302b232b201b30614946bb6c927",
      "patch": [
        {
          "op": "replace",
          "path": "/attributeValues/01JZND7T84VJFQ5KXBT6WDK1WY/value",
          "value": "i changed your name"
        },
        {
          "op": "replace",
          "path": "/componentName",
          "value": "i changed your name"
        }
      ]
    },
    {
      "kind": "DependentValueComponentList",
      "id": "01HRFEV0S23R1G23RP75QQDCA7",
      "fromChecksum": "0b15c7cda9f4d72095a1fc086564a721",
      "toChecksum": "8d728a32c8055c6aa4bf3bc9cb778aa4",
      "patch": [
        {
          "op": "add",
          "path": "/componentIds/0",
          "value": "01JZND7T6F6VEWK5Q0FBNFYQBZ"
        }
      ]
    },
    {
      "kind": "ActionViewList",
      "id": "01HRFEV0S23R1G23RP75QQDCA7",
      "fromChecksum": "d9d8534e9357c3ec74dafec3a3356964",
      "toChecksum": "8d67434bf4e082f24971b3b86b8746e0",
      "patch": [
        {
          "op": "replace",
          "path": "/actions/1/componentName",
          "value": "i changed your name"
        }
      ]
    },
    {
      "kind": "Component",
      "id": "01JZND7T6F6VEWK5Q0FBNFYQBZ",
      "fromChecksum": "b1e4a2d1f49c3a007f15196fb1db1a4a",
      "toChecksum": "d243d7c33063fa0aa7e893884b8b36fe",
      "patch": [
        {
          "op": "replace",
          "path": "/name",
          "value": "i changed your name"
        },
        {
          "op": "replace",
          "path": "/resourceDiff/current",
          "value": null
        },
        {
          "op": "replace",
          "path": "/resourceDiff/diff",
          "value": " {   \\"si\\": {-    \\"name\\": \\"foobar\\",+    \\"name\\": \\"i changed your name\\",     \\"type\\": \\"component\\",     \\"color\\": \\"#FF9900\\"   },   \\"domain\\": {     \\"KeyType\\": \\"rsa\\",     \\"awsResourceType\\": \\"key-pair\\",     \\"region\\": \\"us-east-1\\",     \\"tags\\": {       \\"Name\\": \\"foobar\\"     }   } }"
        }
      ]
    },
    {
      "kind": "ComponentInList",
      "id": "01JZND7T6F6VEWK5Q0FBNFYQBZ",
      "fromChecksum": "d5de7efd45f8aa614a36bb56b0832b22",
      "toChecksum": "af77a652217388bb1af4b54bc3ea7ad7",
      "patch": [
        {
          "op": "replace",
          "path": "/name",
          "value": "i changed your name"
        },
        {
          "op": "replace",
          "path": "/diffStatus",
          "value": "Modified"
        }
      ]
    },
    {
      "kind": "ComponentDiff",
      "id": "01JZND7T6F6VEWK5Q0FBNFYQBZ",
      "fromChecksum": "0f3b704b25806eb0acf02530530b712e",
      "toChecksum": "8fdd9772ab507e498ceaa77c1364d6d5",
      "patch": [
        {
          "op": "replace",
          "path": "/diffStatus",
          "value": "Modified"
        },
        {
          "op": "add",
          "path": "/attributeDiffs/~1si~1name",
          "value": {
            "new": {
              "$source": {
                "value": "i changed your name"
              },
              "$value": "i changed your name"
            },
            "old": {
              "$source": {
                "value": "foobar"
              },
              "$value": "foobar"
            }
          }
        },
        {
          "op": "replace",
          "path": "/resourceDiff/current",
          "value": null
        },
        {
          "op": "replace",
          "path": "/resourceDiff/diff",
          "value": " {   \\"si\\": {-    \\"name\\": \\"foobar\\",+    \\"name\\": \\"i changed your name\\",     \\"type\\": \\"component\\",     \\"color\\": \\"#FF9900\\"   },   \\"domain\\": {     \\"KeyType\\": \\"rsa\\",     \\"awsResourceType\\": \\"key-pair\\",     \\"region\\": \\"us-east-1\\",     \\"tags\\": {       \\"Name\\": \\"foobar\\"     }   } }"
        }
      ]
    }
  ]
}
  `) as WorkspacePatchBatch;
  const newChangeSetId = patch1.meta.changeSetId;
  await db.linkNewChangeset(workspaceId, changeSetId, newChangeSetId);

  // ensure we get the list update!
  let atomGotUpdated = false;
  db.addAtomUpdated(
    Comlink.proxy(
      (
        _workspaceId: WorkspacePk,
        changeSetId: ChangeSetId,
        kind: EntityKind,
        id: string,
        _data: AtomDocument,
        _listIds: string[],
        _removed: boolean,
        _noBroadcast?: boolean,
      ) => {
        if (
          changeSetId === newChangeSetId &&
          kind === EntityKind.ComponentInList &&
          id === "01JZND7T6F6VEWK5Q0FBNFYQBZ"
        )
          atomGotUpdated = true;
      },
    ),
  );

  await db.handleWorkspacePatchMessage(patch1);

  db.addAtomUpdated(Comlink.proxy(() => {}));

  const renamedComponent = (await db.get(
    workspaceId,
    newChangeSetId,
    EntityKind.Component,
    "01JZND7T6F6VEWK5Q0FBNFYQBZ",
  )) as EddaComponent;
  assert(
    renamedComponent.name === "i changed your name",
    `Name change failed ${renamedComponent.name}`,
  );

  const renamedComponentInList = (await db.get(
    workspaceId,
    newChangeSetId,
    EntityKind.ComponentInList,
    "01JZND7T6F6VEWK5Q0FBNFYQBZ",
  )) as ComponentInList;
  assert(
    renamedComponentInList.name === "i changed your name",
    `Name change failed in list ${renamedComponent.name}`,
  );

  assert(atomGotUpdated, "atom not update received");

  log("patch with renamed component done");

  // ANOTHER CHANGE SET
  const patch2 = JSON.parse(`
{
  "meta": {
    "workspaceId": "01HRFEV0S23R1G23RP75QQDCA7",
    "changeSetId": "01K4X3TK1HA66ERH67XRFS94SC",
    "toIndexChecksum": "c839e37eb3fa5f73c325401177d208ff",
    "fromIndexChecksum": "60f8fc2e2e5e49fc416846b2462d36bc"
  },
  "kind": "PatchMessage",
  "patches": [
    {
      "kind": "IncomingConnections",
      "id": "01K2ZNABR4CQV2051T0Z4FWXDR",
      "fromChecksum": "beb9014f1ae402a7a3f245632169e16b",
      "toChecksum": "2949581539ea0cf58ba4827fce33769e",
      "patch": [
        {
          "op": "add",
          "path": "/connections/0",
          "value": {
            "kind": "prop",
            "fromComponentId": "01JYPVCJC71BRFCK1KZKRMN5HX",
            "fromAttributeValueId": "01JYPVCJD67D7RVXPSE0DA98S7",
            "fromAttributeValuePath": "/domain/region",
            "fromPropId": "01JYPVCJ1BF5PD5P059Z2DD8BX",
            "fromPropPath": "/root/domain/region",
            "toComponentId": "01K2ZNABR4CQV2051T0Z4FWXDR",
            "toPropId": "01JZ6DM7MZPJ874RS4TM5MF8DP",
            "toPropPath": "/root/domain/extra/Region",
            "toAttributeValueId": "01K2ZNABVFXB681FR4B7RZQYRN",
            "toAttributeValuePath": "/domain/extra/Region"
          }
        }
      ]
    },
    {
      "kind": "Component",
      "id": "01K2ZNABR4CQV2051T0Z4FWXDR",
      "fromChecksum": "e954224a3964479ab83669294338e2f6",
      "toChecksum": "88515e9f70368dc620ef249fd9bf0343",
      "patch": [
        {
          "op": "replace",
          "path": "/inputCount",
          "value": 1
        },
        {
          "op": "replace",
          "path": "/resourceDiff/current",
          "value": null
        },
        {
          "op": "replace",
          "path": "/resourceDiff/diff",
          "value": " {   \\"si\\": {     \\"name\\": \\"si-2981\\",     \\"type\\": \\"component\\",     \\"color\\": \\"#FF9900\\"   },   \\"domain\\": {     \\"extra\\": {       \\"AwsPermissionsMap\\": \\"{\\\"create\\\":{\\\"permissions\\\":[\\\"iot:CreateThing\\\",\\\"iot:DescribeThing\\\"]},\\\"delete\\\":{\\\"permissions\\\":[\\\"iot:DeleteThing\\\",\\\"iot:DescribeThing\\\"]},\\\"list\\\":{\\\"permissions\\\":[\\\"iot:ListThings\\\"]},\\\"read\\\":{\\\"permissions\\\":[\\\"iot:DescribeThing\\\"]},\\\"update\\\":{\\\"permissions\\\":[\\\"iot:UpdateThing\\\",\\\"iot:DescribeThing\\\"]}}\\",       \\"AwsResourceType\\": \\"AWS::IoT::Thing\\",       \\"PropUsageMap\\": \\"{\\\"createOnly\\\":[\\\"ThingName\\\"],\\\"updatable\\\":[\\\"AttributePayload\\\",\\\"Attributes\\\"],\\\"secrets\\\":[]}\\"     }   } }"
        }
      ]
    },
    {
      "kind": "ComponentInList",
      "id": "01K2ZNABR4CQV2051T0Z4FWXDR",
      "fromChecksum": "16233a29cb6367b98119717dd0dcf82a",
      "toChecksum": "af78b8e9e988802590bf63d16f1d761f",
      "patch": [
        {
          "op": "replace",
          "path": "/inputCount",
          "value": 1
        }
      ]
    },
    {
      "kind": "ComponentDiff",
      "id": "01K2ZNABR4CQV2051T0Z4FWXDR",
      "fromChecksum": "c74e9fabe7d291723d01f985d5859f82",
      "toChecksum": "0287fe6514284f5e189dcfb44d0eb345",
      "patch": [
        {
          "op": "replace",
          "path": "/diffStatus",
          "value": "Modified"
        },
        {
          "op": "add",
          "path": "/attributeDiffs/~1domain~1extra~1Region",
          "value": {
            "new": {
              "$source": {
                "component": "01JYPVCJC71BRFCK1KZKRMN5HX",
                "path": "/domain/region"
              }
            },
            "old": {
              "$source": {
                "prototype": "si:identity(input socket Region)",
                "fromSchema": true
              }
            }
          }
        },
        {
          "op": "replace",
          "path": "/resourceDiff/current",
          "value": null
        },
        {
          "op": "replace",
          "path": "/resourceDiff/diff",
          "value": " {   \\"si\\": {     \\"name\\": \\"si-2981\\",     \\"type\\": \\"component\\",     \\"color\\": \\"#FF9900\\"   },   \\"domain\\": {     \\"extra\\": {       \\"AwsPermissionsMap\\": \\"{\\\"create\\\":{\\\"permissions\\\":[\\\"iot:CreateThing\\\",\\\"iot:DescribeThing\\\"]},\\\"delete\\\":{\\\"permissions\\\":[\\\"iot:DeleteThing\\\",\\\"iot:DescribeThing\\\"]},\\\"list\\\":{\\\"permissions\\\":[\\\"iot:ListThings\\\"]},\\\"read\\\":{\\\"permissions\\\":[\\\"iot:DescribeThing\\\"]},\\\"update\\\":{\\\"permissions\\\":[\\\"iot:UpdateThing\\\",\\\"iot:DescribeThing\\\"]}}\\",       \\"AwsResourceType\\": \\"AWS::IoT::Thing\\",       \\"PropUsageMap\\": \\"{\\\"createOnly\\\":[\\\"ThingName\\\"],\\\"updatable\\\":[\\\"AttributePayload\\\",\\\"Attributes\\\"],\\\"secrets\\\":[]}\\"     }   } }"
        }
      ]
    },
    {
      "kind": "DependentValueComponentList",
      "id": "01HRFEV0S23R1G23RP75QQDCA7",
      "fromChecksum": "0b15c7cda9f4d72095a1fc086564a721",
      "toChecksum": "c328e5bfba14f915246b4653b59b98d9",
      "patch": [
        {
          "op": "add",
          "path": "/componentIds/0",
          "value": "01K2ZNABR4CQV2051T0Z4FWXDR"
        }
      ]
    },
    {
      "kind": "AttributeTree",
      "id": "01K2ZNABR4CQV2051T0Z4FWXDR",
      "fromChecksum": "b31354220a83c99c42a1c7f68744eafc",
      "toChecksum": "f9371ae5825cdc11946ec229f6347908",
      "patch": [
        {
          "op": "replace",
          "path": "/attributeValues/01K2ZNABVFXB681FR4B7RZQYRN/externalSources",
          "value": [
            {
              "componentId": "01JYPVCJC71BRFCK1KZKRMN5HX",
              "componentName": "us east 1",
              "path": "/domain/region",
              "isSecret": false
            }
          ]
        },
        {
          "op": "replace",
          "path": "/attributeValues/01K2ZNABVFXB681FR4B7RZQYRN/overridden",
          "value": true
        }
      ]
    }
  ]
}
  `) as WorkspacePatchBatch;
  const thingComponent = "01K2ZNABR4CQV2051T0Z4FWXDR";
  const regionComponent = "01JYPVCJC71BRFCK1KZKRMN5HX";
  const avChangeSetId = patch2.meta.changeSetId;

  const incomingPre = (await db.get(
    workspaceId,
    changeSetId, // looking it up on HEAD before this ChangeSet is made!
    EntityKind.IncomingConnections,
    thingComponent,
  )) as IncomingConnections;
  assert(incomingPre.connections.length === 0, "shouldn't have connections");

  const outgoingPre = (await db.getOutgoingConnectionsByComponentId(
    workspaceId,
    changeSetId,
  )) as OutgoingConnections;
  const regionOutPreLength = Object.keys(
    outgoingPre.get(regionComponent),
  ).length;
  // there are connections already
  assert(
    regionOutPreLength > 0,
    "no region outgoing connections before patch?",
  );

  await db.linkNewChangeset(workspaceId, changeSetId, avChangeSetId);
  // ensure 106 MTM on new change set!
  const atoms3 = await db.exec({
    sql: `select count(*)
      from atoms
        join index_mtm_atoms
          on atoms.checksum = index_mtm_atoms.checksum
          and atoms.kind = index_mtm_atoms.kind
          and atoms.args = index_mtm_atoms.args
        join changesets
          on index_mtm_atoms.index_checksum = changesets.index_checksum
      where changesets.change_set_id = ?
    `,
    bind: [avChangeSetId],
    returnValue: "resultRows",
  });
  const atomCnt3 = oneInOne(atoms3);
  assert(
    atomCnt3 === 106,
    `${atomCnt3?.toString()} linked atoms (should be 106)`,
  );

  // cold start "no-op" from new change set calls post process on everything
  await db.niflheim(workspaceId, avChangeSetId);

  await db.handleWorkspacePatchMessage(patch2);

  const incoming = (await db.get(
    workspaceId,
    avChangeSetId,
    EntityKind.IncomingConnections,
    thingComponent,
  )) as IncomingConnections;
  assert(
    incoming.connections[0]?.toComponentId === thingComponent,
    `thing ID is wrong ${incoming.connections[0]?.toComponentId}`,
  );
  assert(
    incoming.connections[0]?.fromComponentId === regionComponent,
    `region component ID is wrong ${incoming.connections[0]?.fromComponentId}`,
  );
  assert(incoming.connections[0]?.kind === "prop", "not a prop");
  if (incoming.connections[0]?.kind === "prop") {
    assert(
      incoming.connections[0]?.toAttributeValuePath === "/domain/extra/Region",
      `path is not extra/Region: ${incoming.connections[0]?.toAttributeValuePath}`,
    );
    assert(
      incoming.connections[0]?.fromAttributeValuePath === "/domain/region",
      `path is not domain/Region: ${incoming.connections[0]?.fromAttributeValuePath}`,
    );
  }

  const outgoingPost = (await db.getOutgoingConnectionsByComponentId(
    workspaceId,
    avChangeSetId,
  )) as OutgoingConnections;
  const regionOutPostLength = Object.keys(
    outgoingPost.get(regionComponent),
  ).length;
  assert(
    regionOutPreLength + 1 === regionOutPostLength,
    `outgoing connections not added ${regionOutPreLength} v ${regionOutPostLength}`,
  );

  const avBeforeDVU = (await db.get(
    workspaceId,
    avChangeSetId,
    EntityKind.AttributeTree,
    thingComponent,
  )) as AttributeTree;
  const regionAv = avBeforeDVU.attributeValues["01K2ZNABVFXB681FR4B7RZQYRN"];
  const source = regionAv?.externalSources?.[0];
  assert(
    source?.path === "/domain/region",
    `source path is not region: ${source?.path}`,
  );
  assert(
    source?.componentId === regionComponent,
    `component is not region: ${source?.componentId}`,
  );
  assert(
    regionAv?.value === null,
    `region value is not blank: ${regionAv?.value}`,
  );

  // NOW THE DVU PATCH
  const patch3 = JSON.parse(`
{
  "meta": {
    "workspaceId": "01HRFEV0S23R1G23RP75QQDCA7",
    "changeSetId": "01K4X3TK1HA66ERH67XRFS94SC",
    "toIndexChecksum": "55dee56fd5dedebec8bcd4f2eeb92964",
    "fromIndexChecksum": "c839e37eb3fa5f73c325401177d208ff"
  },
  "kind": "PatchMessage",
  "patches": [
    {
      "kind": "DependentValueComponentList",
      "id": "01HRFEV0S23R1G23RP75QQDCA7",
      "fromChecksum": "c328e5bfba14f915246b4653b59b98d9",
      "toChecksum": "0b15c7cda9f4d72095a1fc086564a721",
      "patch": [
        {
          "op": "remove",
          "path": "/componentIds/0"
        }
      ]
    },
    {
      "kind": "ComponentInList",
      "id": "01K2ZNABR4CQV2051T0Z4FWXDR",
      "fromChecksum": "af78b8e9e988802590bf63d16f1d761f",
      "toChecksum": "9e4f46e6bcc69479dd61d7101e05f82e",
      "patch": [
        {
          "op": "replace",
          "path": "/diffStatus",
          "value": "Modified"
        }
      ]
    },
    {
      "kind": "Component",
      "id": "01K2ZNABR4CQV2051T0Z4FWXDR",
      "fromChecksum": "88515e9f70368dc620ef249fd9bf0343",
      "toChecksum": "88284e369845a2ff8a02f928659af8b2",
      "patch": [
        {
          "op": "replace",
          "path": "/resourceDiff/diff",
          "value": " {   \\"si\\": {     \\"name\\": \\"si-2981\\",     \\"type\\": \\"component\\",     \\"color\\": \\"#FF9900\\"   },   \\"domain\\": {     \\"extra\\": {+      \\"Region\\": \\"us-east-1\\",       \\"AwsPermissionsMap\\": \\"{\\\"create\\\":{\\\"permissions\\\":[\\\"iot:CreateThing\\\",\\\"iot:DescribeThing\\\"]},\\\"delete\\\":{\\\"permissions\\\":[\\\"iot:DeleteThing\\\",\\\"iot:DescribeThing\\\"]},\\\"list\\\":{\\\"permissions\\\":[\\\"iot:ListThings\\\"]},\\\"read\\\":{\\\"permissions\\\":[\\\"iot:DescribeThing\\\"]},\\\"update\\\":{\\\"permissions\\\":[\\\"iot:UpdateThing\\\",\\\"iot:DescribeThing\\\"]}}\\",       \\"AwsResourceType\\": \\"AWS::IoT::Thing\\",       \\"PropUsageMap\\": \\"{\\\"createOnly\\\":[\\\"ThingName\\\"],\\\"updatable\\\":[\\\"AttributePayload\\\",\\\"Attributes\\\"],\\\"secrets\\\":[]}\\"     }   } }"
        }
      ]
    },
    {
      "kind": "ComponentDiff",
      "id": "01K2ZNABR4CQV2051T0Z4FWXDR",
      "fromChecksum": "0287fe6514284f5e189dcfb44d0eb345",
      "toChecksum": "0f1908991c0884b068607492bf863633",
      "patch": [
        {
          "op": "add",
          "path": "/attributeDiffs/~1domain~1extra~1Region/new/$value",
          "value": "us-east-1"
        },
        {
          "op": "replace",
          "path": "/resourceDiff/diff",
          "value": " {   \\"si\\": {     \\"name\\": \\"si-2981\\",     \\"type\\": \\"component\\",     \\"color\\": \\"#FF9900\\"   },   \\"domain\\": {     \\"extra\\": {+      \\"Region\\": \\"us-east-1\\",       \\"AwsPermissionsMap\\": \\"{\\\"create\\\":{\\\"permissions\\\":[\\\"iot:CreateThing\\\",\\\"iot:DescribeThing\\\"]},\\\"delete\\\":{\\\"permissions\\\":[\\\"iot:DeleteThing\\\",\\\"iot:DescribeThing\\\"]},\\\"list\\\":{\\\"permissions\\\":[\\\"iot:ListThings\\\"]},\\\"read\\\":{\\\"permissions\\\":[\\\"iot:DescribeThing\\\"]},\\\"update\\\":{\\\"permissions\\\":[\\\"iot:UpdateThing\\\",\\\"iot:DescribeThing\\\"]}}\\",       \\"AwsResourceType\\": \\"AWS::IoT::Thing\\",       \\"PropUsageMap\\": \\"{\\\"createOnly\\\":[\\\"ThingName\\\"],\\\"updatable\\\":[\\\"AttributePayload\\\",\\\"Attributes\\\"],\\\"secrets\\\":[]}\\"     }   } }"
        }
      ]
    },
    {
      "kind": "AttributeTree",
      "id": "01K2ZNABR4CQV2051T0Z4FWXDR",
      "fromChecksum": "f9371ae5825cdc11946ec229f6347908",
      "toChecksum": "514367e801669599e15d8346d5fe190e",
      "patch": [
        {
          "op": "replace",
          "path": "/attributeValues/01K2ZNABVFXB681FR4B7RZQYRN/value",
          "value": "us-east-1"
        }
      ]
    }
  ]
}
  `);
  await db.handleWorkspacePatchMessage(patch3);

  const afterDVUIndexExists = oneInOne(
    await db.exec({
      sql: "select checksum from indexes where checksum = ?",
      returnValue: "resultRows",
      bind: ["55dee56fd5dedebec8bcd4f2eeb92964"],
    }),
  );
  assert(
    afterDVUIndexExists === "55dee56fd5dedebec8bcd4f2eeb92964",
    "DVU patch didn't create index",
  );

  const avAfterDVU = (await db.get(
    workspaceId,
    avChangeSetId,
    EntityKind.AttributeTree,
    thingComponent,
  )) as AttributeTree;
  const regionAvAgain =
    avAfterDVU.attributeValues["01K2ZNABVFXB681FR4B7RZQYRN"];
  assert(
    regionAvAgain?.value === "us-east-1",
    `region value is not us-east-1: ${regionAvAgain?.value}`,
  );

  const changeSetAgain = await db.exec({
    sql: "select count(*) from changesets",
    returnValue: "resultRows",
  });
  const changeSetCntAgain = oneInOne(changeSetAgain);
  assert(changeSetCntAgain === 3, "3 change sets now");

  log("patch with AV subscription done");

  // OK, ONE MORE CHANGE SET FROM HEAD WITH A NEW COMPONENT
  // AND TEST THE INDEX UPDATES!
  const addChangeSetId = "01K4ZF6QXKB3ZV3124ER2C0TFT";
  await db.linkNewChangeset(workspaceId, changeSetId, addChangeSetId);
  await db.niflheim(workspaceId, addChangeSetId);

  // the system sends these patches in response to making a new change set
  // so let's be sure we can handle them without error...
  const noOpPatch = JSON.parse(`
{
  "meta": {
    "workspaceId": "01HRFEV0S23R1G23RP75QQDCA7",
    "changeSetId": "01K4ZF6QXKB3ZV3124ER2C0TFT",
    "toIndexChecksum": "8601f40a5bc4b73e90dba9d1a4126bfd-different",
    "fromIndexChecksum": "8601f40a5bc4b73e90dba9d1a4126bfd"
  },
  "kind": "PatchMessage",
  "patches": []
}
  `) as WorkspacePatchBatch;
  await db.handleWorkspacePatchMessage(noOpPatch);

  const noOpIndexExists = oneInOne(
    await db.exec({
      sql: "select checksum from indexes where checksum = ?",
      returnValue: "resultRows",
      bind: ["8601f40a5bc4b73e90dba9d1a4126bfd-different"],
    }),
  );
  assert(
    noOpIndexExists === "8601f40a5bc4b73e90dba9d1a4126bfd-different",
    "noOp patch didn't create index",
  );

  // NOTE: i am changing this data's `toIndexChecksum` to a new value
  // representing other behavior we've seen in the wild, even though
  // the particular run i am copying here didn't exhibit it
  const noOpIndex = JSON.parse(`
{
  "meta": {
    "workspaceId": "01HRFEV0S23R1G23RP75QQDCA7",
    "changeSetId": "01K4ZF6QXKB3ZV3124ER2C0TFT",
    "toIndexChecksum": "8601f40a5bc4b73e90dba9d1a4126bfd-different",
    "fromIndexChecksum": "8601f40a5bc4b73e90dba9d1a4126bfd"
  },
  "kind": "IndexUpdate",
  "indexChecksum": "8601f40a5bc4b73e90dba9d1a4126bfd-different",
  "patch": {
    "kind": "ChangeSetMvIndex",
    "id": "cfe344c9242085c961e343a835de646f409e1b0e9f63290c6415dc7c52ef9b9b",
    "fromChecksum": "8601f40a5bc4b73e90dba9d1a4126bfd",
    "toChecksum": "8601f40a5bc4b73e90dba9d1a4126bfd-different",
    "patch": []
  }
}
  `) as WorkspaceIndexUpdate;
  await db.handleIndexMvPatch(noOpIndex);

  // NOTE: i truncated the `SchemaVariantCategories` patch to re-order
  // the list of all SVs by category because it was 20k lines long
  const addPatch = JSON.parse(`
{
  "meta": {
    "workspaceId": "01HRFEV0S23R1G23RP75QQDCA7",
    "changeSetId": "01K4ZF6QXKB3ZV3124ER2C0TFT",
    "toIndexChecksum": "03ce85f750506ee4f9d64396fdbacad0",
    "fromIndexChecksum": "8601f40a5bc4b73e90dba9d1a4126bfd-different"
  },
  "kind": "PatchMessage",
  "patches": [
    {
      "kind": "ManagementConnections",
      "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
      "fromChecksum": "0",
      "toChecksum": "37b7770cd00c8b9b116a7b80dab12f4e",
      "patch": [
        {
          "op": "replace",
          "path": "",
          "value": {
            "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
            "connections": []
          }
        }
      ]
    },
    {
      "kind": "SchemaMembers",
      "id": "01JK0RQN156NY7QDMBZ4P18PES",
      "fromChecksum": "0",
      "toChecksum": "2ceff5b546e77e570d9a327eebb3d9d0",
      "patch": [
        {
          "op": "replace",
          "path": "",
          "value": {
            "id": "01JK0RQN156NY7QDMBZ4P18PES",
            "defaultVariantId": "01K4ZGNVG0KMZ40BFF90VXM9GS",
            "editingVariantId": null
          }
        }
      ]
    },
    {
      "kind": "ActionPrototypeViewList",
      "id": "01K4ZGNVG0KMZ40BFF90VXM9GS",
      "fromChecksum": "0",
      "toChecksum": "64e6f593d6a71875778eb45107b94782",
      "patch": [
        {
          "op": "replace",
          "path": "",
          "value": {
            "id": "01K4ZGNVG0KMZ40BFF90VXM9GS",
            "actionPrototypes": [
              {
                "id": "01K4ZGNVGM4GEZ8GRG3HA5NHQC",
                "funcId": "01K4ZGNVFZE5FM049CCK47BDW8",
                "kind": "Create",
                "displayName": null,
                "name": "Create Asset"
              },
              {
                "id": "01K4ZGNVGM4GEZ8GRG3HA5NHQE",
                "funcId": "01K4ZGNVFYFTE00K1RZTDPS7EX",
                "kind": "Destroy",
                "displayName": null,
                "name": "Delete Asset"
              },
              {
                "id": "01K4ZGNVGNENYV6BS2WCR3Y67C",
                "funcId": "01K4ZGNVFXNM7S40WC3TCWTHHA",
                "kind": "Refresh",
                "displayName": null,
                "name": "Refresh Asset"
              },
              {
                "id": "01K4ZGNVGNENYV6BS2WCR3Y67E",
                "funcId": "01K4ZGNVFWV87D50NRY4N1S57T",
                "kind": "Update",
                "displayName": null,
                "name": "Update Asset"
              }
            ]
          }
        }
      ]
    },
    {
      "kind": "IncomingConnectionsList",
      "id": "01HRFEV0S23R1G23RP75QQDCA7",
      "fromChecksum": "ffe3c2d78d9db8ff677347a9de4b1cb6",
      "toChecksum": "6399d4337f82a08ca63b3a161eff0036",
      "patch": [
        {
          "op": "add",
          "path": "/componentConnections/11",
          "value": {
            "kind": "IncomingConnections",
            "id": "01K4ZGNVS7EQKKBA67BMG0A62S"
          }
        }
      ]
    },
    {
      "kind": "ComponentList",
      "id": "01HRFEV0S23R1G23RP75QQDCA7",
      "fromChecksum": "5b312da3ee04eb9549be0e718e3dfbfc",
      "toChecksum": "e7d1d6c884b8855a6e339ffed53e5723",
      "patch": [
        {
          "op": "add",
          "path": "/components/11",
          "value": {
            "kind": "ComponentInList",
            "id": "01K4ZGNVS7EQKKBA67BMG0A62S"
          }
        }
      ]
    },
    {
      "kind": "LuminorkDefaultVariant",
      "id": "01JK0RQN156NY7QDMBZ4P18PES",
      "fromChecksum": "0",
      "toChecksum": "5ae6a874775ad20538c451f3ef18bbda",
      "patch": [
        {
          "op": "replace",
          "path": "",
          "value": {
            "id": "01JK0RQN156NY7QDMBZ4P18PES",
            "schemaId": "01JK0RQN156NY7QDMBZ4P18PES",
            "variantId": "01K4ZGNVG0KMZ40BFF90VXM9GS",
            "displayName": "AWS::EC2::InternetGateway",
            "category": "AWS::EC2",
            "color": "#FF9900",
            "isLocked": true,
            "description": "Allocates an internet gateway for use with a VPC. After creating the Internet gateway, you then attach it to a VPC.",
            "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-internetgateway.html",
            "assetFuncId": "01K4ZGNVFZE5FM049CCK47BDW9",
            "variantFuncIds": [
              "01JYPTEC12FCY58YY754CYJ4GE",
              "01JYPTEC15AN0F0WWDXEA3JV7Y",
              "01JYPTEC4GG5YR6BVHH627ST9W",
              "01K4ZGNVFVTG0CR7Y9PJHHB1AX",
              "01K4ZGNVFVTG0CR7Y9PJHHB1B0",
              "01K4ZGNVFVTG0CR7Y9PJHHB1B3",
              "01K4ZGNVFWV87D50NRY4N1S57T",
              "01K4ZGNVFXNM7S40WC3TCWTHHA",
              "01K4ZGNVFXNM7S40WC3TCWTHHB",
              "01K4ZGNVFXNM7S40WC3TCWTHHC",
              "01K4ZGNVFYFTE00K1RZTDPS7EX",
              "01K4ZGNVFZE5FM049CCK47BDW8"
            ],
            "domainProps": {
              "propId": "01K4ZGNVG18J8WYQ38GCM31G3Q",
              "name": "domain",
              "propType": "object",
              "description": null,
              "children": [
                {
                  "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M4S",
                  "name": "Tags",
                  "propType": "array",
                  "description": "Any tags to assign to the internet gateway.",
                  "children": [
                    {
                      "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M4X",
                      "name": "TagsItem",
                      "propType": "object",
                      "description": "Specifies a tag. For more information, see [Resource tags](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-resource-tags.html).",
                      "children": [
                        {
                          "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M51",
                          "name": "Key",
                          "propType": "string",
                          "description": "The tag key.",
                          "children": null,
                          "validationFormat": "{\\"type\\":\\"string\\",\\"flags\\":{\\"presence\\":\\"required\\"},\\"rules\\":[{\\"name\\":\\"min\\",\\"args\\":{\\"limit\\":1}},{\\"name\\":\\"max\\",\\"args\\":{\\"limit\\":128}}]}",
                          "defaultValue": null,
                          "hidden": false,
                          "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-internetgateway-tag.html#cfn-ec2-internetgateway-tag-key"
                        },
                        {
                          "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M53",
                          "name": "Value",
                          "propType": "string",
                          "description": "The tag value.",
                          "children": null,
                          "validationFormat": "{\\"type\\":\\"string\\",\\"flags\\":{\\"presence\\":\\"required\\"},\\"rules\\":[{\\"name\\":\\"max\\",\\"args\\":{\\"limit\\":256}}]}",
                          "defaultValue": null,
                          "hidden": false,
                          "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-internetgateway-tag.html#cfn-ec2-internetgateway-tag-value"
                        }
                      ],
                      "validationFormat": null,
                      "defaultValue": null,
                      "hidden": false,
                      "docLink": null
                    }
                  ],
                  "validationFormat": null,
                  "defaultValue": null,
                  "hidden": false,
                  "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-internetgateway.html#cfn-ec2-internetgateway-tags"
                },
                {
                  "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M55",
                  "name": "extra",
                  "propType": "object",
                  "description": null,
                  "children": [
                    {
                      "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M59",
                      "name": "Region",
                      "propType": "string",
                      "description": null,
                      "children": null,
                      "validationFormat": null,
                      "defaultValue": null,
                      "hidden": false,
                      "docLink": null
                    },
                    {
                      "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M5B",
                      "name": "AwsPermissionsMap",
                      "propType": "string",
                      "description": null,
                      "children": null,
                      "validationFormat": null,
                      "defaultValue": "{\\"create\\":{\\"permissions\\":[\\"ec2:CreateInternetGateway\\",\\"ec2:CreateTags\\",\\"ec2:DescribeInternetGateways\\"]},\\"read\\":{\\"permissions\\":[\\"ec2:DescribeInternetGateways\\"]},\\"delete\\":{\\"permissions\\":[\\"ec2:DeleteInternetGateway\\",\\"ec2:DescribeInternetGateways\\"]},\\"update\\":{\\"permissions\\":[\\"ec2:DeleteTags\\",\\"ec2:CreateTags\\",\\"ec2:DescribeInternetGateways\\"]},\\"list\\":{\\"permissions\\":[\\"ec2:DescribeInternetGateways\\"]}}",
                      "hidden": true,
                      "docLink": null
                    },
                    {
                      "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M5D",
                      "name": "AwsResourceType",
                      "propType": "string",
                      "description": null,
                      "children": null,
                      "validationFormat": null,
                      "defaultValue": "AWS::EC2::InternetGateway",
                      "hidden": true,
                      "docLink": null
                    },
                    {
                      "propId": "01K4ZGNVGFZYRGSCWVKPWQNEMZ",
                      "name": "PropUsageMap",
                      "propType": "string",
                      "description": null,
                      "children": null,
                      "validationFormat": null,
                      "defaultValue": "{\\"createOnly\\":[],\\"updatable\\":[\\"Tags\\"],\\"secrets\\":[]}",
                      "hidden": true,
                      "docLink": null
                    }
                  ],
                  "validationFormat": null,
                  "defaultValue": null,
                  "hidden": false,
                  "docLink": null
                }
              ],
              "validationFormat": null,
              "defaultValue": null,
              "hidden": false,
              "docLink": null
            }
          }
        }
      ]
    },
    {
      "kind": "IncomingConnections",
      "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
      "fromChecksum": "0",
      "toChecksum": "37b7770cd00c8b9b116a7b80dab12f4e",
      "patch": [
        {
          "op": "replace",
          "path": "",
          "value": {
            "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
            "connections": []
          }
        }
      ]
    },
    {
      "kind": "LuminorkSchemaVariant",
      "id": "01K4ZGNVG0KMZ40BFF90VXM9GS",
      "fromChecksum": "0",
      "toChecksum": "ec33de6d1fa66392a64c3dee6a83b483",
      "patch": [
        {
          "op": "replace",
          "path": "",
          "value": {
            "id": "01K4ZGNVG0KMZ40BFF90VXM9GS",
            "variantId": "01K4ZGNVG0KMZ40BFF90VXM9GS",
            "displayName": "AWS::EC2::InternetGateway",
            "category": "AWS::EC2",
            "color": "#FF9900",
            "isLocked": true,
            "description": "Allocates an internet gateway for use with a VPC. After creating the Internet gateway, you then attach it to a VPC.",
            "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-internetgateway.html",
            "assetFuncId": "01K4ZGNVFZE5FM049CCK47BDW9",
            "variantFuncIds": [
              "01JYPTEC12FCY58YY754CYJ4GE",
              "01JYPTEC15AN0F0WWDXEA3JV7Y",
              "01JYPTEC4GG5YR6BVHH627ST9W",
              "01K4ZGNVFVTG0CR7Y9PJHHB1AX",
              "01K4ZGNVFVTG0CR7Y9PJHHB1B0",
              "01K4ZGNVFVTG0CR7Y9PJHHB1B3",
              "01K4ZGNVFWV87D50NRY4N1S57T",
              "01K4ZGNVFXNM7S40WC3TCWTHHA",
              "01K4ZGNVFXNM7S40WC3TCWTHHB",
              "01K4ZGNVFXNM7S40WC3TCWTHHC",
              "01K4ZGNVFYFTE00K1RZTDPS7EX",
              "01K4ZGNVFZE5FM049CCK47BDW8"
            ],
            "isDefaultVariant": true,
            "domainProps": {
              "propId": "01K4ZGNVG18J8WYQ38GCM31G3Q",
              "name": "domain",
              "propType": "object",
              "description": null,
              "children": [
                {
                  "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M4S",
                  "name": "Tags",
                  "propType": "array",
                  "description": "Any tags to assign to the internet gateway.",
                  "children": [
                    {
                      "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M4X",
                      "name": "TagsItem",
                      "propType": "object",
                      "description": "Specifies a tag. For more information, see [Resource tags](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-resource-tags.html).",
                      "children": [
                        {
                          "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M51",
                          "name": "Key",
                          "propType": "string",
                          "description": "The tag key.",
                          "children": null,
                          "validationFormat": "{\\"type\\":\\"string\\",\\"flags\\":{\\"presence\\":\\"required\\"},\\"rules\\":[{\\"name\\":\\"min\\",\\"args\\":{\\"limit\\":1}},{\\"name\\":\\"max\\",\\"args\\":{\\"limit\\":128}}]}",
                          "defaultValue": null,
                          "hidden": false,
                          "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-internetgateway-tag.html#cfn-ec2-internetgateway-tag-key"
                        },
                        {
                          "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M53",
                          "name": "Value",
                          "propType": "string",
                          "description": "The tag value.",
                          "children": null,
                          "validationFormat": "{\\"type\\":\\"string\\",\\"flags\\":{\\"presence\\":\\"required\\"},\\"rules\\":[{\\"name\\":\\"max\\",\\"args\\":{\\"limit\\":256}}]}",
                          "defaultValue": null,
                          "hidden": false,
                          "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-internetgateway-tag.html#cfn-ec2-internetgateway-tag-value"
                        }
                      ],
                      "validationFormat": null,
                      "defaultValue": null,
                      "hidden": false,
                      "docLink": null
                    }
                  ],
                  "validationFormat": null,
                  "defaultValue": null,
                  "hidden": false,
                  "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-internetgateway.html#cfn-ec2-internetgateway-tags"
                },
                {
                  "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M55",
                  "name": "extra",
                  "propType": "object",
                  "description": null,
                  "children": [
                    {
                      "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M59",
                      "name": "Region",
                      "propType": "string",
                      "description": null,
                      "children": null,
                      "validationFormat": null,
                      "defaultValue": null,
                      "hidden": false,
                      "docLink": null
                    },
                    {
                      "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M5B",
                      "name": "AwsPermissionsMap",
                      "propType": "string",
                      "description": null,
                      "children": null,
                      "validationFormat": null,
                      "defaultValue": "{\\"create\\":{\\"permissions\\":[\\"ec2:CreateInternetGateway\\",\\"ec2:CreateTags\\",\\"ec2:DescribeInternetGateways\\"]},\\"read\\":{\\"permissions\\":[\\"ec2:DescribeInternetGateways\\"]},\\"delete\\":{\\"permissions\\":[\\"ec2:DeleteInternetGateway\\",\\"ec2:DescribeInternetGateways\\"]},\\"update\\":{\\"permissions\\":[\\"ec2:DeleteTags\\",\\"ec2:CreateTags\\",\\"ec2:DescribeInternetGateways\\"]},\\"list\\":{\\"permissions\\":[\\"ec2:DescribeInternetGateways\\"]}}",
                      "hidden": true,
                      "docLink": null
                    },
                    {
                      "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M5D",
                      "name": "AwsResourceType",
                      "propType": "string",
                      "description": null,
                      "children": null,
                      "validationFormat": null,
                      "defaultValue": "AWS::EC2::InternetGateway",
                      "hidden": true,
                      "docLink": null
                    },
                    {
                      "propId": "01K4ZGNVGFZYRGSCWVKPWQNEMZ",
                      "name": "PropUsageMap",
                      "propType": "string",
                      "description": null,
                      "children": null,
                      "validationFormat": null,
                      "defaultValue": "{\\"createOnly\\":[],\\"updatable\\":[\\"Tags\\"],\\"secrets\\":[]}",
                      "hidden": true,
                      "docLink": null
                    }
                  ],
                  "validationFormat": null,
                  "defaultValue": null,
                  "hidden": false,
                  "docLink": null
                }
              ],
              "validationFormat": null,
              "defaultValue": null,
              "hidden": false,
              "docLink": null
            }
          }
        }
      ]
    },
    {
      "kind": "ViewComponentList",
      "id": "01JYPTEC0T4C53TF747RV5JBX8",
      "fromChecksum": "9917d744afa2039e6e5c649e1cf232e1",
      "toChecksum": "f6781d224ce9ddc7c690a8ab309bc5e8",
      "patch": [
        {
          "op": "add",
          "path": "/components/11",
          "value": {
            "kind": "ComponentInList",
            "id": "01K4ZGNVS7EQKKBA67BMG0A62S"
          }
        }
      ]
    },
    {
      "kind": "DependentValueComponentList",
      "id": "01HRFEV0S23R1G23RP75QQDCA7",
      "fromChecksum": "0b15c7cda9f4d72095a1fc086564a721",
      "toChecksum": "8be472ebf67bea9c075bd8283a2777a2",
      "patch": [
        {
          "op": "add",
          "path": "/componentIds/0",
          "value": "01K4ZGNVS7EQKKBA67BMG0A62S"
        }
      ]
    },
    {
      "kind": "SchemaVariant",
      "id": "01K4ZGNVG0KMZ40BFF90VXM9GS",
      "fromChecksum": "0",
      "toChecksum": "621f25afe1acb8fee18f8227d5c09a4c",
      "patch": [
        {
          "op": "replace",
          "path": "",
          "value": {
            "id": "01K4ZGNVG0KMZ40BFF90VXM9GS",
            "schemaId": "01JK0RQN156NY7QDMBZ4P18PES",
            "schemaName": "AWS::EC2::InternetGateway",
            "schemaVariantId": "01K4ZGNVG0KMZ40BFF90VXM9GS",
            "version": "20250912174825027359000",
            "displayName": "AWS::EC2::InternetGateway",
            "category": "AWS::EC2",
            "description": "Allocates an internet gateway for use with a VPC. After creating the Internet gateway, you then attach it to a VPC.",
            "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-internetgateway.html",
            "color": "#FF9900",
            "isLocked": true,
            "created_at": "2025-09-12T17:48:24.960670Z",
            "updated_at": "2025-09-12T17:48:24.960670Z",
            "canCreateNewComponents": true,
            "isSecretDefining": false,
            "canContribute": false,
            "mgmtFunctions": [
              {
                "id": "01K4ZGNVGQYCMVAZC3NDVKM86E",
                "funcId": "01K4ZGNVFXNM7S40WC3TCWTHHC",
                "description": null,
                "prototypeName": "Discover on AWS",
                "name": "Discover on AWS",
                "kind": "discover"
              },
              {
                "id": "01K4ZGNVGQYCMVAZC3NDVKM86G",
                "funcId": "01K4ZGNVFXNM7S40WC3TCWTHHB",
                "description": null,
                "prototypeName": "Import from AWS",
                "name": "Import from AWS",
                "kind": "import"
              }
            ],
            "propTree": {
              "props": {
                "01K4ZGNVG27VH1JPZ3S7CM9C53": {
                  "id": "01K4ZGNVG27VH1JPZ3S7CM9C53",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "format",
                  "path": "root/code/codeItem/format",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C5D": {
                  "id": "01K4ZGNVG27VH1JPZ3S7CM9C5D",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "result",
                  "path": "root/qualification/qualificationItem/result",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M53": {
                  "id": "01K4ZGNVGEHPJKWWD9ZX6T6M53",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "Value",
                  "path": "root/domain/Tags/TagsItem/Value",
                  "hidden": false,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-internetgateway-tag.html#cfn-ec2-internetgateway-tag-value",
                  "documentation": "The tag value.",
                  "validationFormat": "{\\"type\\":\\"string\\",\\"flags\\":{\\"presence\\":\\"required\\"},\\"rules\\":[{\\"name\\":\\"max\\",\\"args\\":{\\"limit\\":256}}]}",
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C4X": {
                  "id": "01K4ZGNVG27VH1JPZ3S7CM9C4X",
                  "kind": "object",
                  "childKind": "string",
                  "widgetKind": "header",
                  "name": "codeItem",
                  "path": "root/code/codeItem",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M4S": {
                  "id": "01K4ZGNVGEHPJKWWD9ZX6T6M4S",
                  "kind": "array",
                  "childKind": "object",
                  "widgetKind": "array",
                  "name": "Tags",
                  "path": "root/domain/Tags",
                  "hidden": false,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-internetgateway.html#cfn-ec2-internetgateway-tags",
                  "documentation": "Any tags to assign to the internet gateway.",
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG18J8WYQ38GCM31G3V": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G3V",
                  "kind": "object",
                  "childKind": "string",
                  "widgetKind": "header",
                  "name": "secrets",
                  "path": "root/secrets",
                  "hidden": false,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M5D": {
                  "id": "01K4ZGNVGEHPJKWWD9ZX6T6M5D",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "AwsResourceType",
                  "path": "root/domain/extra/AwsResourceType",
                  "hidden": true,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG0KMZ40BFF90VXM9GZ": {
                  "id": "01K4ZGNVG0KMZ40BFF90VXM9GZ",
                  "kind": "object",
                  "childKind": "string",
                  "widgetKind": "header",
                  "name": "si",
                  "path": "root/si",
                  "hidden": false,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M4X": {
                  "id": "01K4ZGNVGEHPJKWWD9ZX6T6M4X",
                  "kind": "object",
                  "childKind": "string",
                  "widgetKind": "header",
                  "name": "TagsItem",
                  "path": "root/domain/Tags/TagsItem",
                  "hidden": false,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": "Specifies a tag. For more information, see [Resource tags](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-resource-tags.html).",
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M5B": {
                  "id": "01K4ZGNVGEHPJKWWD9ZX6T6M5B",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "AwsPermissionsMap",
                  "path": "root/domain/extra/AwsPermissionsMap",
                  "hidden": true,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M51": {
                  "id": "01K4ZGNVGEHPJKWWD9ZX6T6M51",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "Key",
                  "path": "root/domain/Tags/TagsItem/Key",
                  "hidden": false,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-internetgateway-tag.html#cfn-ec2-internetgateway-tag-key",
                  "documentation": "The tag key.",
                  "validationFormat": "{\\"type\\":\\"string\\",\\"flags\\":{\\"presence\\":\\"required\\"},\\"rules\\":[{\\"name\\":\\"min\\",\\"args\\":{\\"limit\\":1}},{\\"name\\":\\"max\\",\\"args\\":{\\"limit\\":128}}]}",
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null,
                  "suggestSources": [
                    {
                      "prop": "/resource_value/KeyId",
                      "schema": "AWS::KMS::Key"
                    },
                    {
                      "prop": "/resource_value/KeyIdentifier",
                      "schema": "AWS::PaymentCryptography::Key"
                    }
                  ]
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C4N": {
                  "id": "01K4ZGNVG27VH1JPZ3S7CM9C4N",
                  "kind": "object",
                  "childKind": "string",
                  "widgetKind": "header",
                  "name": "resource_value",
                  "path": "root/resource_value",
                  "hidden": true,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVGFZYRGSCWVKPWQNEN1": {
                  "id": "01K4ZGNVGFZYRGSCWVKPWQNEN1",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "InternetGatewayId",
                  "path": "root/resource_value/InternetGatewayId",
                  "hidden": false,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-internetgateway.html#cfn-ec2-internetgateway-internetgatewayid",
                  "documentation": "",
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG18J8WYQ38GCM31G3H": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G3H",
                  "kind": "map",
                  "childKind": "string",
                  "widgetKind": "map",
                  "name": "tags",
                  "path": "root/si/tags",
                  "hidden": false,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M55": {
                  "id": "01K4ZGNVGEHPJKWWD9ZX6T6M55",
                  "kind": "object",
                  "childKind": "string",
                  "widgetKind": "header",
                  "name": "extra",
                  "path": "root/domain/extra",
                  "hidden": false,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG18J8WYQ38GCM31G49": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G49",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "last_synced",
                  "path": "root/resource/last_synced",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C4S": {
                  "id": "01K4ZGNVG27VH1JPZ3S7CM9C4S",
                  "kind": "map",
                  "childKind": "object",
                  "widgetKind": "map",
                  "name": "code",
                  "path": "root/code",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG18J8WYQ38GCM31G47": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G47",
                  "kind": "json",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "payload",
                  "path": "root/resource/payload",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG18J8WYQ38GCM31G3D": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G3D",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "color",
                  "name": "color",
                  "path": "root/si/color",
                  "hidden": false,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG18J8WYQ38GCM31G3B": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G3B",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": {
                    "select": {
                      "options": [
                        {
                          "label": "Component",
                          "value": "component"
                        },
                        {
                          "label": "Configuration Frame (down)",
                          "value": "configurationFrameDown"
                        },
                        {
                          "label": "Configuration Frame (up)",
                          "value": "configurationFrameUp"
                        },
                        {
                          "label": "Aggregation Frame",
                          "value": "aggregationFrame"
                        }
                      ]
                    }
                  },
                  "name": "type",
                  "path": "root/si/type",
                  "hidden": false,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C55": {
                  "id": "01K4ZGNVG27VH1JPZ3S7CM9C55",
                  "kind": "map",
                  "childKind": "object",
                  "widgetKind": "map",
                  "name": "qualification",
                  "path": "root/qualification",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG0KMZ40BFF90VXM9H3": {
                  "id": "01K4ZGNVG0KMZ40BFF90VXM9H3",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "name",
                  "path": "root/si/name",
                  "hidden": false,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVGFZYRGSCWVKPWQNEMZ": {
                  "id": "01K4ZGNVGFZYRGSCWVKPWQNEMZ",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "PropUsageMap",
                  "path": "root/domain/extra/PropUsageMap",
                  "hidden": true,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C51": {
                  "id": "01K4ZGNVG27VH1JPZ3S7CM9C51",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "code",
                  "path": "root/code/codeItem/code",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C59": {
                  "id": "01K4ZGNVG27VH1JPZ3S7CM9C59",
                  "kind": "object",
                  "childKind": "string",
                  "widgetKind": "header",
                  "name": "qualificationItem",
                  "path": "root/qualification/qualificationItem",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG18J8WYQ38GCM31G45": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G45",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "message",
                  "path": "root/resource/message",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG18J8WYQ38GCM31G3N": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G3N",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "tag",
                  "path": "root/si/tags/tag",
                  "hidden": false,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C5F": {
                  "id": "01K4ZGNVG27VH1JPZ3S7CM9C5F",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "message",
                  "path": "root/qualification/qualificationItem/message",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVGFZYRGSCWVKPWQNEN3": {
                  "id": "01K4ZGNVGFZYRGSCWVKPWQNEN3",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": {
                    "secret": {
                      "options": [
                        {
                          "label": "secretKind",
                          "value": "AWS Credential"
                        }
                      ]
                    }
                  },
                  "name": "AWS Credential",
                  "path": "root/secrets/AWS Credential",
                  "hidden": false,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null,
                  "suggestSources": [
                    {
                      "prop": "/secrets/AWS Credential",
                      "schema": "AWS Credential"
                    }
                  ]
                },
                "01K4ZGNVG18J8WYQ38GCM31G3Z": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G3Z",
                  "kind": "object",
                  "childKind": "string",
                  "widgetKind": "header",
                  "name": "resource",
                  "path": "root/resource",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG0KMZ40BFF90VXM9GV": {
                  "id": "01K4ZGNVG0KMZ40BFF90VXM9GV",
                  "kind": "object",
                  "childKind": "object",
                  "widgetKind": "header",
                  "name": "root",
                  "path": "root",
                  "hidden": false,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C5H": {
                  "id": "01K4ZGNVG27VH1JPZ3S7CM9C5H",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "deleted_at",
                  "path": "root/deleted_at",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M59": {
                  "id": "01K4ZGNVGEHPJKWWD9ZX6T6M59",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "Region",
                  "path": "root/domain/extra/Region",
                  "hidden": false,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null,
                  "suggestSources": [
                    {
                      "prop": "/domain/region",
                      "schema": "Region"
                    }
                  ]
                },
                "01K4ZGNVG18J8WYQ38GCM31G3Q": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G3Q",
                  "kind": "object",
                  "childKind": "array",
                  "widgetKind": "header",
                  "name": "domain",
                  "path": "root/domain",
                  "hidden": false,
                  "eligibleForConnection": true,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG18J8WYQ38GCM31G43": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G43",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "status",
                  "path": "root/resource/status",
                  "hidden": true,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG18J8WYQ38GCM31G39": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G39",
                  "kind": "boolean",
                  "childKind": null,
                  "widgetKind": "checkbox",
                  "name": "protected",
                  "path": "root/si/protected",
                  "hidden": false,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                },
                "01K4ZGNVG18J8WYQ38GCM31G3F": {
                  "id": "01K4ZGNVG18J8WYQ38GCM31G3F",
                  "kind": "string",
                  "childKind": null,
                  "widgetKind": "text",
                  "name": "resourceId",
                  "path": "root/si/resourceId",
                  "hidden": false,
                  "eligibleForConnection": false,
                  "createOnly": false,
                  "docLink": null,
                  "documentation": null,
                  "validationFormat": null,
                  "defaultCanBeSetBySocket": false,
                  "isOriginSecret": false,
                  "secretDefinition": null
                }
              },
              "treeInfo": {
                "01K4ZGNVG27VH1JPZ3S7CM9C4N": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GV",
                  "children": [
                    "01K4ZGNVGFZYRGSCWVKPWQNEN1"
                  ]
                },
                "01K4ZGNVGFZYRGSCWVKPWQNEN3": {
                  "parent": "01K4ZGNVG18J8WYQ38GCM31G3V",
                  "children": []
                },
                "01K4ZGNVG18J8WYQ38GCM31G3D": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GZ",
                  "children": []
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M59": {
                  "parent": "01K4ZGNVGEHPJKWWD9ZX6T6M55",
                  "children": []
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M4S": {
                  "parent": "01K4ZGNVG18J8WYQ38GCM31G3Q",
                  "children": [
                    "01K4ZGNVGEHPJKWWD9ZX6T6M4X"
                  ]
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C5F": {
                  "parent": "01K4ZGNVG27VH1JPZ3S7CM9C59",
                  "children": []
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C55": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GV",
                  "children": [
                    "01K4ZGNVG27VH1JPZ3S7CM9C59"
                  ]
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C4S": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GV",
                  "children": [
                    "01K4ZGNVG27VH1JPZ3S7CM9C4X"
                  ]
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M55": {
                  "parent": "01K4ZGNVG18J8WYQ38GCM31G3Q",
                  "children": [
                    "01K4ZGNVGEHPJKWWD9ZX6T6M59",
                    "01K4ZGNVGEHPJKWWD9ZX6T6M5B",
                    "01K4ZGNVGEHPJKWWD9ZX6T6M5D",
                    "01K4ZGNVGFZYRGSCWVKPWQNEMZ"
                  ]
                },
                "01K4ZGNVG18J8WYQ38GCM31G3Z": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GV",
                  "children": [
                    "01K4ZGNVG18J8WYQ38GCM31G43",
                    "01K4ZGNVG18J8WYQ38GCM31G45",
                    "01K4ZGNVG18J8WYQ38GCM31G47",
                    "01K4ZGNVG18J8WYQ38GCM31G49"
                  ]
                },
                "01K4ZGNVG18J8WYQ38GCM31G47": {
                  "parent": "01K4ZGNVG18J8WYQ38GCM31G3Z",
                  "children": []
                },
                "01K4ZGNVG18J8WYQ38GCM31G3Q": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GV",
                  "children": [
                    "01K4ZGNVGEHPJKWWD9ZX6T6M4S",
                    "01K4ZGNVGEHPJKWWD9ZX6T6M55"
                  ]
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M5D": {
                  "parent": "01K4ZGNVGEHPJKWWD9ZX6T6M55",
                  "children": []
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M53": {
                  "parent": "01K4ZGNVGEHPJKWWD9ZX6T6M4X",
                  "children": []
                },
                "01K4ZGNVG0KMZ40BFF90VXM9GZ": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GV",
                  "children": [
                    "01K4ZGNVG0KMZ40BFF90VXM9H3",
                    "01K4ZGNVG18J8WYQ38GCM31G39",
                    "01K4ZGNVG18J8WYQ38GCM31G3B",
                    "01K4ZGNVG18J8WYQ38GCM31G3D",
                    "01K4ZGNVG18J8WYQ38GCM31G3F",
                    "01K4ZGNVG18J8WYQ38GCM31G3H"
                  ]
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C5H": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GV",
                  "children": []
                },
                "01K4ZGNVG18J8WYQ38GCM31G43": {
                  "parent": "01K4ZGNVG18J8WYQ38GCM31G3Z",
                  "children": []
                },
                "01K4ZGNVG0KMZ40BFF90VXM9H3": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GZ",
                  "children": []
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C59": {
                  "parent": "01K4ZGNVG27VH1JPZ3S7CM9C55",
                  "children": [
                    "01K4ZGNVG27VH1JPZ3S7CM9C5D",
                    "01K4ZGNVG27VH1JPZ3S7CM9C5F"
                  ]
                },
                "01K4ZGNVG18J8WYQ38GCM31G3F": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GZ",
                  "children": []
                },
                "01K4ZGNVG18J8WYQ38GCM31G49": {
                  "parent": "01K4ZGNVG18J8WYQ38GCM31G3Z",
                  "children": []
                },
                "01K4ZGNVG18J8WYQ38GCM31G3B": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GZ",
                  "children": []
                },
                "01K4ZGNVGFZYRGSCWVKPWQNEMZ": {
                  "parent": "01K4ZGNVGEHPJKWWD9ZX6T6M55",
                  "children": []
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C51": {
                  "parent": "01K4ZGNVG27VH1JPZ3S7CM9C4X",
                  "children": []
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C53": {
                  "parent": "01K4ZGNVG27VH1JPZ3S7CM9C4X",
                  "children": []
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M4X": {
                  "parent": "01K4ZGNVGEHPJKWWD9ZX6T6M4S",
                  "children": [
                    "01K4ZGNVGEHPJKWWD9ZX6T6M51",
                    "01K4ZGNVGEHPJKWWD9ZX6T6M53"
                  ]
                },
                "01K4ZGNVG18J8WYQ38GCM31G3H": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GZ",
                  "children": [
                    "01K4ZGNVG18J8WYQ38GCM31G3N"
                  ]
                },
                "01K4ZGNVG18J8WYQ38GCM31G45": {
                  "parent": "01K4ZGNVG18J8WYQ38GCM31G3Z",
                  "children": []
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C5D": {
                  "parent": "01K4ZGNVG27VH1JPZ3S7CM9C59",
                  "children": []
                },
                "01K4ZGNVG0KMZ40BFF90VXM9GV": {
                  "parent": null,
                  "children": [
                    "01K4ZGNVG0KMZ40BFF90VXM9GZ",
                    "01K4ZGNVG18J8WYQ38GCM31G3Q",
                    "01K4ZGNVG18J8WYQ38GCM31G3V",
                    "01K4ZGNVG18J8WYQ38GCM31G3Z",
                    "01K4ZGNVG27VH1JPZ3S7CM9C4N",
                    "01K4ZGNVG27VH1JPZ3S7CM9C4S",
                    "01K4ZGNVG27VH1JPZ3S7CM9C55",
                    "01K4ZGNVG27VH1JPZ3S7CM9C5H"
                  ]
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M5B": {
                  "parent": "01K4ZGNVGEHPJKWWD9ZX6T6M55",
                  "children": []
                },
                "01K4ZGNVG18J8WYQ38GCM31G39": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GZ",
                  "children": []
                },
                "01K4ZGNVGFZYRGSCWVKPWQNEN1": {
                  "parent": "01K4ZGNVG27VH1JPZ3S7CM9C4N",
                  "children": []
                },
                "01K4ZGNVG18J8WYQ38GCM31G3N": {
                  "parent": "01K4ZGNVG18J8WYQ38GCM31G3H",
                  "children": []
                },
                "01K4ZGNVG27VH1JPZ3S7CM9C4X": {
                  "parent": "01K4ZGNVG27VH1JPZ3S7CM9C4S",
                  "children": [
                    "01K4ZGNVG27VH1JPZ3S7CM9C51",
                    "01K4ZGNVG27VH1JPZ3S7CM9C53"
                  ]
                },
                "01K4ZGNVGEHPJKWWD9ZX6T6M51": {
                  "parent": "01K4ZGNVGEHPJKWWD9ZX6T6M4X",
                  "children": []
                },
                "01K4ZGNVG18J8WYQ38GCM31G3V": {
                  "parent": "01K4ZGNVG0KMZ40BFF90VXM9GV",
                  "children": [
                    "01K4ZGNVGFZYRGSCWVKPWQNEN3"
                  ]
                }
              }
            }
          }
        }
      ]
    },
    {
      "kind": "AttributeTree",
      "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
      "fromChecksum": "0",
      "toChecksum": "0d449d6c57c6e20bb3dc0e5d06109872",
      "patch": [
        {
          "op": "replace",
          "path": "",
          "value": {
            "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
            "attributeValues": {
              "01K4ZGNVSB6CYYBD02FDKS4N1S": {
                "id": "01K4ZGNVSB6CYYBD02FDKS4N1S",
                "key": "awsCloudControlCreate",
                "path": "/code/awsCloudControlCreate",
                "propId": "01K4ZGNVG27VH1JPZ3S7CM9C4X",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": true,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSADAVR26W16AJFBFBK": {
                "id": "01K4ZGNVSADAVR26W16AJFBFBK",
                "key": null,
                "path": "/resource/message",
                "propId": "01K4ZGNVG18J8WYQ38GCM31G45",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSC06KVJ8ZKT7KBPDVT": {
                "id": "01K4ZGNVSC06KVJ8ZKT7KBPDVT",
                "key": null,
                "path": "/domain/extra/PropUsageMap",
                "propId": "01K4ZGNVGFZYRGSCWVKPWQNEMZ",
                "value": "{\\"createOnly\\":[],\\"updatable\\":[\\"Tags\\"],\\"secrets\\":[]}",
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS9ESZY787DMHBG4RV2": {
                "id": "01K4ZGNVS9ESZY787DMHBG4RV2",
                "key": null,
                "path": "/si/resourceId",
                "propId": "01K4ZGNVG18J8WYQ38GCM31G3F",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSB6CYYBD02FDKS4N1H": {
                "id": "01K4ZGNVSB6CYYBD02FDKS4N1H",
                "key": "awsCloudFormationLint",
                "path": "/code/awsCloudFormationLint",
                "propId": "01K4ZGNVG27VH1JPZ3S7CM9C4X",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": true,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS7EQKKBA67BMG0A62V": {
                "id": "01K4ZGNVS7EQKKBA67BMG0A62V",
                "key": null,
                "path": "",
                "propId": "01K4ZGNVG0KMZ40BFF90VXM9GV",
                "value": {},
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": true,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSB6CYYBD02FDKS4N1D": {
                "id": "01K4ZGNVSB6CYYBD02FDKS4N1D",
                "key": null,
                "path": "/resource_value/InternetGatewayId",
                "propId": "01K4ZGNVGFZYRGSCWVKPWQNEN1",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSADAVR26W16AJFBFBB": {
                "id": "01K4ZGNVSADAVR26W16AJFBFBB",
                "key": null,
                "path": "/secrets/AWS Credential",
                "propId": "01K4ZGNVGFZYRGSCWVKPWQNEN3",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSADAVR26W16AJFBFBQ": {
                "id": "01K4ZGNVSADAVR26W16AJFBFBQ",
                "key": null,
                "path": "/resource/payload",
                "propId": "01K4ZGNVG18J8WYQ38GCM31G47",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSB6CYYBD02FDKS4N21": {
                "id": "01K4ZGNVSB6CYYBD02FDKS4N21",
                "key": null,
                "path": "/domain/extra/AwsPermissionsMap",
                "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M5B",
                "value": "{\\"create\\":{\\"permissions\\":[\\"ec2:CreateInternetGateway\\",\\"ec2:CreateTags\\",\\"ec2:DescribeInternetGateways\\"]},\\"read\\":{\\"permissions\\":[\\"ec2:DescribeInternetGateways\\"]},\\"delete\\":{\\"permissions\\":[\\"ec2:DeleteInternetGateway\\",\\"ec2:DescribeInternetGateways\\"]},\\"update\\":{\\"permissions\\":[\\"ec2:DeleteTags\\",\\"ec2:CreateTags\\",\\"ec2:DescribeInternetGateways\\"]},\\"list\\":{\\"permissions\\":[\\"ec2:DescribeInternetGateways\\"]}}",
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS8ACE00QV8M9C08705": {
                "id": "01K4ZGNVS8ACE00QV8M9C08705",
                "key": null,
                "path": "/deleted_at",
                "propId": "01K4ZGNVG27VH1JPZ3S7CM9C5H",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS8ACE00QV8M9C08701": {
                "id": "01K4ZGNVS8ACE00QV8M9C08701",
                "key": null,
                "path": "/qualification",
                "propId": "01K4ZGNVG27VH1JPZ3S7CM9C55",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS7EQKKBA67BMG0A637": {
                "id": "01K4ZGNVS7EQKKBA67BMG0A637",
                "key": null,
                "path": "/secrets",
                "propId": "01K4ZGNVG18J8WYQ38GCM31G3V",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSADAVR26W16AJFBFB7": {
                "id": "01K4ZGNVSADAVR26W16AJFBFB7",
                "key": null,
                "path": "/domain/extra",
                "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M55",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSC06KVJ8ZKT7KBPDVP": {
                "id": "01K4ZGNVSC06KVJ8ZKT7KBPDVP",
                "key": null,
                "path": "/domain/extra/AwsResourceType",
                "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M5D",
                "value": "AWS::EC2::InternetGateway",
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS9ESZY787DMHBG4RTY": {
                "id": "01K4ZGNVS9ESZY787DMHBG4RTY",
                "key": null,
                "path": "/si/color",
                "propId": "01K4ZGNVG18J8WYQ38GCM31G3D",
                "value": "#FF9900",
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS9ESZY787DMHBG4RTJ": {
                "id": "01K4ZGNVS9ESZY787DMHBG4RTJ",
                "key": null,
                "path": "/si/name",
                "propId": "01K4ZGNVG0KMZ40BFF90VXM9H3",
                "value": "si-6666",
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": true,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS7EQKKBA67BMG0A633": {
                "id": "01K4ZGNVS7EQKKBA67BMG0A633",
                "key": null,
                "path": "/domain",
                "propId": "01K4ZGNVG18J8WYQ38GCM31G3Q",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS8ACE00QV8M9C086ZN": {
                "id": "01K4ZGNVS8ACE00QV8M9C086ZN",
                "key": null,
                "path": "/resource",
                "propId": "01K4ZGNVG18J8WYQ38GCM31G3Z",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSADAVR26W16AJFBFB3": {
                "id": "01K4ZGNVSADAVR26W16AJFBFB3",
                "key": null,
                "path": "/domain/Tags",
                "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M4S",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS8ACE00QV8M9C086ZX": {
                "id": "01K4ZGNVS8ACE00QV8M9C086ZX",
                "key": null,
                "path": "/code",
                "propId": "01K4ZGNVG27VH1JPZ3S7CM9C4S",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS9ESZY787DMHBG4RV6": {
                "id": "01K4ZGNVS9ESZY787DMHBG4RV6",
                "key": null,
                "path": "/si/tags",
                "propId": "01K4ZGNVG18J8WYQ38GCM31G3H",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSADAVR26W16AJFBFBV": {
                "id": "01K4ZGNVSADAVR26W16AJFBFBV",
                "key": null,
                "path": "/resource/last_synced",
                "propId": "01K4ZGNVG18J8WYQ38GCM31G49",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSB6CYYBD02FDKS4N1N": {
                "id": "01K4ZGNVSB6CYYBD02FDKS4N1N",
                "key": "awsCloudControlUpdate",
                "path": "/code/awsCloudControlUpdate",
                "propId": "01K4ZGNVG27VH1JPZ3S7CM9C4X",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": true,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS7EQKKBA67BMG0A62Z": {
                "id": "01K4ZGNVS7EQKKBA67BMG0A62Z",
                "key": null,
                "path": "/si",
                "propId": "01K4ZGNVG0KMZ40BFF90VXM9GZ",
                "value": {},
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": true,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSB6CYYBD02FDKS4N1X": {
                "id": "01K4ZGNVSB6CYYBD02FDKS4N1X",
                "key": null,
                "path": "/domain/extra/Region",
                "propId": "01K4ZGNVGEHPJKWWD9ZX6T6M59",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVSADAVR26W16AJFBFBF": {
                "id": "01K4ZGNVSADAVR26W16AJFBFBF",
                "key": null,
                "path": "/resource/status",
                "propId": "01K4ZGNVG18J8WYQ38GCM31G43",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS8ACE00QV8M9C086ZS": {
                "id": "01K4ZGNVS8ACE00QV8M9C086ZS",
                "key": null,
                "path": "/resource_value",
                "propId": "01K4ZGNVG27VH1JPZ3S7CM9C4N",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": true,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS9ESZY787DMHBG4RTT": {
                "id": "01K4ZGNVS9ESZY787DMHBG4RTT",
                "key": null,
                "path": "/si/type",
                "propId": "01K4ZGNVG18J8WYQ38GCM31G3B",
                "value": "component",
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": true,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              },
              "01K4ZGNVS9ESZY787DMHBG4RTP": {
                "id": "01K4ZGNVS9ESZY787DMHBG4RTP",
                "key": null,
                "path": "/si/protected",
                "propId": "01K4ZGNVG18J8WYQ38GCM31G39",
                "value": null,
                "externalSources": null,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": false,
                "validation": null,
                "secret": null,
                "hasSocketConnection": false,
                "isDefaultSource": false
              }
            },
            "props": {
              "01K4ZGNVGFZYRGSCWVKPWQNEN1": {
                "id": "01K4ZGNVGFZYRGSCWVKPWQNEN1",
                "kind": "string",
                "childKind": null,
                "widgetKind": "text",
                "name": "InternetGatewayId",
                "path": "root/resource_value/InternetGatewayId",
                "hidden": false,
                "eligibleForConnection": true,
                "createOnly": false,
                "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-internetgateway.html#cfn-ec2-internetgateway-internetgatewayid",
                "documentation": "",
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVGEHPJKWWD9ZX6T6M55": {
                "id": "01K4ZGNVGEHPJKWWD9ZX6T6M55",
                "kind": "object",
                "childKind": "string",
                "widgetKind": "header",
                "name": "extra",
                "path": "root/domain/extra",
                "hidden": false,
                "eligibleForConnection": true,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG18J8WYQ38GCM31G45": {
                "id": "01K4ZGNVG18J8WYQ38GCM31G45",
                "kind": "string",
                "childKind": null,
                "widgetKind": "text",
                "name": "message",
                "path": "root/resource/message",
                "hidden": true,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVGEHPJKWWD9ZX6T6M5D": {
                "id": "01K4ZGNVGEHPJKWWD9ZX6T6M5D",
                "kind": "string",
                "childKind": null,
                "widgetKind": "text",
                "name": "AwsResourceType",
                "path": "root/domain/extra/AwsResourceType",
                "hidden": true,
                "eligibleForConnection": true,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG0KMZ40BFF90VXM9GV": {
                "id": "01K4ZGNVG0KMZ40BFF90VXM9GV",
                "kind": "object",
                "childKind": "object",
                "widgetKind": "header",
                "name": "root",
                "path": "root",
                "hidden": false,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG18J8WYQ38GCM31G39": {
                "id": "01K4ZGNVG18J8WYQ38GCM31G39",
                "kind": "boolean",
                "childKind": null,
                "widgetKind": "checkbox",
                "name": "protected",
                "path": "root/si/protected",
                "hidden": false,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG27VH1JPZ3S7CM9C5H": {
                "id": "01K4ZGNVG27VH1JPZ3S7CM9C5H",
                "kind": "string",
                "childKind": null,
                "widgetKind": "text",
                "name": "deleted_at",
                "path": "root/deleted_at",
                "hidden": true,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG18J8WYQ38GCM31G3Q": {
                "id": "01K4ZGNVG18J8WYQ38GCM31G3Q",
                "kind": "object",
                "childKind": "array",
                "widgetKind": "header",
                "name": "domain",
                "path": "root/domain",
                "hidden": false,
                "eligibleForConnection": true,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG18J8WYQ38GCM31G3Z": {
                "id": "01K4ZGNVG18J8WYQ38GCM31G3Z",
                "kind": "object",
                "childKind": "string",
                "widgetKind": "header",
                "name": "resource",
                "path": "root/resource",
                "hidden": true,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG18J8WYQ38GCM31G3D": {
                "id": "01K4ZGNVG18J8WYQ38GCM31G3D",
                "kind": "string",
                "childKind": null,
                "widgetKind": "color",
                "name": "color",
                "path": "root/si/color",
                "hidden": false,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG27VH1JPZ3S7CM9C55": {
                "id": "01K4ZGNVG27VH1JPZ3S7CM9C55",
                "kind": "map",
                "childKind": "object",
                "widgetKind": "map",
                "name": "qualification",
                "path": "root/qualification",
                "hidden": true,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVGFZYRGSCWVKPWQNEN3": {
                "id": "01K4ZGNVGFZYRGSCWVKPWQNEN3",
                "kind": "string",
                "childKind": null,
                "widgetKind": {
                  "secret": {
                    "options": [
                      {
                        "label": "secretKind",
                        "value": "AWS Credential"
                      }
                    ]
                  }
                },
                "name": "AWS Credential",
                "path": "root/secrets/AWS Credential",
                "hidden": false,
                "eligibleForConnection": true,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null,
                "suggestSources": [
                  {
                    "prop": "/secrets/AWS Credential",
                    "schema": "AWS Credential"
                  }
                ]
              },
              "01K4ZGNVG27VH1JPZ3S7CM9C4X": {
                "id": "01K4ZGNVG27VH1JPZ3S7CM9C4X",
                "kind": "object",
                "childKind": "string",
                "widgetKind": "header",
                "name": "codeItem",
                "path": "root/code/codeItem",
                "hidden": true,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG18J8WYQ38GCM31G47": {
                "id": "01K4ZGNVG18J8WYQ38GCM31G47",
                "kind": "json",
                "childKind": null,
                "widgetKind": "text",
                "name": "payload",
                "path": "root/resource/payload",
                "hidden": true,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG18J8WYQ38GCM31G3V": {
                "id": "01K4ZGNVG18J8WYQ38GCM31G3V",
                "kind": "object",
                "childKind": "string",
                "widgetKind": "header",
                "name": "secrets",
                "path": "root/secrets",
                "hidden": false,
                "eligibleForConnection": true,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG27VH1JPZ3S7CM9C4N": {
                "id": "01K4ZGNVG27VH1JPZ3S7CM9C4N",
                "kind": "object",
                "childKind": "string",
                "widgetKind": "header",
                "name": "resource_value",
                "path": "root/resource_value",
                "hidden": true,
                "eligibleForConnection": true,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG18J8WYQ38GCM31G3H": {
                "id": "01K4ZGNVG18J8WYQ38GCM31G3H",
                "kind": "map",
                "childKind": "string",
                "widgetKind": "map",
                "name": "tags",
                "path": "root/si/tags",
                "hidden": false,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVGEHPJKWWD9ZX6T6M59": {
                "id": "01K4ZGNVGEHPJKWWD9ZX6T6M59",
                "kind": "string",
                "childKind": null,
                "widgetKind": "text",
                "name": "Region",
                "path": "root/domain/extra/Region",
                "hidden": false,
                "eligibleForConnection": true,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null,
                "suggestSources": [
                  {
                    "prop": "/domain/region",
                    "schema": "Region"
                  }
                ]
              },
              "01K4ZGNVGEHPJKWWD9ZX6T6M5B": {
                "id": "01K4ZGNVGEHPJKWWD9ZX6T6M5B",
                "kind": "string",
                "childKind": null,
                "widgetKind": "text",
                "name": "AwsPermissionsMap",
                "path": "root/domain/extra/AwsPermissionsMap",
                "hidden": true,
                "eligibleForConnection": true,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG0KMZ40BFF90VXM9H3": {
                "id": "01K4ZGNVG0KMZ40BFF90VXM9H3",
                "kind": "string",
                "childKind": null,
                "widgetKind": "text",
                "name": "name",
                "path": "root/si/name",
                "hidden": false,
                "eligibleForConnection": true,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVGFZYRGSCWVKPWQNEMZ": {
                "id": "01K4ZGNVGFZYRGSCWVKPWQNEMZ",
                "kind": "string",
                "childKind": null,
                "widgetKind": "text",
                "name": "PropUsageMap",
                "path": "root/domain/extra/PropUsageMap",
                "hidden": true,
                "eligibleForConnection": true,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG27VH1JPZ3S7CM9C4S": {
                "id": "01K4ZGNVG27VH1JPZ3S7CM9C4S",
                "kind": "map",
                "childKind": "object",
                "widgetKind": "map",
                "name": "code",
                "path": "root/code",
                "hidden": true,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG0KMZ40BFF90VXM9GZ": {
                "id": "01K4ZGNVG0KMZ40BFF90VXM9GZ",
                "kind": "object",
                "childKind": "string",
                "widgetKind": "header",
                "name": "si",
                "path": "root/si",
                "hidden": false,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG18J8WYQ38GCM31G3F": {
                "id": "01K4ZGNVG18J8WYQ38GCM31G3F",
                "kind": "string",
                "childKind": null,
                "widgetKind": "text",
                "name": "resourceId",
                "path": "root/si/resourceId",
                "hidden": false,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG18J8WYQ38GCM31G3B": {
                "id": "01K4ZGNVG18J8WYQ38GCM31G3B",
                "kind": "string",
                "childKind": null,
                "widgetKind": {
                  "select": {
                    "options": [
                      {
                        "label": "Component",
                        "value": "component"
                      },
                      {
                        "label": "Configuration Frame (down)",
                        "value": "configurationFrameDown"
                      },
                      {
                        "label": "Configuration Frame (up)",
                        "value": "configurationFrameUp"
                      },
                      {
                        "label": "Aggregation Frame",
                        "value": "aggregationFrame"
                      }
                    ]
                  }
                },
                "name": "type",
                "path": "root/si/type",
                "hidden": false,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVGEHPJKWWD9ZX6T6M4S": {
                "id": "01K4ZGNVGEHPJKWWD9ZX6T6M4S",
                "kind": "array",
                "childKind": "object",
                "widgetKind": "array",
                "name": "Tags",
                "path": "root/domain/Tags",
                "hidden": false,
                "eligibleForConnection": true,
                "createOnly": false,
                "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-internetgateway.html#cfn-ec2-internetgateway-tags",
                "documentation": "Any tags to assign to the internet gateway.",
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG18J8WYQ38GCM31G43": {
                "id": "01K4ZGNVG18J8WYQ38GCM31G43",
                "kind": "string",
                "childKind": null,
                "widgetKind": "text",
                "name": "status",
                "path": "root/resource/status",
                "hidden": true,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              },
              "01K4ZGNVG18J8WYQ38GCM31G49": {
                "id": "01K4ZGNVG18J8WYQ38GCM31G49",
                "kind": "string",
                "childKind": null,
                "widgetKind": "text",
                "name": "last_synced",
                "path": "root/resource/last_synced",
                "hidden": true,
                "eligibleForConnection": false,
                "createOnly": false,
                "docLink": null,
                "documentation": null,
                "validationFormat": null,
                "defaultCanBeSetBySocket": false,
                "isOriginSecret": false,
                "secretDefinition": null
              }
            },
            "treeInfo": {
              "01K4ZGNVS9ESZY787DMHBG4RTT": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62Z",
                "children": []
              },
              "01K4ZGNVSC06KVJ8ZKT7KBPDVP": {
                "parent": "01K4ZGNVSADAVR26W16AJFBFB7",
                "children": []
              },
              "01K4ZGNVS9ESZY787DMHBG4RTY": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62Z",
                "children": []
              },
              "01K4ZGNVS8ACE00QV8M9C086ZX": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62V",
                "children": [
                  "01K4ZGNVSB6CYYBD02FDKS4N1H",
                  "01K4ZGNVSB6CYYBD02FDKS4N1N",
                  "01K4ZGNVSB6CYYBD02FDKS4N1S"
                ]
              },
              "01K4ZGNVS7EQKKBA67BMG0A633": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62V",
                "children": [
                  "01K4ZGNVSADAVR26W16AJFBFB3",
                  "01K4ZGNVSADAVR26W16AJFBFB7"
                ]
              },
              "01K4ZGNVSADAVR26W16AJFBFBQ": {
                "parent": "01K4ZGNVS8ACE00QV8M9C086ZN",
                "children": []
              },
              "01K4ZGNVSC06KVJ8ZKT7KBPDVT": {
                "parent": "01K4ZGNVSADAVR26W16AJFBFB7",
                "children": []
              },
              "01K4ZGNVS8ACE00QV8M9C08705": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62V",
                "children": []
              },
              "01K4ZGNVSADAVR26W16AJFBFB3": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A633",
                "children": []
              },
              "01K4ZGNVSB6CYYBD02FDKS4N21": {
                "parent": "01K4ZGNVSADAVR26W16AJFBFB7",
                "children": []
              },
              "01K4ZGNVSB6CYYBD02FDKS4N1S": {
                "parent": "01K4ZGNVS8ACE00QV8M9C086ZX",
                "children": []
              },
              "01K4ZGNVS9ESZY787DMHBG4RV6": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62Z",
                "children": []
              },
              "01K4ZGNVS9ESZY787DMHBG4RTJ": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62Z",
                "children": []
              },
              "01K4ZGNVSADAVR26W16AJFBFBK": {
                "parent": "01K4ZGNVS8ACE00QV8M9C086ZN",
                "children": []
              },
              "01K4ZGNVSB6CYYBD02FDKS4N1N": {
                "parent": "01K4ZGNVS8ACE00QV8M9C086ZX",
                "children": []
              },
              "01K4ZGNVSB6CYYBD02FDKS4N1X": {
                "parent": "01K4ZGNVSADAVR26W16AJFBFB7",
                "children": []
              },
              "01K4ZGNVSADAVR26W16AJFBFBF": {
                "parent": "01K4ZGNVS8ACE00QV8M9C086ZN",
                "children": []
              },
              "01K4ZGNVS7EQKKBA67BMG0A62Z": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62V",
                "children": [
                  "01K4ZGNVS9ESZY787DMHBG4RTJ",
                  "01K4ZGNVS9ESZY787DMHBG4RTP",
                  "01K4ZGNVS9ESZY787DMHBG4RTT",
                  "01K4ZGNVS9ESZY787DMHBG4RTY",
                  "01K4ZGNVS9ESZY787DMHBG4RV2",
                  "01K4ZGNVS9ESZY787DMHBG4RV6"
                ]
              },
              "01K4ZGNVS8ACE00QV8M9C08701": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62V",
                "children": []
              },
              "01K4ZGNVSB6CYYBD02FDKS4N1D": {
                "parent": "01K4ZGNVS8ACE00QV8M9C086ZS",
                "children": []
              },
              "01K4ZGNVSADAVR26W16AJFBFBB": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A637",
                "children": []
              },
              "01K4ZGNVSADAVR26W16AJFBFB7": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A633",
                "children": [
                  "01K4ZGNVSB6CYYBD02FDKS4N1X",
                  "01K4ZGNVSB6CYYBD02FDKS4N21",
                  "01K4ZGNVSC06KVJ8ZKT7KBPDVP",
                  "01K4ZGNVSC06KVJ8ZKT7KBPDVT"
                ]
              },
              "01K4ZGNVSADAVR26W16AJFBFBV": {
                "parent": "01K4ZGNVS8ACE00QV8M9C086ZN",
                "children": []
              },
              "01K4ZGNVS9ESZY787DMHBG4RTP": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62Z",
                "children": []
              },
              "01K4ZGNVSB6CYYBD02FDKS4N1H": {
                "parent": "01K4ZGNVS8ACE00QV8M9C086ZX",
                "children": []
              },
              "01K4ZGNVS9ESZY787DMHBG4RV2": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62Z",
                "children": []
              },
              "01K4ZGNVS8ACE00QV8M9C086ZN": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62V",
                "children": [
                  "01K4ZGNVSADAVR26W16AJFBFBF",
                  "01K4ZGNVSADAVR26W16AJFBFBK",
                  "01K4ZGNVSADAVR26W16AJFBFBQ",
                  "01K4ZGNVSADAVR26W16AJFBFBV"
                ]
              },
              "01K4ZGNVS7EQKKBA67BMG0A62V": {
                "parent": null,
                "children": [
                  "01K4ZGNVS7EQKKBA67BMG0A62Z",
                  "01K4ZGNVS7EQKKBA67BMG0A633",
                  "01K4ZGNVS7EQKKBA67BMG0A637",
                  "01K4ZGNVS8ACE00QV8M9C086ZN",
                  "01K4ZGNVS8ACE00QV8M9C086ZS",
                  "01K4ZGNVS8ACE00QV8M9C086ZX",
                  "01K4ZGNVS8ACE00QV8M9C08701",
                  "01K4ZGNVS8ACE00QV8M9C08705"
                ]
              },
              "01K4ZGNVS7EQKKBA67BMG0A637": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62V",
                "children": [
                  "01K4ZGNVSADAVR26W16AJFBFBB"
                ]
              },
              "01K4ZGNVS8ACE00QV8M9C086ZS": {
                "parent": "01K4ZGNVS7EQKKBA67BMG0A62V",
                "children": [
                  "01K4ZGNVSB6CYYBD02FDKS4N1D"
                ]
              }
            },
            "componentName": "si-6666",
            "schemaName": "AWS::EC2::InternetGateway"
          }
        }
      ]
    },
    {
      "kind": "ActionViewList",
      "id": "01HRFEV0S23R1G23RP75QQDCA7",
      "fromChecksum": "d9d8534e9357c3ec74dafec3a3356964",
      "toChecksum": "844f283b6b7c1e831b48f58c433e4393",
      "patch": [
        {
          "op": "replace",
          "path": "/actions/4/id",
          "value": "01K4ZGNVST1SXCPYZB5P253PMG"
        },
        {
          "op": "replace",
          "path": "/actions/4/prototypeId",
          "value": "01K4ZGNVGM4GEZ8GRG3HA5NHQC"
        },
        {
          "op": "replace",
          "path": "/actions/4/componentId",
          "value": "01K4ZGNVS7EQKKBA67BMG0A62S"
        },
        {
          "op": "replace",
          "path": "/actions/4/componentSchemaName",
          "value": "AWS::EC2::InternetGateway"
        },
        {
          "op": "replace",
          "path": "/actions/4/componentName",
          "value": "si-6666"
        },
        {
          "op": "replace",
          "path": "/actions/4/originatingChangeSetId",
          "value": "01K4ZF6QXKB3ZV3124ER2C0TFT"
        },
        {
          "op": "remove",
          "path": "/actions/4/dependentOn/0"
        },
        {
          "op": "remove",
          "path": "/actions/4/holdStatusInfluencedBy/0"
        },
        {
          "op": "replace",
          "path": "/actions/5/id",
          "value": "01K1GRZHQ6SC3V1CJVJ9KAE8ZV"
        },
        {
          "op": "replace",
          "path": "/actions/5/prototypeId",
          "value": "01K1GRZFAGEEAPNWZHJJ342MV5"
        },
        {
          "op": "replace",
          "path": "/actions/5/componentId",
          "value": "01K1GRZHEKA8EK4RVC8X2M8N6Y"
        },
        {
          "op": "replace",
          "path": "/actions/5/componentSchemaName",
          "value": "AWS::EC2::Subnet"
        },
        {
          "op": "replace",
          "path": "/actions/5/componentName",
          "value": "si-5540"
        },
        {
          "op": "replace",
          "path": "/actions/6/id",
          "value": "01K1GS00V3KZPNYMQ1YP2RHMBE"
        },
        {
          "op": "replace",
          "path": "/actions/6/prototypeId",
          "value": "01K1GRZYEWVSPDR6DVE5GN8S71"
        },
        {
          "op": "replace",
          "path": "/actions/6/componentId",
          "value": "01K1GS00KRGAP3VAAMTN9NQNZR"
        },
        {
          "op": "replace",
          "path": "/actions/6/componentSchemaName",
          "value": "AWS::EC2::RouteTable"
        },
        {
          "op": "replace",
          "path": "/actions/6/componentName",
          "value": "si-2742"
        },
        {
          "op": "add",
          "path": "/actions/7",
          "value": {
            "id": "01K1GS64D84J63XVX4M26D45X9",
            "prototypeId": "01K1GRZFAGEEAPNWZHJJ342MV5",
            "componentId": "01K1GS643R30F0N4FPBBM19W5Z",
            "componentSchemaName": "AWS::EC2::Subnet",
            "componentName": "si-3014",
            "name": "Create",
            "description": null,
            "kind": "Create",
            "state": "Queued",
            "originatingChangeSetId": "01K1GRZ32BEA8T9VB2W4115NCH",
            "myDependencies": [],
            "dependentOn": [
              "01JZK1VKN3BB7SZR6CETPQEHN1"
            ],
            "holdStatusInfluencedBy": [
              "01JZK1VKN3BB7SZR6CETPQEHN1"
            ]
          }
        }
      ]
    },
    {
      "kind": "Component",
      "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
      "fromChecksum": "0",
      "toChecksum": "feb2c727b8082cb73a3fe9906788799b",
      "patch": [
        {
          "op": "replace",
          "path": "",
          "value": {
            "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
            "name": "si-6666",
            "color": "#FF9900",
            "schemaName": "AWS::EC2::InternetGateway",
            "schemaId": "01JK0RQN156NY7QDMBZ4P18PES",
            "schemaVariantId": {
              "kind": "SchemaVariant",
              "id": "01K4ZGNVG0KMZ40BFF90VXM9GS"
            },
            "schemaMembers": {
              "kind": "SchemaMembers",
              "id": "01JK0RQN156NY7QDMBZ4P18PES"
            },
            "schemaVariantName": "AWS::EC2::InternetGateway",
            "schemaVariantDescription": "Allocates an internet gateway for use with a VPC. After creating the Internet gateway, you then attach it to a VPC.",
            "schemaVariantDocLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-internetgateway.html",
            "schemaCategory": "AWS::EC2",
            "hasResource": false,
            "qualificationTotals": {
              "total": 0,
              "warned": 0,
              "succeeded": 0,
              "failed": 0
            },
            "inputCount": 0,
            "resourceDiff": {
              "current": "{  \\"si\\": {    \\"name\\": \\"si-6666\\",    \\"type\\": \\"component\\",    \\"color\\": \\"#FF9900\\"  }}",
              "diff": "+{+  \\"si\\": {+    \\"name\\": \\"si-6666\\",+    \\"type\\": \\"component\\",+    \\"color\\": \\"#FF9900\\"+  }+}"
            },
            "isSecretDefining": false,
            "toDelete": false
          }
        }
      ]
    },
    {
      "kind": "ComponentInList",
      "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
      "fromChecksum": "0",
      "toChecksum": "f7ce92fc3f94fe04dee944d2cdea7720",
      "patch": [
        {
          "op": "replace",
          "path": "",
          "value": {
            "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
            "name": "si-6666",
            "color": "#FF9900",
            "schemaName": "AWS::EC2::InternetGateway",
            "schemaId": "01JK0RQN156NY7QDMBZ4P18PES",
            "schemaVariantId": "01K4ZGNVG0KMZ40BFF90VXM9GS",
            "schemaVariantName": "AWS::EC2::InternetGateway",
            "schemaCategory": "AWS::EC2",
            "hasResource": false,
            "resourceId": null,
            "qualificationTotals": {
              "total": 0,
              "warned": 0,
              "succeeded": 0,
              "failed": 0
            },
            "inputCount": 0,
            "diffStatus": "Added",
            "toDelete": false,
            "hasSocketConnections": false
          }
        }
      ]
    },
    {
      "kind": "ComponentDiff",
      "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
      "fromChecksum": "0",
      "toChecksum": "8aa8765552b1dfb5c84ae0e7ff2b885a",
      "patch": [
        {
          "op": "replace",
          "path": "",
          "value": {
            "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
            "diffStatus": "Added",
            "attributeDiffs": {
              "": {
                "new": {
                  "$source": {
                    "value": {}
                  },
                  "$value": {
                    "si": {
                      "name": "si-6666",
                      "type": "component",
                      "color": "#FF9900"
                    }
                  }
                }
              },
              "/si": {
                "new": {
                  "$source": {
                    "value": {}
                  },
                  "$value": {
                    "name": "si-6666",
                    "type": "component",
                    "color": "#FF9900"
                  }
                }
              },
              "/si/name": {
                "new": {
                  "$source": {
                    "value": "si-6666"
                  },
                  "$value": "si-6666"
                }
              },
              "/si/protected": {
                "new": {
                  "$source": {
                    "prototype": "si:unset()",
                    "fromSchema": true
                  }
                }
              },
              "/si/type": {
                "new": {
                  "$source": {
                    "value": "component"
                  },
                  "$value": "component"
                }
              },
              "/si/color": {
                "new": {
                  "$source": {
                    "value": "#FF9900",
                    "fromSchema": true
                  },
                  "$value": "#FF9900"
                }
              },
              "/si/resourceId": {
                "new": {
                  "$source": {
                    "prototype": "si:unset()",
                    "fromSchema": true
                  }
                }
              },
              "/si/tags": {
                "new": {
                  "$source": {
                    "prototype": "si:unset()",
                    "fromSchema": true
                  }
                }
              },
              "/domain": {
                "new": {
                  "$source": {
                    "prototype": "si:unset()",
                    "fromSchema": true
                  }
                }
              },
              "/domain/Tags": {
                "new": {
                  "$source": {
                    "prototype": "si:unset()",
                    "fromSchema": true
                  }
                }
              },
              "/domain/extra": {
                "new": {
                  "$source": {
                    "prototype": "si:unset()",
                    "fromSchema": true
                  }
                }
              },
              "/domain/extra/Region": {
                "new": {
                  "$source": {
                    "prototype": "si:unset()",
                    "fromSchema": true
                  }
                }
              },
              "/secrets": {
                "new": {
                  "$source": {
                    "prototype": "si:unset()",
                    "fromSchema": true
                  }
                }
              },
              "/secrets/AWS Credential": {
                "new": {
                  "$source": {
                    "prototype": "si:unset()",
                    "fromSchema": true
                  }
                }
              }
            },
            "resourceDiff": {
              "current": "{  \\"si\\": {    \\"name\\": \\"si-6666\\",    \\"type\\": \\"component\\",    \\"color\\": \\"#FF9900\\"  }}",
              "diff": "+{+  \\"si\\": {+    \\"name\\": \\"si-6666\\",+    \\"type\\": \\"component\\",+    \\"color\\": \\"#FF9900\\"+  }+}"
            }
          }
        }
      ]
    },
    {
      "kind": "ActionDiffList",
      "id": "01HRFEV0S23R1G23RP75QQDCA7",
      "fromChecksum": "0f4f01ff311d5d65e3d4be2ffac33172",
      "toChecksum": "da9dd52f0459df647b28a39223a6e915",
      "patch": [
        {
          "op": "add",
          "path": "/actionDiffs/01K4ZGNVST1SXCPYZB5P253PMG",
          "value": {
            "id": "01K4ZGNVST1SXCPYZB5P253PMG",
            "diffStatus": {
              "Added": {
                "new_state": "Queued"
              }
            },
            "componentId": "01K4ZGNVS7EQKKBA67BMG0A62S"
          }
        }
      ]
    }
  ]
}
  `) as WorkspacePatchBatch;
  await db.handleWorkspacePatchMessage(addPatch);
  const componentId = "01K4ZGNVS7EQKKBA67BMG0A62S";
  const addedC = (await db.get(
    workspaceId,
    addChangeSetId,
    EntityKind.Component,
    componentId,
  )) as BifrostComponent | -1;
  assert(addedC !== -1, "added component doesn't exit");
  if (addedC === -1) return;
  assert(addedC.name === "si-6666", "name mismatch");
  const newList = JSON.parse(
    await db.getList(
      workspaceId,
      addChangeSetId,
      EntityKind.ComponentList,
      workspaceId,
    ),
  ) as ComponentInList[];
  assert(newList.length === 12, "new component missing in list");

  log("new component added");

  const indexPatchAdd = JSON.parse(`
{
  "meta": {
    "workspaceId": "01HRFEV0S23R1G23RP75QQDCA7",
    "changeSetId": "01K4ZF6QXKB3ZV3124ER2C0TFT",
    "toIndexChecksum": "03ce85f750506ee4f9d64396fdbacad0",
    "fromIndexChecksum": "8601f40a5bc4b73e90dba9d1a4126bfd"
  },
  "kind": "IndexUpdate",
  "indexChecksum": "03ce85f750506ee4f9d64396fdbacad0",
  "patch": {
    "kind": "ChangeSetMvIndex",
    "id": "56d81b6e3b3c2a49b4adcec6cab790e30d7b2f9fd382b655c34f7210288747a4",
    "fromChecksum": "8601f40a5bc4b73e90dba9d1a4126bfd",
    "toChecksum": "03ce85f750506ee4f9d64396fdbacad0",
    "patch": [
      {
        "op": "replace",
        "path": "/snapshotAddress",
        "value": "56d81b6e3b3c2a49b4adcec6cab790e30d7b2f9fd382b655c34f7210288747a4"
      },
      {
        "op": "replace",
        "path": "/mvList/0/checksum",
        "value": "da9dd52f0459df647b28a39223a6e915"
      },
      {
        "op": "replace",
        "path": "/mvList/10/kind",
        "value": "ActionPrototypeViewList"
      },
      {
        "op": "replace",
        "path": "/mvList/10/id",
        "value": "01K4ZGNVG0KMZ40BFF90VXM9GS"
      },
      {
        "op": "replace",
        "path": "/mvList/10/checksum",
        "value": "64e6f593d6a71875778eb45107b94782"
      },
      {
        "op": "replace",
        "path": "/mvList/11/kind",
        "value": "ActionViewList"
      },
      {
        "op": "replace",
        "path": "/mvList/11/id",
        "value": "01HRFEV0S23R1G23RP75QQDCA7"
      },
      {
        "op": "replace",
        "path": "/mvList/11/checksum",
        "value": "844f283b6b7c1e831b48f58c433e4393"
      },
      {
        "op": "replace",
        "path": "/mvList/12/id",
        "value": "01JYPVBKQGCZY2MANT0GW3MZRS"
      },
      {
        "op": "replace",
        "path": "/mvList/12/checksum",
        "value": "d3aa5b0bd1e74bd68706f583642e2a5e"
      },
      {
        "op": "replace",
        "path": "/mvList/13/id",
        "value": "01JYPVCJC71BRFCK1KZKRMN5HX"
      },
      {
        "op": "replace",
        "path": "/mvList/13/checksum",
        "value": "559e2405a91d068e6b681170c22d6ebf"
      },
      {
        "op": "replace",
        "path": "/mvList/14/id",
        "value": "01JYPVE0W40KC9EZ0YERQ30RNB"
      },
      {
        "op": "replace",
        "path": "/mvList/14/checksum",
        "value": "d882556661877f194e0fe439e9cc8784"
      },
      {
        "op": "replace",
        "path": "/mvList/15/id",
        "value": "01JZ961KGGBTPAQ6T6620985AB"
      },
      {
        "op": "replace",
        "path": "/mvList/15/checksum",
        "value": "27fd189ef3a0bb0a8706bfe61f2208b3"
      },
      {
        "op": "replace",
        "path": "/mvList/16/id",
        "value": "01JZK1VKDN40NZCCTYR350RDFD"
      },
      {
        "op": "replace",
        "path": "/mvList/16/checksum",
        "value": "cfc02804f644e52b50db21f66b0be07b"
      },
      {
        "op": "replace",
        "path": "/mvList/17/id",
        "value": "01JZND7T6F6VEWK5Q0FBNFYQBZ"
      },
      {
        "op": "replace",
        "path": "/mvList/17/checksum",
        "value": "d803a3f1f77af406c40bd6a5a18690dd"
      },
      {
        "op": "replace",
        "path": "/mvList/18/id",
        "value": "01K1GRZHEKA8EK4RVC8X2M8N6Y"
      },
      {
        "op": "replace",
        "path": "/mvList/18/checksum",
        "value": "556c8dbd4e84cfc49d3fe6ef5d2a47c1"
      },
      {
        "op": "replace",
        "path": "/mvList/19/id",
        "value": "01K1GS00KRGAP3VAAMTN9NQNZR"
      },
      {
        "op": "replace",
        "path": "/mvList/19/checksum",
        "value": "36d643e8a91283fc0848e66b28116838"
      },
      {
        "op": "replace",
        "path": "/mvList/20/id",
        "value": "01K1GS643R30F0N4FPBBM19W5Z"
      },
      {
        "op": "replace",
        "path": "/mvList/20/checksum",
        "value": "0e8273e32e6e6b381f79ca3327f3b062"
      },
      {
        "op": "replace",
        "path": "/mvList/21/id",
        "value": "01K2ZNABR4CQV2051T0Z4FWXDR"
      },
      {
        "op": "replace",
        "path": "/mvList/21/checksum",
        "value": "b31354220a83c99c42a1c7f68744eafc"
      },
      {
        "op": "replace",
        "path": "/mvList/22/kind",
        "value": "AttributeTree"
      },
      {
        "op": "replace",
        "path": "/mvList/22/id",
        "value": "01K3GTCTNVYW3NGHP33G5F31XD"
      },
      {
        "op": "replace",
        "path": "/mvList/22/checksum",
        "value": "7b1c3ddec931ed78d11021e9a36ef50d"
      },
      {
        "op": "replace",
        "path": "/mvList/23/kind",
        "value": "AttributeTree"
      },
      {
        "op": "replace",
        "path": "/mvList/23/id",
        "value": "01K4ZGNVS7EQKKBA67BMG0A62S"
      },
      {
        "op": "replace",
        "path": "/mvList/23/checksum",
        "value": "0d449d6c57c6e20bb3dc0e5d06109872"
      },
      {
        "op": "replace",
        "path": "/mvList/24/id",
        "value": "01JYPVBKQGCZY2MANT0GW3MZRS"
      },
      {
        "op": "replace",
        "path": "/mvList/24/checksum",
        "value": "d8044e598ae73cb99cf437082b51f6b1"
      },
      {
        "op": "replace",
        "path": "/mvList/25/id",
        "value": "01JYPVCJC71BRFCK1KZKRMN5HX"
      },
      {
        "op": "replace",
        "path": "/mvList/25/checksum",
        "value": "999c17dcd7e0bc88b231e3da51b584c2"
      },
      {
        "op": "replace",
        "path": "/mvList/26/id",
        "value": "01JYPVE0W40KC9EZ0YERQ30RNB"
      },
      {
        "op": "replace",
        "path": "/mvList/26/checksum",
        "value": "bbe7363fe6d3c287b849e002b6f24a35"
      },
      {
        "op": "replace",
        "path": "/mvList/27/id",
        "value": "01JZ961KGGBTPAQ6T6620985AB"
      },
      {
        "op": "replace",
        "path": "/mvList/27/checksum",
        "value": "a8c99bc009ffce12fe0d318ebfd09fa1"
      },
      {
        "op": "replace",
        "path": "/mvList/28/id",
        "value": "01JZK1VKDN40NZCCTYR350RDFD"
      },
      {
        "op": "replace",
        "path": "/mvList/28/checksum",
        "value": "3a5c3bbd6c26c8fe5b9e27c0fbd61362"
      },
      {
        "op": "replace",
        "path": "/mvList/29/id",
        "value": "01JZND7T6F6VEWK5Q0FBNFYQBZ"
      },
      {
        "op": "replace",
        "path": "/mvList/29/checksum",
        "value": "b1e4a2d1f49c3a007f15196fb1db1a4a"
      },
      {
        "op": "replace",
        "path": "/mvList/30/id",
        "value": "01K1GRZHEKA8EK4RVC8X2M8N6Y"
      },
      {
        "op": "replace",
        "path": "/mvList/30/checksum",
        "value": "6c819c6c085d5f0ee0c3fe115e230f85"
      },
      {
        "op": "replace",
        "path": "/mvList/31/id",
        "value": "01K1GS00KRGAP3VAAMTN9NQNZR"
      },
      {
        "op": "replace",
        "path": "/mvList/31/checksum",
        "value": "ef31d4833b7af0345ca41689784e8709"
      },
      {
        "op": "replace",
        "path": "/mvList/32/id",
        "value": "01K1GS643R30F0N4FPBBM19W5Z"
      },
      {
        "op": "replace",
        "path": "/mvList/32/checksum",
        "value": "37eaf2c3c5c0feb37acf145f1622b0fa"
      },
      {
        "op": "replace",
        "path": "/mvList/33/kind",
        "value": "Component"
      },
      {
        "op": "replace",
        "path": "/mvList/33/id",
        "value": "01K2ZNABR4CQV2051T0Z4FWXDR"
      },
      {
        "op": "replace",
        "path": "/mvList/33/checksum",
        "value": "e954224a3964479ab83669294338e2f6"
      },
      {
        "op": "replace",
        "path": "/mvList/34/kind",
        "value": "Component"
      },
      {
        "op": "replace",
        "path": "/mvList/34/id",
        "value": "01K3GTCTNVYW3NGHP33G5F31XD"
      },
      {
        "op": "replace",
        "path": "/mvList/34/checksum",
        "value": "2ea75387de1dec40e3ea2726cf718361"
      },
      {
        "op": "replace",
        "path": "/mvList/35/kind",
        "value": "Component"
      },
      {
        "op": "replace",
        "path": "/mvList/35/id",
        "value": "01K4ZGNVS7EQKKBA67BMG0A62S"
      },
      {
        "op": "replace",
        "path": "/mvList/35/checksum",
        "value": "feb2c727b8082cb73a3fe9906788799b"
      },
      {
        "op": "replace",
        "path": "/mvList/36/id",
        "value": "01JYPVBKQGCZY2MANT0GW3MZRS"
      },
      {
        "op": "replace",
        "path": "/mvList/36/checksum",
        "value": "3cfe8f8c92c1b70048bab591f258e3aa"
      },
      {
        "op": "replace",
        "path": "/mvList/37/id",
        "value": "01JYPVCJC71BRFCK1KZKRMN5HX"
      },
      {
        "op": "replace",
        "path": "/mvList/37/checksum",
        "value": "db6baddd242158cc3738c1d0628828b0"
      },
      {
        "op": "replace",
        "path": "/mvList/38/id",
        "value": "01JYPVE0W40KC9EZ0YERQ30RNB"
      },
      {
        "op": "replace",
        "path": "/mvList/38/checksum",
        "value": "ea7998d753722685f2f7805824b008c9"
      },
      {
        "op": "replace",
        "path": "/mvList/39/id",
        "value": "01JZ961KGGBTPAQ6T6620985AB"
      },
      {
        "op": "replace",
        "path": "/mvList/39/checksum",
        "value": "14625e5264e3d3959650b08c410d4d6b"
      },
      {
        "op": "replace",
        "path": "/mvList/40/id",
        "value": "01JZK1VKDN40NZCCTYR350RDFD"
      },
      {
        "op": "replace",
        "path": "/mvList/40/checksum",
        "value": "851b07346db0a2bcaa2f91d6838eb556"
      },
      {
        "op": "replace",
        "path": "/mvList/41/id",
        "value": "01JZND7T6F6VEWK5Q0FBNFYQBZ"
      },
      {
        "op": "replace",
        "path": "/mvList/41/checksum",
        "value": "0f3b704b25806eb0acf02530530b712e"
      },
      {
        "op": "replace",
        "path": "/mvList/42/id",
        "value": "01K1GRZHEKA8EK4RVC8X2M8N6Y"
      },
      {
        "op": "replace",
        "path": "/mvList/42/checksum",
        "value": "84518fc14500a892618735edde418327"
      },
      {
        "op": "replace",
        "path": "/mvList/43/id",
        "value": "01K1GS00KRGAP3VAAMTN9NQNZR"
      },
      {
        "op": "replace",
        "path": "/mvList/43/checksum",
        "value": "d75bef1be33c11b3d9a8c9bc7a236052"
      },
      {
        "op": "replace",
        "path": "/mvList/44/kind",
        "value": "ComponentDiff"
      },
      {
        "op": "replace",
        "path": "/mvList/44/id",
        "value": "01K1GS643R30F0N4FPBBM19W5Z"
      },
      {
        "op": "replace",
        "path": "/mvList/44/checksum",
        "value": "a627274f3d37b13fb37e1f12fbdf820a"
      },
      {
        "op": "replace",
        "path": "/mvList/45/kind",
        "value": "ComponentDiff"
      },
      {
        "op": "replace",
        "path": "/mvList/45/id",
        "value": "01K2ZNABR4CQV2051T0Z4FWXDR"
      },
      {
        "op": "replace",
        "path": "/mvList/45/checksum",
        "value": "c74e9fabe7d291723d01f985d5859f82"
      },
      {
        "op": "replace",
        "path": "/mvList/46/kind",
        "value": "ComponentDiff"
      },
      {
        "op": "replace",
        "path": "/mvList/46/id",
        "value": "01K3GTCTNVYW3NGHP33G5F31XD"
      },
      {
        "op": "replace",
        "path": "/mvList/46/checksum",
        "value": "6dadc2b0e3d7ba7999763a0e27a3722d"
      },
      {
        "op": "replace",
        "path": "/mvList/47/kind",
        "value": "ComponentDiff"
      },
      {
        "op": "replace",
        "path": "/mvList/47/id",
        "value": "01K4ZGNVS7EQKKBA67BMG0A62S"
      },
      {
        "op": "replace",
        "path": "/mvList/47/checksum",
        "value": "8aa8765552b1dfb5c84ae0e7ff2b885a"
      },
      {
        "op": "replace",
        "path": "/mvList/48/id",
        "value": "01JYPVBKQGCZY2MANT0GW3MZRS"
      },
      {
        "op": "replace",
        "path": "/mvList/48/checksum",
        "value": "c2a53f985d39533aae7302844f2ab68f"
      },
      {
        "op": "replace",
        "path": "/mvList/49/id",
        "value": "01JYPVCJC71BRFCK1KZKRMN5HX"
      },
      {
        "op": "replace",
        "path": "/mvList/49/checksum",
        "value": "39c017c20ebac86f09cab8bbb64a1c62"
      },
      {
        "op": "replace",
        "path": "/mvList/50/id",
        "value": "01JYPVE0W40KC9EZ0YERQ30RNB"
      },
      {
        "op": "replace",
        "path": "/mvList/50/checksum",
        "value": "c7f964cb33b4fce6cb588e308317d2d3"
      },
      {
        "op": "replace",
        "path": "/mvList/51/id",
        "value": "01JZ961KGGBTPAQ6T6620985AB"
      },
      {
        "op": "replace",
        "path": "/mvList/51/checksum",
        "value": "6ab53c754a93c88c9cafd3e3dc203e28"
      },
      {
        "op": "replace",
        "path": "/mvList/52/id",
        "value": "01JZK1VKDN40NZCCTYR350RDFD"
      },
      {
        "op": "replace",
        "path": "/mvList/52/checksum",
        "value": "2d844ba7667dcb3b51e1d8c5f8627204"
      },
      {
        "op": "replace",
        "path": "/mvList/53/id",
        "value": "01JZND7T6F6VEWK5Q0FBNFYQBZ"
      },
      {
        "op": "replace",
        "path": "/mvList/53/checksum",
        "value": "d5de7efd45f8aa614a36bb56b0832b22"
      },
      {
        "op": "replace",
        "path": "/mvList/54/id",
        "value": "01K1GRZHEKA8EK4RVC8X2M8N6Y"
      },
      {
        "op": "replace",
        "path": "/mvList/54/checksum",
        "value": "2323f40fb743048f3171ab038dc50f2c"
      },
      {
        "op": "replace",
        "path": "/mvList/55/kind",
        "value": "ComponentInList"
      },
      {
        "op": "replace",
        "path": "/mvList/55/id",
        "value": "01K1GS00KRGAP3VAAMTN9NQNZR"
      },
      {
        "op": "replace",
        "path": "/mvList/55/checksum",
        "value": "14acdfe5d6cce185bc03ffaff3e75896"
      },
      {
        "op": "replace",
        "path": "/mvList/56/kind",
        "value": "ComponentInList"
      },
      {
        "op": "replace",
        "path": "/mvList/56/id",
        "value": "01K1GS643R30F0N4FPBBM19W5Z"
      },
      {
        "op": "replace",
        "path": "/mvList/56/checksum",
        "value": "c01c2e6ec0c3dbdf1ebd310ba85e1350"
      },
      {
        "op": "replace",
        "path": "/mvList/57/kind",
        "value": "ComponentInList"
      },
      {
        "op": "replace",
        "path": "/mvList/57/id",
        "value": "01K2ZNABR4CQV2051T0Z4FWXDR"
      },
      {
        "op": "replace",
        "path": "/mvList/57/checksum",
        "value": "16233a29cb6367b98119717dd0dcf82a"
      },
      {
        "op": "replace",
        "path": "/mvList/58/kind",
        "value": "ComponentInList"
      },
      {
        "op": "replace",
        "path": "/mvList/58/id",
        "value": "01K3GTCTNVYW3NGHP33G5F31XD"
      },
      {
        "op": "replace",
        "path": "/mvList/58/checksum",
        "value": "98bf0740585c43ec4c41d8cead12a437"
      },
      {
        "op": "replace",
        "path": "/mvList/59/kind",
        "value": "ComponentInList"
      },
      {
        "op": "replace",
        "path": "/mvList/59/id",
        "value": "01K4ZGNVS7EQKKBA67BMG0A62S"
      },
      {
        "op": "replace",
        "path": "/mvList/59/checksum",
        "value": "f7ce92fc3f94fe04dee944d2cdea7720"
      },
      {
        "op": "replace",
        "path": "/mvList/60/kind",
        "value": "ComponentList"
      },
      {
        "op": "replace",
        "path": "/mvList/60/id",
        "value": "01HRFEV0S23R1G23RP75QQDCA7"
      },
      {
        "op": "replace",
        "path": "/mvList/60/checksum",
        "value": "e7d1d6c884b8855a6e339ffed53e5723"
      },
      {
        "op": "replace",
        "path": "/mvList/61/kind",
        "value": "DependentValueComponentList"
      },
      {
        "op": "replace",
        "path": "/mvList/61/id",
        "value": "01HRFEV0S23R1G23RP75QQDCA7"
      },
      {
        "op": "replace",
        "path": "/mvList/61/checksum",
        "value": "8be472ebf67bea9c075bd8283a2777a2"
      },
      {
        "op": "replace",
        "path": "/mvList/62/kind",
        "value": "ErasedComponents"
      },
      {
        "op": "replace",
        "path": "/mvList/62/id",
        "value": "01HRFEV0S23R1G23RP75QQDCA7"
      },
      {
        "op": "replace",
        "path": "/mvList/62/checksum",
        "value": "6676c379faad7e9570aa0b15e528665e"
      },
      {
        "op": "replace",
        "path": "/mvList/63/id",
        "value": "01JYPVBKQGCZY2MANT0GW3MZRS"
      },
      {
        "op": "replace",
        "path": "/mvList/63/checksum",
        "value": "e84ca4d069578e3de600c9fa3bb5c2cd"
      },
      {
        "op": "replace",
        "path": "/mvList/64/id",
        "value": "01JYPVCJC71BRFCK1KZKRMN5HX"
      },
      {
        "op": "replace",
        "path": "/mvList/64/checksum",
        "value": "81b500830d32895804e03d139599e5c0"
      },
      {
        "op": "replace",
        "path": "/mvList/65/id",
        "value": "01JYPVE0W40KC9EZ0YERQ30RNB"
      },
      {
        "op": "replace",
        "path": "/mvList/65/checksum",
        "value": "ce8a1556e7627252103b2eaac2b5cec7"
      },
      {
        "op": "replace",
        "path": "/mvList/66/id",
        "value": "01JZ961KGGBTPAQ6T6620985AB"
      },
      {
        "op": "replace",
        "path": "/mvList/66/checksum",
        "value": "e11418e990c6b914530de5fc1454ff20"
      },
      {
        "op": "replace",
        "path": "/mvList/67/id",
        "value": "01JZK1VKDN40NZCCTYR350RDFD"
      },
      {
        "op": "replace",
        "path": "/mvList/67/checksum",
        "value": "185b62676993bdce48c3b3a8457f6aee"
      },
      {
        "op": "replace",
        "path": "/mvList/68/id",
        "value": "01JZND7T6F6VEWK5Q0FBNFYQBZ"
      },
      {
        "op": "replace",
        "path": "/mvList/68/checksum",
        "value": "30e1750c813fe79f40c133c6c3777508"
      },
      {
        "op": "replace",
        "path": "/mvList/69/kind",
        "value": "IncomingConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/69/id",
        "value": "01K1GRZHEKA8EK4RVC8X2M8N6Y"
      },
      {
        "op": "replace",
        "path": "/mvList/69/checksum",
        "value": "13d4a33df7962ab9f46036275c454a72"
      },
      {
        "op": "replace",
        "path": "/mvList/70/kind",
        "value": "IncomingConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/70/id",
        "value": "01K1GS00KRGAP3VAAMTN9NQNZR"
      },
      {
        "op": "replace",
        "path": "/mvList/70/checksum",
        "value": "eb887ff54e6fa02f6750cff035cbc139"
      },
      {
        "op": "replace",
        "path": "/mvList/71/kind",
        "value": "IncomingConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/71/id",
        "value": "01K1GS643R30F0N4FPBBM19W5Z"
      },
      {
        "op": "replace",
        "path": "/mvList/71/checksum",
        "value": "5877c5291fd6a2c17d15fb64ba28bb88"
      },
      {
        "op": "replace",
        "path": "/mvList/72/kind",
        "value": "IncomingConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/72/id",
        "value": "01K2ZNABR4CQV2051T0Z4FWXDR"
      },
      {
        "op": "replace",
        "path": "/mvList/72/checksum",
        "value": "beb9014f1ae402a7a3f245632169e16b"
      },
      {
        "op": "replace",
        "path": "/mvList/73/kind",
        "value": "IncomingConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/73/id",
        "value": "01K3GTCTNVYW3NGHP33G5F31XD"
      },
      {
        "op": "replace",
        "path": "/mvList/73/checksum",
        "value": "6f7d19a52d02e53709a70468fcf20896"
      },
      {
        "op": "replace",
        "path": "/mvList/74/kind",
        "value": "IncomingConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/74/id",
        "value": "01K4ZGNVS7EQKKBA67BMG0A62S"
      },
      {
        "op": "replace",
        "path": "/mvList/74/checksum",
        "value": "37b7770cd00c8b9b116a7b80dab12f4e"
      },
      {
        "op": "replace",
        "path": "/mvList/75/kind",
        "value": "IncomingConnectionsList"
      },
      {
        "op": "replace",
        "path": "/mvList/75/id",
        "value": "01HRFEV0S23R1G23RP75QQDCA7"
      },
      {
        "op": "replace",
        "path": "/mvList/75/checksum",
        "value": "6399d4337f82a08ca63b3a161eff0036"
      },
      {
        "op": "replace",
        "path": "/mvList/76/id",
        "value": "01J1QXAGAP2AQR8VGGCW5NMKV0"
      },
      {
        "op": "replace",
        "path": "/mvList/76/checksum",
        "value": "cd66cb6f76cd8d112739754a2fedfd19"
      },
      {
        "op": "replace",
        "path": "/mvList/77/id",
        "value": "01J1QXVEYNNPX24G6VNR2RBY3Y"
      },
      {
        "op": "replace",
        "path": "/mvList/77/checksum",
        "value": "4cceb654830d2fb9e71a747364380cdd"
      },
      {
        "op": "replace",
        "path": "/mvList/78/id",
        "value": "01J1QZ621A10A8VP1CTPPKKSCP"
      },
      {
        "op": "replace",
        "path": "/mvList/78/checksum",
        "value": "456bf946fb7b79f08cc95bc5fdf0597b"
      },
      {
        "op": "replace",
        "path": "/mvList/79/kind",
        "value": "LuminorkDefaultVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/79/id",
        "value": "01J1QZDXCGJV29YF0FTW61A7R0"
      },
      {
        "op": "replace",
        "path": "/mvList/79/checksum",
        "value": "e5a9ea5f8dfebe792cba7adbbfee4ec3"
      },
      {
        "op": "replace",
        "path": "/mvList/80/kind",
        "value": "LuminorkDefaultVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/80/id",
        "value": "01JK0QZHF0TXMJFVE1VK2175EC"
      },
      {
        "op": "replace",
        "path": "/mvList/80/checksum",
        "value": "01593f3242341c260cdfc98eb1b5d895"
      },
      {
        "op": "replace",
        "path": "/mvList/81/kind",
        "value": "LuminorkDefaultVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/81/id",
        "value": "01JK0QZHF6ZSY6JFM39MBCRJ4C"
      },
      {
        "op": "replace",
        "path": "/mvList/81/checksum",
        "value": "616b9727d42a879ad543147f4d3fc559"
      },
      {
        "op": "replace",
        "path": "/mvList/82/kind",
        "value": "LuminorkDefaultVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/82/id",
        "value": "01JK0RQN156NY7QDMBZ4P18PES"
      },
      {
        "op": "replace",
        "path": "/mvList/82/checksum",
        "value": "5ae6a874775ad20538c451f3ef18bbda"
      },
      {
        "op": "replace",
        "path": "/mvList/83/kind",
        "value": "LuminorkDefaultVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/83/id",
        "value": "01JK0RQN1A2WMW5QRD0SNJS40A"
      },
      {
        "op": "replace",
        "path": "/mvList/83/checksum",
        "value": "32394a0db46e01e770a0eb8f63cd0a00"
      },
      {
        "op": "replace",
        "path": "/mvList/84/kind",
        "value": "LuminorkDefaultVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/84/id",
        "value": "01JK0RQN1HW97HSBZPKTY8DJK0"
      },
      {
        "op": "replace",
        "path": "/mvList/84/checksum",
        "value": "aceb8bbd72b3aaca196e2cf0ab0da3ea"
      },
      {
        "op": "replace",
        "path": "/mvList/85/kind",
        "value": "LuminorkDefaultVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/85/id",
        "value": "01JK1AYWD3XWA7KEJZ22RDWG35"
      },
      {
        "op": "replace",
        "path": "/mvList/85/checksum",
        "value": "593d3085538aecbfc4de341682254972"
      },
      {
        "op": "replace",
        "path": "/mvList/86/id",
        "value": "01JYPVBKBV3R7GS1DF6RZ013K7"
      },
      {
        "op": "replace",
        "path": "/mvList/86/checksum",
        "value": "3f2990bc1521b5c2a1409b94d2d5523e"
      },
      {
        "op": "replace",
        "path": "/mvList/87/id",
        "value": "01JYPVCHYRN8PHPJSN89YHF4RA"
      },
      {
        "op": "replace",
        "path": "/mvList/87/checksum",
        "value": "b7062e55634584f6348127b4969a5a67"
      },
      {
        "op": "replace",
        "path": "/mvList/88/kind",
        "value": "LuminorkSchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/88/id",
        "value": "01JYPVD3A6J86TABR82FK3JHWE"
      },
      {
        "op": "replace",
        "path": "/mvList/88/checksum",
        "value": "d654361687bf38517fffe1675b95bb3b"
      },
      {
        "op": "replace",
        "path": "/mvList/89/kind",
        "value": "LuminorkSchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/89/id",
        "value": "01JYPVDYA05AZECXFDVQNWGG15"
      },
      {
        "op": "replace",
        "path": "/mvList/89/checksum",
        "value": "adc1b7514d40d2ee88b7952a387a6680"
      },
      {
        "op": "replace",
        "path": "/mvList/90/kind",
        "value": "LuminorkSchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/90/id",
        "value": "01JZ1GBF4YGZPDNV4D6AJKS5JD"
      },
      {
        "op": "replace",
        "path": "/mvList/90/checksum",
        "value": "5369dce4edd1ff209a733ddf903f0d87"
      },
      {
        "op": "replace",
        "path": "/mvList/91/kind",
        "value": "LuminorkSchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/91/id",
        "value": "01JZ6DM7HB9273HAM46BC6F54Z"
      },
      {
        "op": "replace",
        "path": "/mvList/91/checksum",
        "value": "401729cbc251261a200c8f0c0780ccfc"
      },
      {
        "op": "replace",
        "path": "/mvList/92/kind",
        "value": "LuminorkSchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/92/id",
        "value": "01JZK1VH7QWBQ9KN82AVE5P82K"
      },
      {
        "op": "replace",
        "path": "/mvList/92/checksum",
        "value": "ef15bfcdd31eaae1d06074524500f0df"
      },
      {
        "op": "replace",
        "path": "/mvList/93/kind",
        "value": "LuminorkSchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/93/id",
        "value": "01K1GRZEX1QAB58FG6Q2C38VS5"
      },
      {
        "op": "replace",
        "path": "/mvList/93/checksum",
        "value": "0eca2fb408ca138c85e597d55652247a"
      },
      {
        "op": "replace",
        "path": "/mvList/94/kind",
        "value": "LuminorkSchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/94/id",
        "value": "01K1GRZY5HYEV4FN3SG8803EEA"
      },
      {
        "op": "replace",
        "path": "/mvList/94/checksum",
        "value": "185f46ee3cde633389a7d559f4d0c1ff"
      },
      {
        "op": "replace",
        "path": "/mvList/95/kind",
        "value": "LuminorkSchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/95/id",
        "value": "01K4ZGNVG0KMZ40BFF90VXM9GS"
      },
      {
        "op": "replace",
        "path": "/mvList/95/checksum",
        "value": "ec33de6d1fa66392a64c3dee6a83b483"
      },
      {
        "op": "replace",
        "path": "/mvList/96/id",
        "value": "01JYPVBKQGCZY2MANT0GW3MZRS"
      },
      {
        "op": "replace",
        "path": "/mvList/96/checksum",
        "value": "e84ca4d069578e3de600c9fa3bb5c2cd"
      },
      {
        "op": "replace",
        "path": "/mvList/97/id",
        "value": "01JYPVCJC71BRFCK1KZKRMN5HX"
      },
      {
        "op": "replace",
        "path": "/mvList/97/checksum",
        "value": "75e95383b8537ddb785510f5989c2c97"
      },
      {
        "op": "replace",
        "path": "/mvList/98/id",
        "value": "01JYPVE0W40KC9EZ0YERQ30RNB"
      },
      {
        "op": "replace",
        "path": "/mvList/98/checksum",
        "value": "9520cad00f7bb577f56c6527e870315d"
      },
      {
        "op": "replace",
        "path": "/mvList/99/kind",
        "value": "ManagementConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/99/id",
        "value": "01JZ961KGGBTPAQ6T6620985AB"
      },
      {
        "op": "replace",
        "path": "/mvList/99/checksum",
        "value": "e11418e990c6b914530de5fc1454ff20"
      },
      {
        "op": "replace",
        "path": "/mvList/100/kind",
        "value": "ManagementConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/100/id",
        "value": "01JZK1VKDN40NZCCTYR350RDFD"
      },
      {
        "op": "replace",
        "path": "/mvList/100/checksum",
        "value": "9f309df9070c6da78310c6c2591753ad"
      },
      {
        "op": "replace",
        "path": "/mvList/101/kind",
        "value": "ManagementConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/101/id",
        "value": "01JZND7T6F6VEWK5Q0FBNFYQBZ"
      },
      {
        "op": "replace",
        "path": "/mvList/101/checksum",
        "value": "088b6cb25650075f7b3b97e6c9433b61"
      },
      {
        "op": "replace",
        "path": "/mvList/102/kind",
        "value": "ManagementConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/102/id",
        "value": "01K1GRZHEKA8EK4RVC8X2M8N6Y"
      },
      {
        "op": "replace",
        "path": "/mvList/102/checksum",
        "value": "7a13f4c0f6dbd15f4725e8e7d3e69b5e"
      },
      {
        "op": "replace",
        "path": "/mvList/103/kind",
        "value": "ManagementConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/103/id",
        "value": "01K1GS00KRGAP3VAAMTN9NQNZR"
      },
      {
        "op": "replace",
        "path": "/mvList/103/checksum",
        "value": "e0a655027bed364254149bc4c036d8b5"
      },
      {
        "op": "replace",
        "path": "/mvList/104/kind",
        "value": "ManagementConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/104/id",
        "value": "01K1GS643R30F0N4FPBBM19W5Z"
      },
      {
        "op": "replace",
        "path": "/mvList/104/checksum",
        "value": "31a4a283c75a105412891f323f57d6f9"
      },
      {
        "op": "replace",
        "path": "/mvList/105/kind",
        "value": "ManagementConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/105/id",
        "value": "01K2ZNABR4CQV2051T0Z4FWXDR"
      },
      {
        "op": "replace",
        "path": "/mvList/105/checksum",
        "value": "beb9014f1ae402a7a3f245632169e16b"
      },
      {
        "op": "replace",
        "path": "/mvList/106/kind",
        "value": "ManagementConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/106/id",
        "value": "01K3GTCTNVYW3NGHP33G5F31XD"
      },
      {
        "op": "replace",
        "path": "/mvList/106/checksum",
        "value": "6f7d19a52d02e53709a70468fcf20896"
      },
      {
        "op": "replace",
        "path": "/mvList/107/kind",
        "value": "ManagementConnections"
      },
      {
        "op": "replace",
        "path": "/mvList/107/id",
        "value": "01K4ZGNVS7EQKKBA67BMG0A62S"
      },
      {
        "op": "replace",
        "path": "/mvList/107/checksum",
        "value": "37b7770cd00c8b9b116a7b80dab12f4e"
      },
      {
        "op": "replace",
        "path": "/mvList/108/kind",
        "value": "SchemaMembers"
      },
      {
        "op": "replace",
        "path": "/mvList/108/id",
        "value": "01J1QXAGAP2AQR8VGGCW5NMKV0"
      },
      {
        "op": "replace",
        "path": "/mvList/108/checksum",
        "value": "ad8834df0b55a83745d119b7bb3936b3"
      },
      {
        "op": "replace",
        "path": "/mvList/109/kind",
        "value": "SchemaMembers"
      },
      {
        "op": "replace",
        "path": "/mvList/109/id",
        "value": "01J1QXVEYNNPX24G6VNR2RBY3Y"
      },
      {
        "op": "replace",
        "path": "/mvList/109/checksum",
        "value": "90e5fa5f3d5613b061d81f89b8448d2b"
      },
      {
        "op": "replace",
        "path": "/mvList/110/kind",
        "value": "SchemaMembers"
      },
      {
        "op": "replace",
        "path": "/mvList/110/id",
        "value": "01J1QZ621A10A8VP1CTPPKKSCP"
      },
      {
        "op": "replace",
        "path": "/mvList/110/checksum",
        "value": "7cf045bfef09c623ef255bfc918d667e"
      },
      {
        "op": "replace",
        "path": "/mvList/111/kind",
        "value": "SchemaMembers"
      },
      {
        "op": "replace",
        "path": "/mvList/111/id",
        "value": "01J1QZDXCGJV29YF0FTW61A7R0"
      },
      {
        "op": "replace",
        "path": "/mvList/111/checksum",
        "value": "d74e3e2e9417948c0bbc14cc46c279ec"
      },
      {
        "op": "replace",
        "path": "/mvList/112/kind",
        "value": "SchemaMembers"
      },
      {
        "op": "replace",
        "path": "/mvList/112/id",
        "value": "01JK0QZHF0TXMJFVE1VK2175EC"
      },
      {
        "op": "replace",
        "path": "/mvList/112/checksum",
        "value": "29c4bfa6f2844df7ae3b97cfe9996ed7"
      },
      {
        "op": "replace",
        "path": "/mvList/113/kind",
        "value": "SchemaMembers"
      },
      {
        "op": "replace",
        "path": "/mvList/113/id",
        "value": "01JK0QZHF6ZSY6JFM39MBCRJ4C"
      },
      {
        "op": "replace",
        "path": "/mvList/113/checksum",
        "value": "dac697f306c860f462abe2eedff11558"
      },
      {
        "op": "replace",
        "path": "/mvList/114/kind",
        "value": "SchemaMembers"
      },
      {
        "op": "replace",
        "path": "/mvList/114/id",
        "value": "01JK0RQN156NY7QDMBZ4P18PES"
      },
      {
        "op": "replace",
        "path": "/mvList/114/checksum",
        "value": "2ceff5b546e77e570d9a327eebb3d9d0"
      },
      {
        "op": "replace",
        "path": "/mvList/115/kind",
        "value": "SchemaMembers"
      },
      {
        "op": "replace",
        "path": "/mvList/115/id",
        "value": "01JK0RQN1A2WMW5QRD0SNJS40A"
      },
      {
        "op": "replace",
        "path": "/mvList/115/checksum",
        "value": "4c40a817b3a58585a23de288559812ce"
      },
      {
        "op": "replace",
        "path": "/mvList/116/kind",
        "value": "SchemaMembers"
      },
      {
        "op": "replace",
        "path": "/mvList/116/id",
        "value": "01JK0RQN1HW97HSBZPKTY8DJK0"
      },
      {
        "op": "replace",
        "path": "/mvList/116/checksum",
        "value": "a6642c18f8ad56b35badb1a6b2709992"
      },
      {
        "op": "replace",
        "path": "/mvList/117/kind",
        "value": "SchemaMembers"
      },
      {
        "op": "replace",
        "path": "/mvList/117/id",
        "value": "01JK1AYWD3XWA7KEJZ22RDWG35"
      },
      {
        "op": "replace",
        "path": "/mvList/117/checksum",
        "value": "4ed3ec60448042c1d3413fb8659330fe"
      },
      {
        "op": "replace",
        "path": "/mvList/118/kind",
        "value": "SchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/118/id",
        "value": "01JYPVBKBV3R7GS1DF6RZ013K7"
      },
      {
        "op": "replace",
        "path": "/mvList/118/checksum",
        "value": "cf637f46a91f0800abb52c208a3776d6"
      },
      {
        "op": "replace",
        "path": "/mvList/119/kind",
        "value": "SchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/119/id",
        "value": "01JYPVCHYRN8PHPJSN89YHF4RA"
      },
      {
        "op": "replace",
        "path": "/mvList/119/checksum",
        "value": "180b5e2605f469bb0db8a182ebb32381"
      },
      {
        "op": "replace",
        "path": "/mvList/120/kind",
        "value": "SchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/120/id",
        "value": "01JYPVD3A6J86TABR82FK3JHWE"
      },
      {
        "op": "replace",
        "path": "/mvList/120/checksum",
        "value": "45da9931c3b2163a96e090311b86c4c6"
      },
      {
        "op": "replace",
        "path": "/mvList/121/kind",
        "value": "SchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/121/id",
        "value": "01JYPVDYA05AZECXFDVQNWGG15"
      },
      {
        "op": "replace",
        "path": "/mvList/121/checksum",
        "value": "4022d20aa724d6a0cc5b66da1bdd9028"
      },
      {
        "op": "replace",
        "path": "/mvList/122/kind",
        "value": "SchemaVariant"
      },
      {
        "op": "replace",
        "path": "/mvList/122/id",
        "value": "01JZ1GBF4YGZPDNV4D6AJKS5JD"
      },
      {
        "op": "replace",
        "path": "/mvList/122/checksum",
        "value": "02b59fd2a2afcc894d079f911e5c56f3"
      },
      {
        "op": "add",
        "path": "/mvList/123",
        "value": {
          "kind": "SchemaVariant",
          "id": "01JZ6DM7HB9273HAM46BC6F54Z",
          "checksum": "6970fe0954cf9518f9b93ef765ea37d9"
        }
      },
      {
        "op": "add",
        "path": "/mvList/124",
        "value": {
          "kind": "SchemaVariant",
          "id": "01JZK1VH7QWBQ9KN82AVE5P82K",
          "checksum": "692d71b0abdfa15f4509cf9066bf4458"
        }
      },
      {
        "op": "add",
        "path": "/mvList/125",
        "value": {
          "kind": "SchemaVariant",
          "id": "01K1GRZEX1QAB58FG6Q2C38VS5",
          "checksum": "2f725362e460a07dfc1c7aa56a43afee"
        }
      },
      {
        "op": "add",
        "path": "/mvList/126",
        "value": {
          "kind": "SchemaVariant",
          "id": "01K1GRZY5HYEV4FN3SG8803EEA",
          "checksum": "c2e44a4e670b61d2eb23b7616d5f5a64"
        }
      },
      {
        "op": "add",
        "path": "/mvList/127",
        "value": {
          "kind": "SchemaVariant",
          "id": "01K4ZGNVG0KMZ40BFF90VXM9GS",
          "checksum": "621f25afe1acb8fee18f8227d5c09a4c"
        }
      },
      {
        "op": "add",
        "path": "/mvList/128",
        "value": {
          "kind": "SchemaVariantCategories",
          "id": "01HRFEV0S23R1G23RP75QQDCA7",
          "checksum": "27b8f36fc1d18738271eb9ecd1903439"
        }
      },
      {
        "op": "add",
        "path": "/mvList/129",
        "value": {
          "kind": "View",
          "id": "01JYPTEC0T4C53TF747RV5JBX8",
          "checksum": "d972afb3cff12f3e9784e258175ee01e"
        }
      },
      {
        "op": "add",
        "path": "/mvList/130",
        "value": {
          "kind": "View",
          "id": "01JZ6DKPS8YE4T589WEF3NXC01",
          "checksum": "d609d78660a3232224bffceb30fe0398"
        }
      },
      {
        "op": "add",
        "path": "/mvList/131",
        "value": {
          "kind": "ViewComponentList",
          "id": "01JYPTEC0T4C53TF747RV5JBX8",
          "checksum": "f6781d224ce9ddc7c690a8ab309bc5e8"
        }
      },
      {
        "op": "add",
        "path": "/mvList/132",
        "value": {
          "kind": "ViewComponentList",
          "id": "01JZ6DKPS8YE4T589WEF3NXC01",
          "checksum": "b9a42acef68f5fb8f0c268be2492cbf0"
        }
      },
      {
        "op": "add",
        "path": "/mvList/133",
        "value": {
          "kind": "ViewList",
          "id": "01HRFEV0S23R1G23RP75QQDCA7",
          "checksum": "2b0f127b478a3f350b91f4b41599982a"
        }
      }
    ]
  }
}
  `) as WorkspaceIndexUpdate;
  await db.handleIndexMvPatch(indexPatchAdd);

  const indexChecksum = "03ce85f750506ee4f9d64396fdbacad0";
  const indexLookup = await db.exec({
    sql: "select checksum from indexes where checksum = ? ",
    bind: [indexChecksum],
    returnValue: "resultRows",
  });
  const dbChecksum = oneInOne(indexLookup);
  assert(
    dbChecksum === indexChecksum,
    `index checksum not written: ${dbChecksum?.toString()} != ${indexChecksum}`,
  );

  const atomLookup = await db.exec({
    sql: "select checksum from atoms where kind = ? and args = ? and checksum = ?",
    bind: [EntityKind.MvIndex, workspaceId, indexChecksum],
    returnValue: "resultRows",
  });
  const atomChecksum = oneInOne(atomLookup);
  assert(
    atomChecksum === indexChecksum,
    `atom checksum not updated: ${atomChecksum?.toString()} != ${indexChecksum}`,
  );

  // get the index mv, assures that the atom and MTM are in place!
  const indexListAtom = (await db.get(
    workspaceId,
    changeSetId,
    EntityKind.MvIndex,
    workspaceId,
  )) as object | -1;
  assert(indexListAtom !== -1, `MvIndex Atom doesn't exist when it should`);

  log("index update patched");

  await db.vanaheim(workspaceId);

  const numGlobal = await db.exec({
    sql: "select count(*) from global_atoms",
    returnValue: "resultRows",
  });
  const num = parseInt(oneInOne(numGlobal) as string);
  assert(num > 0, "no global MVs after vanaheim");

  const variantId = "01J1QXEJC12EEBZ00H4T15YHNQ";
  const variant = (await db.getGlobal(
    workspaceId,
    EntityKind.CachedDefaultVariant,
    variantId,
  )) as CachedDefaultVariant;
  assert(variant.id === variantId, "global variant id does not match");

  const updatedName = "Updated Docker Image";
  await db.handleDeploymentPatchMessage({
    meta: {
      fromIndexChecksum: "d556d181be774b20d83d5deb7eb61448",
      toIndexChecksum: "doesntmatter",
    },
    kind: MessageKind.DEPLOYMENT_PATCH,
    patches: [
      {
        patch: [{ op: "replace", path: "/displayName", value: updatedName }],
        id: variantId,
        kind: EntityKind.CachedDefaultVariant,
        fromChecksum: "5283e232a15993fdd4b03499a8b3058d",
        toChecksum: "doesntmatter",
      },
    ],
  });

  const variantAfter = (await db.getGlobal(
    workspaceId,
    EntityKind.CachedDefaultVariant,
    variantId,
  )) as CachedDefaultVariant;
  assert(
    variantAfter.id === variantId,
    "after global variant id does not match",
  );

  assert(
    variantAfter.displayName === updatedName,
    "variant name did not update",
  );

  // remove an MV to express more of the functionality
  const removePatch = JSON.parse(`
{
  "meta": {
    "workspaceId": "01HRFEV0S23R1G23RP75QQDCA7",
    "changeSetId": "01K4ZF6QXKB3ZV3124ER2C0TFT",
    "toIndexChecksum": "03ce85f750506ee4f9d64396fdbacad0-removed",
    "fromIndexChecksum": "03ce85f750506ee4f9d64396fdbacad0"
  },
  "kind": "PatchMessage",
  "patches": [
    {
      "kind": "ComponentInList",
      "id": "01K4ZGNVS7EQKKBA67BMG0A62S",
      "fromChecksum": "37b7770cd00c8b9b116a7b80dab12f4e",
      "toChecksum": "0",
      "patch": []
    }
  ]
}`) as WorkspacePatchBatch;
  await db.handleWorkspacePatchMessage(removePatch);
  const compoenentInListAtom = (await db.get(
    workspaceId,
    changeSetId,
    EntityKind.ComponentInList,
    "01K4ZGNVS7EQKKBA67BMG0A62S",
  )) as object | -1;
  assert(
    compoenentInListAtom === -1,
    `ComponentInList Atom shouldn't exist when it does`,
  );
  log("removed component");

  log("deployment MV done");
};

/**
 * THE INVOCATION
 */

async function go() {
  const worker = new Worker(new URL(workerUrl, import.meta.url), {
    type: "module",
  });
  const db: Comlink.Remote<TabDBInterface> = Comlink.wrap(worker);
  db.addListenerBustCache(Comlink.proxy(bustTanStackCache));
  db.addListenerInFlight(Comlink.proxy(() => {}));
  db.addListenerReturned(Comlink.proxy(() => {}));
  db.addAtomUpdated(Comlink.proxy(() => {}));
  db.addListenerLobbyExit(Comlink.proxy(() => {}));
  db.addConnStatusFn(Comlink.proxy(() => {}));
  await db.initDB(true);
  await db.migrate(true);
  db.createLock();
  db.setBearer(workspaceId, "doesnt matter 123");
  try {
    await fullDiagnosticTest(db);
  } catch (e: unknown) {
    // eslint-disable-next-line no-console
    console.error(e);
    if (e instanceof Error) {
      assert(false, e.toString());
    } else {
      assert(false, e as string);
    }
  } finally {
    done();
  }
}

go();
