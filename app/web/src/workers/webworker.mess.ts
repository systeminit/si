/* eslint-disable no-useless-escape */
import * as Comlink from "comlink";
import { ulid } from "ulid";
import {
  TabDBInterface,
  BustCacheFn,
  AtomWithDocument,
  Gettable,
} from "@/workers/types/dbinterface";
import { EntityKind } from "./types/entity_kind_types";

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
const writeElm = document.querySelector("#write > ul");
const readElm = document.querySelector("#read > ul");

let totalDbCount = 0;

const createMsg = (msg: string) => {
  const li = document.createElement("li");
  li.appendChild(document.createTextNode(msg));
  return li;
};

const writes = (msg: string) => {
  writeElm?.append(createMsg(msg));
};

const reads = (msg: string) => {
  readElm?.append(createMsg(msg));
};

const done = () => {
  const end = performance.now();
  const ms = end - perfTestRunStart;
  const msg = `Whole perf test took ${ms.toFixed(3)}ms`;
  const elm = document.getElementById("timestamp");
  const stamp = document.createTextNode(msg);
  elm?.appendChild(stamp);
};

/**
 * THE TEST
 *
 * The goal of this test is to assess the performance of SQLite
 * What are the limits (number of statements) that either reads
 * or writes become slow?
 *
 * NOTE: the product behavior is designed such that when we load,
 * we do a lot of writes. Then reads are opened. When new writes come
 * in, new reads *can* happen as a result of user behavior. When writes
 * finish, reads will be re-fired. We want this test to mimic write-then-read
 * not "throw writes and reads at the very same time and see what happens"
 *
 * In order to directly compare with workspaces in PROD
 * We want to include the thread interface IO transfer cost
 * in these measurements.
 *
 * NOTE: if we want to see the behavior for "throw a bunch of stuff
 * at the wall at the same time"... just start taking off `await Promise.all`
 * and just let everything hit the event loop
 *
 */
const workspaceId = "01HRFEV0S23R1G23RP75QQDCA7";
const changeSetId = "01JYPTEC5JM3T1Y4ECEPT9560J";

