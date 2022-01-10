import _ from "lodash";

import { Schematic } from "./schematic";
import {
  Node,
  QualificationStatus,
  ResourceStatus,
  ActionStatus,
  ComponentType,
} from "./node";

function generateUniqueId() {
  return _.uniqueId();
}

const nodeA: Node = {
  id: "A:1",
  label: {
    title: "k8s service",
    name: "whiskers",
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
  position: [
    {
      id: generateUniqueId(),
      x: 300,
      y: 100
    },
  ],
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
    name: "dev",
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
  position: [
    {
      id: "B:1.S:1",
      x: 100,
      y: 100
    },
  ],
  input: [],
  output: [
    {
      id: generateUniqueId(),
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

export const schematicData: Schematic = {
  nodes: [nodeA, nodeB],
  connections: [],
  lastUpdated: new Date(Date.now()),
  checksum: "i5i5i55i5i5i5i55i5i5i",
};
