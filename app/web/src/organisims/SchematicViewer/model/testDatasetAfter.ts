import _ from "lodash";

import { Schematic } from "./schematic";
import {
  Node,
  QualificationStatus,
  ResourceStatus,
  ActionStatus,
  ComponentType,
} from "./node";

import { Connection, ConnectionKind } from "./connection";

function generateUniqueId() {
  return _.uniqueId();
}

const nodeA: Node = {
  id: "A:1",
  label: {
    title: "k8s service",
    name: "whiskers A",
  },
  classification: {
    component: ComponentType.APPLICATION,
    kind: "kubernetes",
    type: "service",
  },
  status: {
    qualification: QualificationStatus.SUCCEEDED,
    resource: ResourceStatus.HEALTHY,
    changeCount: 3,
    action: {
      name: "aaa",
      timestamp: new Date(Date.now()),
      status: ActionStatus.SUCCEEDED,
    },
  },
  position: {
    ctx: [
      {
        id: generateUniqueId(),
        position: {
          x: 300,
          y: 100,
        },
      },
    ],
  },
  input: [
    {
      id: "A:1.S:1",
      type: "kubernetes.namespace",
      name: "namespace",
    },
    {
      id: "A:1.S:2",
      type: "kubernetes.deployment",
      name: "deployment",
    },
    {
      id: "A:1.S:3",
      type: "kubernetes.service",
      name: "service",
    },
    {
      id: "A:1.S:4",
      type: "kubernetes.env",
      name: "env",
    },
  ],
  output: [
    {
      id: "A:1.S:5",
      type: "kubernetes.service",
    },
  ],
  display: {
    color: 0x32b832,
  },
  connections: [],
  lastUpdated: new Date(Date.now()),
  checksum: "j4j4j4j4j4j4j4j4j4j4j4",
  schematic: {
    deployment: false,
    component: true,
  },
};

const nodeB: Node = {
  id: "B:1",
  label: {
    title: "k8s namespace",
    name: "dev A",
  },
  classification: {
    component: ComponentType.APPLICATION,
    kind: "kubernetes",
    type: "namespace",
  },
  status: {
    qualification: QualificationStatus.SUCCEEDED,
    resource: ResourceStatus.HEALTHY,
    changeCount: 0,
    action: {
      name: "aaa",
      timestamp: new Date(Date.now()),
      status: ActionStatus.SUCCEEDED,
    },
  },
  position: {
    ctx: [
      {
        id: generateUniqueId(),
        position: { x: 100, y: 100 },
      },
    ],
  },
  input: [],
  output: [
    {
      id: "B:1.S:1",
      type: "kubernetes.namespace",
    },
  ],
  display: {
    color: 0x3251b8,
  },
  connections: [],
  lastUpdated: new Date(Date.now()),
  checksum: "j4j4j4j4j4j4j4j4j4j4j4",
  schematic: {
    deployment: false,
    component: true,
  },
};

const conn: Connection = {
  id: generateUniqueId(),
  classification: {
    kind: ConnectionKind.DEPLOYMENT,
  },
  source: {
    nodeId: nodeB.id,
    socketId: nodeB.output[0].id,
  },
  destination: {
    nodeId: nodeA.id,
    socketId: nodeA.input[0].id,
  },
  lastUpdated: new Date(Date.now()),
  checksum: "j4j4j4j4j4j4j4j4j4j4j4",
  schematic: {
    deployment: false,
    component: true,
  },
};

export const schematicDataAfter: Schematic = {
  nodes: [nodeA, nodeB],
  connections: [conn],
  lastUpdated: new Date(Date.now()),
  checksum: "i5i5i55i5i5i5i55i5i5i",
};