// variable sized data, where 1 is large and 3 are smaller
const large = `
{
        "id": "01K1GRZY5HYEV4FN3SG8803EEA",
        "schemaId": "01JK0RQN1A2WMW5QRD0SNJS40A",
        "schemaName": "AWS::EC2::RouteTable",
        "schemaVariantId": "01K1GRZY5HYEV4FN3SG8803EEA",
        "version": "20250731173802496447000",
        "displayName": "AWS::EC2::RouteTable",
        "category": "AWS::EC2",
        "description": "Specifies a route table for the specified VPC. After you create a route table, you can add routes and associate the table with a subnet. For more information, see [Route tables](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_Route_Tables.html) in the *Amazon VPC User Guide*.",
        "link": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-routetable.html",
        "color": "#FF9900",
        "isLocked": true,
        "created_at": "2025-07-31T17:38:02.033001Z",
        "updated_at": "2025-07-31T17:38:02.033001Z",
        "canCreateNewComponents": true,
        "isSecretDefining": false,
        "canContribute": false,
        "mgmtFunctions": [
          {
            "id": "01K1GRZYG8AZ149KHS5F4NQJH0",
            "funcId": "01K1GRZY4GD7T21FD62XD8WNJA",
            "description": null,
            "prototypeName": "Discover on AWS",
            "name": "Discover on AWS",
            "kind": "discover"
          },
          {
            "id": "01K1GRZYG90F9BXMNKKXQA2SCJ",
            "funcId": "01K1GRZY4C6EWX8CVVXT7NZHC3",
            "description": null,
            "prototypeName": "Import from AWS",
            "name": "Import from AWS",
            "kind": "import"
          }
        ],
        "propTree": {
          "props": {
            "01K1GRZY9XD1C408ZSSMRCKHFS": {
              "id": "01K1GRZY9XD1C408ZSSMRCKHFS",
              "kind": "string",
              "childKind": null,
              "widgetKind": "text",
              "name": "Key",
              "path": "root/domain/Tags/TagsItem/Key",
              "hidden": false,
              "eligibleForConnection": true,
              "createOnly": false,
              "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-routetable-tag.html#cfn-ec2-routetable-tag-key",
              "documentation": "The tag key.",
              "validationFormat": "{\\"type\\":\\"string\\",\\"flags\\":{\\"presence\\":\\"required\\"}}",
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
            "01K1GRZY619065Q4CDPS3E6J71": {
              "id": "01K1GRZY619065Q4CDPS3E6J71",
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
            "01K1GRZY5K1538VWZVVH2MXMP5": {
              "id": "01K1GRZY5K1538VWZVVH2MXMP5",
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
            "01K1GRZY5J8MJ6KABW40PEX72V": {
              "id": "01K1GRZY5J8MJ6KABW40PEX72V",
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
            "01K1GRZY5Q4NK3BASQ3NZZP75Y": {
              "id": "01K1GRZY5Q4NK3BASQ3NZZP75Y",
              "kind": "object",
              "childKind": "string",
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
            "01K1GRZY5Q4NK3BASQ3NZZP762": {
              "id": "01K1GRZY5Q4NK3BASQ3NZZP762",
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
            "01K1GRZY5PKEP0GKBAV4DE6F92": {
              "id": "01K1GRZY5PKEP0GKBAV4DE6F92",
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
            "01K1GRZY9YYTVYBAV7ER36CPT7": {
              "id": "01K1GRZY9YYTVYBAV7ER36CPT7",
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
            "01K1GRZY9XD1C408ZSSMRCKHFV": {
              "id": "01K1GRZY9XD1C408ZSSMRCKHFV",
              "kind": "string",
              "childKind": null,
              "widgetKind": "text",
              "name": "Value",
              "path": "root/domain/Tags/TagsItem/Value",
              "hidden": false,
              "eligibleForConnection": true,
              "createOnly": false,
              "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-ec2-routetable-tag.html#cfn-ec2-routetable-tag-value",
              "documentation": "The tag value.",
              "validationFormat": "{\\"type\\":\\"string\\",\\"flags\\":{\\"presence\\":\\"required\\"}}",
              "defaultCanBeSetBySocket": false,
              "isOriginSecret": false,
              "secretDefinition": null
            },
            "01K1GRZY9Z29H6N0T3V5MQE1K1": {
              "id": "01K1GRZY9Z29H6N0T3V5MQE1K1",
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
              "defaultCanBeSetBySocket": true,
              "isOriginSecret": false,
              "secretDefinition": null,
              "suggestSources": [
                {
                  "prop": "/domain/region",
                  "schema": "Region"
                }
              ]
            },
            "01K1GRZY62MM7VXETJEYDW54C8": {
              "id": "01K1GRZY62MM7VXETJEYDW54C8",
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
            "01K1GRZY5MQ278E8C2BTTHP465": {
              "id": "01K1GRZY5MQ278E8C2BTTHP465",
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
            "01K1GRZYA1Z5GV4M5AHFTFB7ST": {
              "id": "01K1GRZYA1Z5GV4M5AHFTFB7ST",
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
            "01K1GRZY5NTQ50N9DWPH41BZXJ": {
              "id": "01K1GRZY5NTQ50N9DWPH41BZXJ",
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
            "01K1GRZY9VHJBXW2N31E6V740N": {
              "id": "01K1GRZY9VHJBXW2N31E6V740N",
              "kind": "array",
              "childKind": "object",
              "widgetKind": "array",
              "name": "Tags",
              "path": "root/domain/Tags",
              "hidden": false,
              "eligibleForConnection": true,
              "createOnly": false,
              "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-routetable.html#cfn-ec2-routetable-tags",
              "documentation": "Any tags assigned to the route table.",
              "validationFormat": null,
              "defaultCanBeSetBySocket": true,
              "isOriginSecret": false,
              "secretDefinition": null
            },
            "01K1GRZY5TTHRK378EK8Y9DKJ2": {
              "id": "01K1GRZY5TTHRK378EK8Y9DKJ2",
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
            "01K1GRZYA380X39C7AQA2A1K5F": {
              "id": "01K1GRZYA380X39C7AQA2A1K5F",
              "kind": "string",
              "childKind": null,
              "widgetKind": "text",
              "name": "RouteTableId",
              "path": "root/resource_value/RouteTableId",
              "hidden": false,
              "eligibleForConnection": true,
              "createOnly": false,
              "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-routetable.html#cfn-ec2-routetable-routetableid",
              "documentation": "",
              "validationFormat": null,
              "defaultCanBeSetBySocket": false,
              "isOriginSecret": false,
              "secretDefinition": null
            },
            "01K1GRZY5J8MJ6KABW40PEX72Q": {
              "id": "01K1GRZY5J8MJ6KABW40PEX72Q",
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
            "01K1GRZY5RBGQFXZH6ZNMJFSYB": {
              "id": "01K1GRZY5RBGQFXZH6ZNMJFSYB",
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
            "01K1GRZY5SF3B9K6XZ0S6G0BDY": {
              "id": "01K1GRZY5SF3B9K6XZ0S6G0BDY",
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
            "01K1GRZY60KF5D9NYEMWC5JJ8K": {
              "id": "01K1GRZY60KF5D9NYEMWC5JJ8K",
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
            "01K1GRZYA5V8SP1C65Z3TV2QA0": {
              "id": "01K1GRZYA5V8SP1C65Z3TV2QA0",
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
              "defaultCanBeSetBySocket": true,
              "isOriginSecret": false,
              "secretDefinition": null
            },
            "01K1GRZY9T0T4EPR44MNQ3RGYK": {
              "id": "01K1GRZY9T0T4EPR44MNQ3RGYK",
              "kind": "string",
              "childKind": null,
              "widgetKind": "text",
              "name": "VpcId",
              "path": "root/domain/VpcId",
              "hidden": false,
              "eligibleForConnection": true,
              "createOnly": true,
              "docLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-routetable.html#cfn-ec2-routetable-vpcid",
              "documentation": "The ID of the VPC.",
              "validationFormat": "{\\"type\\":\\"string\\",\\"flags\\":{\\"presence\\":\\"required\\"}}",
              "defaultCanBeSetBySocket": true,
              "isOriginSecret": false,
              "secretDefinition": null,
              "suggestSources": [
                {
                  "prop": "/resource_value/VpcId",
                  "schema": "AWS::EC2::VPC"
                }
              ]
            },
            "01K1GRZY5NTQ50N9DWPH41BZXG": {
              "id": "01K1GRZY5NTQ50N9DWPH41BZXG",
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
            "01K1GRZY5XQTNDJ3EX1G6V35YF": {
              "id": "01K1GRZY5XQTNDJ3EX1G6V35YF",
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
            "01K1GRZY5W8M93YV376R66D3H9": {
              "id": "01K1GRZY5W8M93YV376R66D3H9",
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
            "01K1GRZY9Z29H6N0T3V5MQE1K3": {
              "id": "01K1GRZY9Z29H6N0T3V5MQE1K3",
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
            "01K1GRZY637AK0GW8N1A52P4FZ": {
              "id": "01K1GRZY637AK0GW8N1A52P4FZ",
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
            "01K1GRZY5SF3B9K6XZ0S6G0BE0": {
              "id": "01K1GRZY5SF3B9K6XZ0S6G0BE0",
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
            "01K1GRZY5Y4413D1DVM8F1930K": {
              "id": "01K1GRZY5Y4413D1DVM8F1930K",
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
            "01K1GRZYA0VXMVD85TQ223ZYBQ": {
              "id": "01K1GRZYA0VXMVD85TQ223ZYBQ",
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
            "01K1GRZY619065Q4CDPS3E6J6X": {
              "id": "01K1GRZY619065Q4CDPS3E6J6X",
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
            "01K1GRZY5Y4413D1DVM8F1930Q": {
              "id": "01K1GRZY5Y4413D1DVM8F1930Q",
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
            "01K1GRZY5W8M93YV376R66D3HB": {
              "id": "01K1GRZY5W8M93YV376R66D3HB",
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
            "01K1GRZY9WW8S6Y8ZAMXHNWCQF": {
              "id": "01K1GRZY9WW8S6Y8ZAMXHNWCQF",
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
            "01K1GRZY5ZT0S3ZMA2DRCHZ6Y1": {
              "id": "01K1GRZY5ZT0S3ZMA2DRCHZ6Y1",
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
            }
          },
          "treeInfo": {
            "01K1GRZY5PKEP0GKBAV4DE6F92": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72V",
              "children": []
            },
            "01K1GRZY9XD1C408ZSSMRCKHFV": {
              "parent": "01K1GRZY9WW8S6Y8ZAMXHNWCQF",
              "children": []
            },
            "01K1GRZY60KF5D9NYEMWC5JJ8K": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72Q",
              "children": [
                "01K1GRZY619065Q4CDPS3E6J6X"
              ]
            },
            "01K1GRZY5W8M93YV376R66D3HB": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72Q",
              "children": [
                "01K1GRZYA380X39C7AQA2A1K5F"
              ]
            },
            "01K1GRZY5J8MJ6KABW40PEX72V": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72Q",
              "children": [
                "01K1GRZY5K1538VWZVVH2MXMP5",
                "01K1GRZY5MQ278E8C2BTTHP465",
                "01K1GRZY5NTQ50N9DWPH41BZXG",
                "01K1GRZY5NTQ50N9DWPH41BZXJ",
                "01K1GRZY5PKEP0GKBAV4DE6F92"
              ]
            },
            "01K1GRZY5RBGQFXZH6ZNMJFSYB": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72Q",
              "children": [
                "01K1GRZY5SF3B9K6XZ0S6G0BDY",
                "01K1GRZY5SF3B9K6XZ0S6G0BE0",
                "01K1GRZY5TTHRK378EK8Y9DKJ2",
                "01K1GRZY5W8M93YV376R66D3H9"
              ]
            },
            "01K1GRZY5K1538VWZVVH2MXMP5": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72V",
              "children": []
            },
            "01K1GRZY5Q4NK3BASQ3NZZP75Y": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72Q",
              "children": [
                "01K1GRZY9T0T4EPR44MNQ3RGYK",
                "01K1GRZY9VHJBXW2N31E6V740N",
                "01K1GRZY9YYTVYBAV7ER36CPT7"
              ]
            },
            "01K1GRZY9XD1C408ZSSMRCKHFS": {
              "parent": "01K1GRZY9WW8S6Y8ZAMXHNWCQF",
              "children": []
            },
            "01K1GRZY637AK0GW8N1A52P4FZ": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72Q",
              "children": []
            },
            "01K1GRZY5SF3B9K6XZ0S6G0BDY": {
              "parent": "01K1GRZY5RBGQFXZH6ZNMJFSYB",
              "children": []
            },
            "01K1GRZY9VHJBXW2N31E6V740N": {
              "parent": "01K1GRZY5Q4NK3BASQ3NZZP75Y",
              "children": [
                "01K1GRZY9WW8S6Y8ZAMXHNWCQF"
              ]
            },
            "01K1GRZY5MQ278E8C2BTTHP465": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72V",
              "children": []
            },
            "01K1GRZY62MM7VXETJEYDW54C8": {
              "parent": "01K1GRZY619065Q4CDPS3E6J6X",
              "children": []
            },
            "01K1GRZY5Y4413D1DVM8F1930K": {
              "parent": "01K1GRZY5XQTNDJ3EX1G6V35YF",
              "children": [
                "01K1GRZY5Y4413D1DVM8F1930Q",
                "01K1GRZY5ZT0S3ZMA2DRCHZ6Y1"
              ]
            },
            "01K1GRZY5NTQ50N9DWPH41BZXJ": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72V",
              "children": []
            },
            "01K1GRZY619065Q4CDPS3E6J6X": {
              "parent": "01K1GRZY60KF5D9NYEMWC5JJ8K",
              "children": [
                "01K1GRZY619065Q4CDPS3E6J71",
                "01K1GRZY62MM7VXETJEYDW54C8"
              ]
            },
            "01K1GRZY9WW8S6Y8ZAMXHNWCQF": {
              "parent": "01K1GRZY9VHJBXW2N31E6V740N",
              "children": [
                "01K1GRZY9XD1C408ZSSMRCKHFS",
                "01K1GRZY9XD1C408ZSSMRCKHFV"
              ]
            },
            "01K1GRZY9YYTVYBAV7ER36CPT7": {
              "parent": "01K1GRZY5Q4NK3BASQ3NZZP75Y",
              "children": [
                "01K1GRZY9Z29H6N0T3V5MQE1K1",
                "01K1GRZY9Z29H6N0T3V5MQE1K3",
                "01K1GRZYA0VXMVD85TQ223ZYBQ",
                "01K1GRZYA1Z5GV4M5AHFTFB7ST"
              ]
            },
            "01K1GRZY9T0T4EPR44MNQ3RGYK": {
              "parent": "01K1GRZY5Q4NK3BASQ3NZZP75Y",
              "children": []
            },
            "01K1GRZY5XQTNDJ3EX1G6V35YF": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72Q",
              "children": [
                "01K1GRZY5Y4413D1DVM8F1930K"
              ]
            },
            "01K1GRZY9Z29H6N0T3V5MQE1K1": {
              "parent": "01K1GRZY9YYTVYBAV7ER36CPT7",
              "children": []
            },
            "01K1GRZYA5V8SP1C65Z3TV2QA0": {
              "parent": "01K1GRZY5Q4NK3BASQ3NZZP762",
              "children": []
            },
            "01K1GRZYA380X39C7AQA2A1K5F": {
              "parent": "01K1GRZY5W8M93YV376R66D3HB",
              "children": []
            },
            "01K1GRZY619065Q4CDPS3E6J71": {
              "parent": "01K1GRZY619065Q4CDPS3E6J6X",
              "children": []
            },
            "01K1GRZY5W8M93YV376R66D3H9": {
              "parent": "01K1GRZY5RBGQFXZH6ZNMJFSYB",
              "children": []
            },
            "01K1GRZY9Z29H6N0T3V5MQE1K3": {
              "parent": "01K1GRZY9YYTVYBAV7ER36CPT7",
              "children": []
            },
            "01K1GRZY5Y4413D1DVM8F1930Q": {
              "parent": "01K1GRZY5Y4413D1DVM8F1930K",
              "children": []
            },
            "01K1GRZY5SF3B9K6XZ0S6G0BE0": {
              "parent": "01K1GRZY5RBGQFXZH6ZNMJFSYB",
              "children": []
            },
            "01K1GRZY5J8MJ6KABW40PEX72Q": {
              "parent": null,
              "children": [
                "01K1GRZY5J8MJ6KABW40PEX72V",
                "01K1GRZY5Q4NK3BASQ3NZZP75Y",
                "01K1GRZY5Q4NK3BASQ3NZZP762",
                "01K1GRZY5RBGQFXZH6ZNMJFSYB",
                "01K1GRZY5W8M93YV376R66D3HB",
                "01K1GRZY5XQTNDJ3EX1G6V35YF",
                "01K1GRZY60KF5D9NYEMWC5JJ8K",
                "01K1GRZY637AK0GW8N1A52P4FZ"
              ]
            },
            "01K1GRZY5Q4NK3BASQ3NZZP762": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72Q",
              "children": [
                "01K1GRZYA5V8SP1C65Z3TV2QA0"
              ]
            },
            "01K1GRZYA1Z5GV4M5AHFTFB7ST": {
              "parent": "01K1GRZY9YYTVYBAV7ER36CPT7",
              "children": []
            },
            "01K1GRZY5ZT0S3ZMA2DRCHZ6Y1": {
              "parent": "01K1GRZY5Y4413D1DVM8F1930K",
              "children": []
            },
            "01K1GRZYA0VXMVD85TQ223ZYBQ": {
              "parent": "01K1GRZY9YYTVYBAV7ER36CPT7",
              "children": []
            },
            "01K1GRZY5TTHRK378EK8Y9DKJ2": {
              "parent": "01K1GRZY5RBGQFXZH6ZNMJFSYB",
              "children": []
            },
            "01K1GRZY5NTQ50N9DWPH41BZXG": {
              "parent": "01K1GRZY5J8MJ6KABW40PEX72V",
              "children": []
            }
          }
        }
      }
`;
const small = `
{
        "id": "01JYPVD3A6J86TABR82FK3JHWE",
        "actionPrototypes": [
          {
            "id": "01JYPVD3K1C9GZ8WBQP7X8NVQH",
            "funcId": "01JYPVD391E3W3087NA2TAP20Y",
            "kind": "Manual",
            "displayName": "Update Source Dest Check Action",
            "name": "si:awsEc2InstanceModifySourceDestCheckAction"
          },
          {
            "id": "01JYPVD3K0H6HXAFHYDE8AH2HW",
            "funcId": "01JYPVD39ET5VFGN3080Y3387R",
            "kind": "Create",
            "displayName": "Create EC2 Instance",
            "name": "si:awsEc2CreateAction"
          },
          {
            "id": "01JYPVD3JZMBH4SCM5XY6NPXYD",
            "funcId": "01JYPVD38HPKGNDG7GMWM5NGAA",
            "kind": "Manual",
            "displayName": "Stop Ec2 Instance",
            "name": "si:awsEc2InstanceStopAction"
          },
          {
            "id": "01JYPVD3JY7T4R2FQBMRPP8WTR",
            "funcId": "01JYPVD38DRAWXQ3FYFD3V0HR4",
            "kind": "Manual",
            "displayName": "Modify instance metadata options",
            "name": "si:awsEc2ModifyInstanceMetadataOptions"
          },
          {
            "id": "01JYPVD3JX7N3XP1NG7VNQS2E9",
            "funcId": "01JYPVD396QNGSRBBB7SDHT5WK",
            "kind": "Destroy",
            "displayName": "Delete EC2 Instance",
            "name": "si:awsEc2DeleteAction"
          },
          {
            "id": "01JYPVD3JWF85M7ZNH0FE5TAP7",
            "funcId": "01JYPVD38PVEHQJ3N0YGFXCY6S",
            "kind": "Manual",
            "displayName": "Start Ec2 Instance",
            "name": "si:awsEc2InstanceStartAction"
          },
          {
            "id": "01JYPVD3JVE6016FNSE3DGVENX",
            "funcId": "01JYPVD38VA1HVGBGE2FS7GSVF",
            "kind": "Manual",
            "displayName": "Reboot Ec2 Instance",
            "name": "si:awsEc2InstanceRebootAction"
          },
          {
            "id": "01JYPVD3JTMNPJKCW1N1XA04SR",
            "funcId": "01JYPVD36KF5PQ6XFMXZDDY773",
            "kind": "Manual",
            "displayName": "Associate the instance profile",
            "name": "updateInstanceProfile"
          },
          {
            "id": "01JYPVD3JSMC752MJ3A42H7EHM",
            "funcId": "01JYPVD3883W715S5N3F9WCVMW",
            "kind": "Refresh",
            "displayName": "Refresh EC2 Instance's Resource",
            "name": "si:awsEc2RefreshAction"
          }
        ]
      }
`;
const small2 = `
{
      "kind": "ComponentInList",
      "id": "01K1GRZHEKA8EK4RVC8X2M8N6Y",
      "checksum": "2323f40fb743048f3171ab038dc50f2c",
      "data": {
        "id": "01K1GRZHEKA8EK4RVC8X2M8N6Y",
        "name": "si-5540",
        "color": "#FF9900",
        "schemaName": "AWS::EC2::Subnet",
        "schemaId": "01JK0QZHF6ZSY6JFM39MBCRJ4C",
        "schemaVariantId": "01K1GRZEX1QAB58FG6Q2C38VS5",
        "schemaVariantName": "AWS::EC2::Subnet",
        "schemaCategory": "AWS::EC2",
        "hasResource": false,
        "resourceId": null,
        "qualificationTotals": {
          "total": 2,
          "warned": 0,
          "succeeded": 2,
          "failed": 0
        },
        "inputCount": 3,
        "diffStatus": "None",
        "toDelete": false,
        "hasSocketConnections": false
      }
    }
`;
const small3 = `
{
      "kind": "Component",
      "id": "01K1GS00KRGAP3VAAMTN9NQNZR",
      "checksum": "ef31d4833b7af0345ca41689784e8709",
      "data": {
        "id": "01K1GS00KRGAP3VAAMTN9NQNZR",
        "name": "si-2742",
        "color": "#FF9900",
        "schemaName": "AWS::EC2::RouteTable",
        "schemaId": "01JK0RQN1A2WMW5QRD0SNJS40A",
        "schemaVariantId": {
          "kind": "SchemaVariant",
          "id": "01K1GRZY5HYEV4FN3SG8803EEA"
        },
        "schemaMembers": {
          "kind": "SchemaMembers",
          "id": "01JK0RQN1A2WMW5QRD0SNJS40A"
        },
        "schemaVariantName": "AWS::EC2::RouteTable",
        "schemaVariantDescription": "Specifies a route table for the specified VPC. After you create a route table, you can add routes and associate the table with a subnet. For more information, see [Route tables](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_Route_Tables.html) in the *Amazon VPC User Guide*.",
        "schemaVariantDocLink": "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-routetable.html",
        "schemaCategory": "AWS::EC2",
        "hasResource": false,
        "qualificationTotals": {
          "total": 2,
          "warned": 0,
          "succeeded": 2,
          "failed": 0
        },
        "inputCount": 3,
        "resourceDiff": {
          "current": "",
          "diff": null
        },
        "isSecretDefining": false,
        "toDelete": false
      }
    }
`;

const opts = [large, small, small2, small3];
const checksums = await Promise.all(
  opts.map(async (doc) => {
    const encoder = new TextEncoder();
    const buffer = encoder.encode(doc);
    const hashBuffer = await window.crypto.subtle.digest("SHA-256", buffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const checksum = hashArray
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    return checksum;
  }),
);

const _docData = (): [string, string] => {
  const idx = Math.floor(Math.random() * opts.length);
  const doc = opts[idx];
  const checksum = checksums[idx];
  if (!doc) throw new Error("nope");
  if (!checksum) throw new Error("nope2");
  return [doc, checksum];
};

const _makeComponents = async (count: number): Promise<AtomWithDocument[]> => {
  // a component makes all these MVs
  const entities = [
    EntityKind.Component,
    EntityKind.ComponentInList,
    EntityKind.ComponentDiff,
    EntityKind.IncomingConnections,
    EntityKind.SchemaVariant,
    EntityKind.ActionPrototypeViewList,
    EntityKind.AttributeTree,
  ];
  const atoms = await Promise.all(
    [...Array(count + 1).keys()].flatMap(() => {
      const id = ulid();
      return entities.map(async (kind) => {
        const [doc, checksum] = await _docData();

        const atom: AtomWithDocument = {
          doc,
          kind,
          id,
          checksum,
        };
        return atom;
      });
    }),
  );
  return atoms;
};

const sleep = (ms: number) => {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
};

const asyncTimeit = async (
  fn: () => Promise<void>,
  count: number,
  total: number,
  log: (str: string) => void,
) => {
  const start = performance.now();
  await fn();
  const end = performance.now();
  const ms = end - start;
  log(`${count} @ ${total} took ${ms.toFixed(3)}ms`);
};

const _clearSome = async (db: Comlink.Remote<TabDBInterface>) => {
  // magic that deletes % of the db, bc random() is a 64bit int
  const perc = 2 ** 63 * 0.6;
  await db.exec({
    sql: `DELETE FROM atoms WHERE random() > ?`,
    bind: [perc],
  });
};

const doIO = true;
const readAtoms = async (
  db: Comlink.Remote<TabDBInterface>,
  atoms: AtomWithDocument[],
  total: number,
) => {
  if (atoms.length === 0) {
    reads("none to read");
  }
  // throwing all reads at once onto the event loop
  // because we don't have that level of control
  const start = performance.now();
  await Promise.all(
    atoms.map(async (atom) => {
      // await sleep(1000);
      await asyncTimeit(
        async () => {
          if (doIO) {
            const _mv = await db.get(
              workspaceId,
              changeSetId,
              atom.kind as Gettable,
              atom.id,
            );
            // console.log("mv size", new TextEncoder().encode(_mv).byteLength);
          } else {
            // no IO
            //  - 100ms faster
            // no MTM
            //  - 50% faster
            await db.exec({
              sql: `
        select
          data
        from
          atoms
        where
          AND atoms.kind = ?
          AND atoms.args = ?
        `,
              bind: [changeSetId, atom.kind, atom.id],
            });
          }
        },
        1,
        total,
        reads,
      );
    }),
  );
  const end = performance.now();
  const ms = end - start;
  reads(`batch perf took ${ms.toFixed(3)}ms`);
};

let perfTestRunStart: number;
const fullPerfTest = async (db: Comlink.Remote<TabDBInterface>) => {
  const indexChecksum = ulid();
  db.exec({
    sql: "INSERT INTO indexes (checksum) values (?)",
    bind: [indexChecksum],
  });

  db.exec({
    sql: "INSERT INTO changesets (change_set_id, workspace_id, index_checksum) VALUES (?, ?, ?)",
    bind: [changeSetId, workspaceId, indexChecksum],
  });

  const howMany = await db.exec({
    sql: "SELECT COUNT(*) FROM atoms",
    returnValue: "resultRows",
  });
  // eslint-disable-next-line no-console
  console.assert(howMany[0]?.[0] === 0, "ATOMS IS NOT EMPTY");
  const runs = [10, 50, 250, 500, 1_000, 2_000];
  const cache: AtomWithDocument[] = [];

  perfTestRunStart = performance.now();

  // fill the DB
  // if you want to do everything all at once
  // remove some of the await before `asyncTimeIt`
  // and for extra spicy things, do `runs.map` rather than a for loop
  // for (const n of runs) {
  runs.map(async (n) => {
    writes(`load ${n}`);
    const atoms = await _makeComponents(n);
    atoms.forEach((a) => {
      cache.push(a);
    });
    asyncTimeit(
      async () => {
        await db.bulkCreateAtoms(atoms);
      },
      n,
      totalDbCount,
      writes,
    );
    asyncTimeit(
      async () => {
        await db.bulkInsertAtomMTMs(atoms, indexChecksum);
      },
      n,
      totalDbCount,
      writes,
    );
    totalDbCount += atoms.length;

    reads(`select batch-${n}`);
    readAtoms(db, atoms.slice(0, 10), totalDbCount);
  });

  // are smaller or larger inserts effected by data already in the db?
  // also, sleep in between to see if that helps perf at all
  // for (const n of [...runs].reverse()) {
  [...runs].reverse().map(async (n) => {
    // await sleep(1000);
    writes(`reverse ${n}`);
    const atoms = await _makeComponents(n);
    asyncTimeit(
      async () => {
        await db.bulkCreateAtoms(atoms);
      },
      n,
      totalDbCount,
      writes,
    );
    asyncTimeit(
      async () => {
        await db.bulkInsertAtomMTMs(atoms, indexChecksum);
      },
      n,
      totalDbCount,
      writes,
    );
    totalDbCount += atoms.length;
  });

  const cache2 = [...cache];
  // now lets upsert half the db!
  // use the cached atoms, change the doc and checksum
  // this forces UPDATE
  // for (const n of runs) {
  runs.map(async (n) => {
    const atoms = cache.splice(0, n).map((a) => {
      return {
        ...a,
        checksum: `${a.checksum}-different`,
        doc: `${a.doc}-different`,
      };
    });

    writes(`upsert ${n}`);
    asyncTimeit(
      async () => {
        await db.bulkCreateAtoms(atoms);
      },
      n,
      totalDbCount,
      writes,
    );
    asyncTimeit(
      async () => {
        await db.bulkInsertAtomMTMs(atoms, indexChecksum);
      },
      n,
      totalDbCount,
      writes,
    );
  });

  // do it again, but sleep in between each run!
  // for (const n of runs) {
  runs.map(async (n) => {
    await sleep(1000);
    const atoms = cache2.splice(0, n).map((a) => {
      return {
        ...a,
        checksum: `${a.checksum}-again`,
        doc: `${a.doc}-again`,
      };
    });

    writes(`upsert sleep ${n}`);
    asyncTimeit(
      async () => {
        await db.bulkCreateAtoms(atoms);
      },
      n,
      totalDbCount,
      writes,
    );
    asyncTimeit(
      async () => {
        await db.bulkInsertAtomMTMs(atoms, indexChecksum);
      },
      n,
      totalDbCount,
      writes,
    );
  });

  // now lets select all the records of a specific kind
  /*
  const cnt = runs.reduce((n, c) => n + c);
  const kinds = [
    EntityKind.Component,
    EntityKind.SchemaVariant,
    EntityKind.AttributeTree,
  ];
  await Promise.all(
    kinds.map(async (kind) => {
      await asyncTimeit(
        async () => {
          await db.exec({
            sql: `
        select
          data
        from
          atoms
          inner join index_mtm_atoms mtm
            ON atoms.kind = mtm.kind AND atoms.args = mtm.args AND atoms.checksum = mtm.checksum
          inner join indexes ON mtm.index_checksum = indexes.checksum
          inner join changesets ON changesets.index_checksum = indexes.checksum
        where
          changesets.change_set_id = ?
          AND
          atoms.kind = ?
        `,
            bind: [changeSetId, kind],
          });
        },
        cnt,
        totalDbCount,
        reads,
      );
    }),
  );
  */
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
  db.setBearer(workspaceId, "doesnt matter 123");
  try {
    await fullPerfTest(db);
  } catch (e: unknown) {
    // eslint-disable-next-line no-console
    console.error(e);
    if (e instanceof Error) {
      // eslint-disable-next-line no-console
      console.assert(false, e.toString());
    } else {
      // eslint-disable-next-line no-console
      console.assert(false, e as string);
    }
  } finally {
    done();
  }
}

go();
