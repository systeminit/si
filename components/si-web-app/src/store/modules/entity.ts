import { Module } from "vuex";
import _ from "lodash";
import { snakeCase, camelCase } from "change-case";
import { registry, Props, PropMethod, PropLink, PropObject } from "si-registry";

import { generateName } from "@/api/names";
import { graphqlQuery, graphqlMutation } from "@/api/apollo";
import { RootStore } from "@/store";
import { Item, Node } from "./node";
import {
  ServiceEntity,
  Edge,
  NodeNodeKind,
  KubernetesClusterEntity,
  MinikubeEntity,
  ServerEntity,
} from "@/graphql-types";

interface EntityMeta {
  workspaceId: string;
  partial: boolean;
  entity: Entity;
}

export interface Entity {
  id: string;
  name: string;
  description: string;
  siStorable: {
    typeName: string;
    changeSetId: string;
    [key: string]: any;
  };
  siProperties: {
    [key: string]: any;
  };
  properties: {
    [key: string]: any;
  };
  constraints: {
    [key: string]: any;
  };
  implicitConstraints: {
    [key: string]: any;
  };
}

export interface EntityStore {
  entities: Entity[];
}

export interface EntityProperty {
  path: (string | number)[];
  prop: Props;
  name: string;
  label: string;
  required: boolean;
  repeated: boolean;
  kind: string;
  hidden: boolean;
}

interface AddMutation {
  entities: Entity[];
}

interface DeleteEntityAction {
  typeName: string;
  id: string;
}

interface CreateEntityPayload {
  typeName: string;
  data?: {
    name?: string;
    [key: string]: any;
  };
}

interface UpdateEntityPayload {
  typeName: string;
  data: {
    name?: string;
    description?: string;
    displayName?: string;
    [field: string]: any;
  };
  hypotheticalState?: {
    path: string[];
    value: any;
  };
}

export const entity: Module<EntityStore, RootStore> = {
  namespaced: true,
  state: {
    entities: [],
  },
  mutations: {
    add(state, payload: AddMutation) {
      state.entities = _.unionBy(payload.entities, state.entities, "id");
    },
  },
  getters: {
    // prettier-ignore
    get: (state) => (filter: any): Entity => {
      const result = _.find(state.entities, filter);
      if (result) {
        return result;
      } else {
        throw new Error(`Cannot get entity for entity with filter: ${JSON.stringify(filter)}`);
      }
    },
    // Prettier cannot handle the glory of this syntax. Bow before the functions.
    // prettier-ignore
    optionsFromType: (state, getters, rootState, rootGetters) => (camelFromType: string, ): {key: string; value: string}[] => {
      const fromType = snakeCase(camelFromType);
      const all: Entity[] = _.filter(state.entities, ["siStorable.typeName", fromType]);
      let inChangeSet: Entity[];
      if (rootState.changeSet.current) {
        inChangeSet = _.filter(all, (entity, _index, collection) => {
          if (!entity.siStorable.changeSetId || entity.siStorable.changeSetId == rootState.changeSet?.current?.id) {
            return true;
          } else {
            return false;
          }
        });
      } else {
        inChangeSet = _.filter(all, (entity) => {
          if (!entity.siStorable.changeSetId) {
            return true
          } else {
            return false;
          }
        });
      }

      const results = _.uniqBy(
        _.map(
          _.orderBy(inChangeSet, ["siStorable.changeSetEntryCount"], ["desc"]),
          (entity) => {
            return {
              key: entity.name,
              value: entity.siStorable.itemId || entity.id
            }
          }
        ),
        'value'
      );
      return results;
    }
  },
  actions: {
    async serverIntelligence(
      { state, rootState, dispatch, rootGetters },
      { entity }: { entity: ServerEntity },
    ): Promise<void> {
      dispatch("serverIntelligenceSoftware", { entity });
      dispatch("serverIntelligenceHardware", { entity });
    },
    async serverIntelligenceSoftware(
      { state, rootState, dispatch, rootGetters },
      { entity }: { entity: ServerEntity },
    ): Promise<void> {
      let targetType = `ubuntu_entity`;

      // It just needs to create a cluster! ha!
      let smartNode = rootGetters["node/getNodeByEntityId"](entity.id);
      if (!smartNode) {
        smartNode = rootGetters["node/getNodeByEntityId"](
          entity.siStorable?.itemId,
        );
      }
      const smartEdges: Edge[] = rootGetters["edge/filter"]((edge: Edge) => {
        if (
          edge.tailVertex?.id == smartNode.id &&
          edge.headVertex?.typeName == targetType
        ) {
          return true;
        } else {
          return false;
        }
      });
      // Update
      if (smartEdges.length > 0) {
        for (const edge of smartEdges) {
          let edgeNode = rootGetters["node/getNodeById"](edge.headVertex?.id);
          if (edgeNode) {
            // Kubernetes Deployment Entity Update
            if (edge.headVertex?.typeName == targetType) {
              // Do nothing! :)
            }
          } else {
            console.log(
              "broken edge! the node does not exist, but the edge does!",
              { edge, edgeNode },
            );
          }
        }
      } else {
        const currentSystem = rootGetters["system/current"];
        const systemEdges = rootGetters["edge/allRelatedEdges"](
          currentSystem.id,
        );
        const possibleTargetEdges: Edge[] = _.filter(systemEdges, edge => {
          if (edge.headVertex.typeName == targetType) {
            return true;
          } else {
            return false;
          }
        });

        let targetNode: Node | undefined;

        if (possibleTargetEdges) {
          // Find the correct target node, since one might exist
          for (const possibleTargetEdge of possibleTargetEdges) {
            const possibleTargetNode = rootGetters["node/getNodeById"](
              possibleTargetEdge.headVertex?.id,
            );
            if (
              rootState.changeSet.current?.id &&
              possibleTargetNode.display[rootState.changeSet.current.id]
            ) {
              targetNode = possibleTargetNode;
              break;
            } else if (possibleTargetNode.display["saved"]) {
              targetNode = possibleTargetNode;
              break;
            }
          }
        }
        if (!targetNode) {
          // Create the new target node, since one does not exist
          let newEntityPayload: CreateEntityPayload = {
            typeName: camelCase(targetType),
            data: {
              name: `${currentSystem.name}-ubuntu`,
              description: `${currentSystem.name}-ubuntu`,
              properties: {
                version: "20.04 LTS",
              },
            },
          };
          const createdEntity = await dispatch("create", newEntityPayload);
          targetNode = rootGetters["node/getNodeByEntityId"](
            createdEntity.siStorable?.itemId,
          );
          let nodeSizeY = 100;
          let nodeOffsetY = nodeSizeY * 0.5;
          let nodeOffsetX = 0;

          await dispatch(
            "node/setNodePosition",
            {
              position: {
                x: smartNode.position.x + nodeOffsetX,
                y: smartNode.position.y + nodeSizeY + nodeOffsetY,
              },
              id: targetNode?.id,
            },
            { root: true },
          );
        }
        await dispatch(
          "edge/create",
          {
            tailVertex: {
              id: smartNode.id,
              socket: "output",
              typeName: smartNode.stack[0].siStorable?.typeName,
            },
            headVertex: {
              id: targetNode?.id,
              socket: "input",
              typeName: targetNode?.stack[0].siStorable?.typeName,
            },
            bidirectional: true,
          },
          { root: true },
        );
      }
    },
    async serverIntelligenceHardware(
      { state, rootState, dispatch, rootGetters },
      { entity }: { entity: ServerEntity },
    ): Promise<void> {
      let targetType = `ec2_instance_entity`;

      // It just needs to create a cluster! ha!
      let smartNode = rootGetters["node/getNodeByEntityId"](entity.id);
      if (!smartNode) {
        smartNode = rootGetters["node/getNodeByEntityId"](
          entity.siStorable?.itemId,
        );
      }
      const smartEdges: Edge[] = rootGetters["edge/filter"]((edge: Edge) => {
        if (
          edge.tailVertex?.id == smartNode.id &&
          edge.headVertex?.typeName == targetType
        ) {
          return true;
        } else {
          return false;
        }
      });
      // Update
      if (smartEdges.length > 0) {
        for (const edge of smartEdges) {
          let edgeNode = rootGetters["node/getNodeById"](edge.headVertex?.id);
          if (edgeNode) {
            // Kubernetes Deployment Entity Update
            if (edge.headVertex?.typeName == targetType) {
              // Do nothing! :)
            }
          } else {
            console.log(
              "broken edge! the node does not exist, but the edge does!",
              { edge, edgeNode },
            );
          }
        }
      } else {
        const currentSystem = rootGetters["system/current"];
        const systemEdges = rootGetters["edge/allRelatedEdges"](
          currentSystem.id,
        );
        const possibleTargetEdges: Edge[] = _.filter(systemEdges, edge => {
          if (edge.headVertex.typeName == targetType) {
            return true;
          } else {
            return false;
          }
        });

        let targetNode: Node | undefined;

        if (possibleTargetEdges) {
          // Find the correct target node, since one might exist
          for (const possibleTargetEdge of possibleTargetEdges) {
            const possibleTargetNode = rootGetters["node/getNodeById"](
              possibleTargetEdge.headVertex?.id,
            );
            if (
              rootState.changeSet.current?.id &&
              possibleTargetNode.display[rootState.changeSet.current.id]
            ) {
              targetNode = possibleTargetNode;
              break;
            } else if (possibleTargetNode.display["saved"]) {
              targetNode = possibleTargetNode;
              break;
            }
          }
        }
        if (!targetNode) {
          // Create the new target node, since one does not exist
          let newEntityPayload: CreateEntityPayload = {
            typeName: camelCase(targetType),
            data: {
              name: `${currentSystem.name}-ec2-instance`,
              description: `${currentSystem.name}-ec2-instance`,
              properties: {
                region: "us-east-2",
                instanceType: "t2.micro",
              },
            },
          };
          const createdEntity = await dispatch("create", newEntityPayload);
          targetNode = rootGetters["node/getNodeByEntityId"](
            createdEntity.siStorable?.itemId,
          );
          let nodeSizeY = 100;
          let nodeOffsetY = nodeSizeY * 0.5;
          let nodeOffsetX = 0;

          await dispatch(
            "node/setNodePosition",
            {
              position: {
                x: smartNode.position.x + nodeOffsetX,
                y: smartNode.position.y + nodeSizeY + nodeOffsetY,
              },
              id: targetNode?.id,
            },
            { root: true },
          );
        }
        await dispatch(
          "edge/create",
          {
            tailVertex: {
              id: smartNode.id,
              socket: "output",
              typeName: smartNode.stack[0].siStorable?.typeName,
            },
            headVertex: {
              id: targetNode?.id,
              socket: "input",
              typeName: targetNode?.stack[0].siStorable?.typeName,
            },
            bidirectional: true,
          },
          { root: true },
        );
      }
    },
    async minikubeIntelligence(
      { state, rootState, dispatch, rootGetters },
      { entity }: { entity: MinikubeEntity },
    ): Promise<void> {
      const targetType = `server_entity`;

      // It just needs to create a cluster! ha!
      let smartNode = rootGetters["node/getNodeByEntityId"](entity.id);
      if (!smartNode) {
        smartNode = rootGetters["node/getNodeByEntityId"](
          entity.siStorable?.itemId,
        );
      }
      const smartEdges: Edge[] = rootGetters["edge/filter"]((edge: Edge) => {
        if (
          edge.tailVertex?.id == smartNode.id &&
          edge.headVertex?.typeName == targetType
        ) {
          return true;
        } else {
          return false;
        }
      });
      // Update
      if (smartEdges.length > 0) {
        for (const edge of smartEdges) {
          let edgeNode = rootGetters["node/getNodeById"](edge.headVertex?.id);
          if (edgeNode) {
            // Kubernetes Deployment Entity Update
            if (edge.headVertex?.typeName == targetType) {
              // Do nothing! :)
            }
          } else {
            console.log(
              "broken edge! the node does not exist, but the edge does!",
              { edge, edgeNode },
            );
          }
        }
      } else {
        const currentSystem = rootGetters["system/current"];
        const systemEdges = rootGetters["edge/allRelatedEdges"](
          currentSystem.id,
        );
        const possibleTargetEdges: Edge[] = _.filter(systemEdges, edge => {
          if (edge.headVertex.typeName == targetType) {
            return true;
          } else {
            return false;
          }
        });

        let targetNode: Node | undefined;

        if (possibleTargetEdges) {
          // Find the correct target node, since one might exist
          for (const possibleTargetEdge of possibleTargetEdges) {
            const possibleTargetNode = rootGetters["node/getNodeById"](
              possibleTargetEdge.headVertex?.id,
            );
            if (
              rootState.changeSet.current?.id &&
              possibleTargetNode.display[rootState.changeSet.current.id]
            ) {
              targetNode = possibleTargetNode;
              break;
            } else if (possibleTargetNode.display["saved"]) {
              targetNode = possibleTargetNode;
              break;
            }
          }
        }
        if (!targetNode) {
          // Create the new target node, since one does not exist
          let newEntityPayload: CreateEntityPayload = {
            typeName: camelCase(targetType),
            data: {
              name: `${currentSystem.name}-minikube-server`,
              description: `${currentSystem.name}-minikube-server`,
              properties: {
                operatingSystem: "Ubuntu 20.04 LTS",
                cpu: "Intel x86_64",
                memory: "1GiB",
              },
            },
          };
          const createdEntity = await dispatch("create", newEntityPayload);
          targetNode = rootGetters["node/getNodeByEntityId"](
            createdEntity.siStorable?.itemId,
          );
          let nodeSizeY = 100;
          let nodeOffsetY = nodeSizeY * 0.5;
          let nodeOffsetX = 0;

          await dispatch(
            "node/setNodePosition",
            {
              position: {
                x: smartNode.position.x + nodeOffsetX,
                y: smartNode.position.y + nodeSizeY + nodeOffsetY,
              },
              id: targetNode?.id,
            },
            { root: true },
          );
        }
        await dispatch(
          "edge/create",
          {
            tailVertex: {
              id: smartNode.id,
              socket: "output",
              typeName: smartNode.stack[0].siStorable?.typeName,
            },
            headVertex: {
              id: targetNode?.id,
              socket: "input",
              typeName: targetNode?.stack[0].siStorable?.typeName,
            },
            bidirectional: true,
          },
          { root: true },
        );
      }
    },
    async kubernetesClusterIntelligence(
      { state, rootState, dispatch, rootGetters },
      { entity }: { entity: KubernetesClusterEntity },
    ): Promise<void> {
      const targetType = `${entity.properties?.class || "minikube"}_entity`;

      // It just needs to create a cluster! ha!
      let smartNode = rootGetters["node/getNodeByEntityId"](entity.id);
      if (!smartNode) {
        smartNode = rootGetters["node/getNodeByEntityId"](
          entity.siStorable?.itemId,
        );
      }
      const smartEdges: Edge[] = rootGetters["edge/filter"]((edge: Edge) => {
        if (
          edge.tailVertex?.id == smartNode.id &&
          edge.headVertex?.typeName == targetType
        ) {
          return true;
        } else {
          return false;
        }
      });
      // Update
      if (smartEdges.length > 0) {
        for (const edge of smartEdges) {
          let edgeNode = rootGetters["node/getNodeById"](edge.headVertex?.id);
          if (edgeNode) {
            // Kubernetes Deployment Entity Update
            if (edge.headVertex?.typeName == targetType) {
              // Do nothing! :)
            }
          } else {
            console.log(
              "broken edge! the node does not exist, but the edge does!",
              { edge, edgeNode },
            );
          }
        }
      } else {
        const currentSystem = rootGetters["system/current"];
        const systemEdges = rootGetters["edge/allRelatedEdges"](
          currentSystem.id,
        );
        const possibleTargetEdges: Edge[] = _.filter(systemEdges, edge => {
          if (edge.headVertex.typeName == targetType) {
            return true;
          } else {
            return false;
          }
        });

        let targetNode: Node | undefined;

        if (possibleTargetEdges) {
          // Find the correct target node, since one might exist
          for (const possibleTargetEdge of possibleTargetEdges) {
            const possibleTargetNode = rootGetters["node/getNodeById"](
              possibleTargetEdge.headVertex?.id,
            );
            if (
              rootState.changeSet.current?.id &&
              possibleTargetNode.display[rootState.changeSet.current.id]
            ) {
              targetNode = possibleTargetNode;
              break;
            } else if (possibleTargetNode.display["saved"]) {
              targetNode = possibleTargetNode;
              break;
            }
          }
        }
        if (!targetNode) {
          // Create the new target node, since one does not exist
          let newEntityPayload: CreateEntityPayload = {
            typeName: camelCase(targetType),
            data: {
              name: `${currentSystem.name}-minikube`,
              description: `${currentSystem.name}-minikube`,
              properties: {
                kubernetesVersion: "v1.18",
                driver: "docker",
              },
            },
          };
          const createdEntity = await dispatch("create", newEntityPayload);
          targetNode = rootGetters["node/getNodeByEntityId"](
            createdEntity.siStorable?.itemId,
          );
          let nodeSizeY = 100;
          let nodeOffsetY = nodeSizeY * 0.5;
          let nodeOffsetX = 0;

          await dispatch(
            "node/setNodePosition",
            {
              position: {
                x: smartNode.position.x + nodeOffsetX,
                y: smartNode.position.y + nodeSizeY + nodeOffsetY,
              },
              id: targetNode?.id,
            },
            { root: true },
          );
        }
        await dispatch(
          "edge/create",
          {
            tailVertex: {
              id: smartNode.id,
              socket: "output",
              typeName: smartNode.stack[0].siStorable?.typeName,
            },
            headVertex: {
              id: targetNode?.id,
              socket: "input",
              typeName: targetNode?.stack[0].siStorable?.typeName,
            },
            bidirectional: true,
          },
          { root: true },
        );
      }
    },
    async kubernetesObjectIntelligence(
      { state, rootState, dispatch, rootGetters },
      { entity }: { entity: ServiceEntity },
    ): Promise<void> {
      // It just needs to create a cluster! ha!
      let smartNode = rootGetters["node/getNodeByEntityId"](entity.id);
      if (!smartNode) {
        smartNode = rootGetters["node/getNodeByEntityId"](
          entity.siStorable?.itemId,
        );
      }
      const smartEdges: Edge[] = rootGetters["edge/filter"]((edge: Edge) => {
        if (
          edge.tailVertex?.id == smartNode.id &&
          edge.headVertex?.typeName == "kubernetes_cluster_entity"
        ) {
          return true;
        } else {
          return false;
        }
      });
      // Update
      if (smartEdges.length > 0) {
        for (const edge of smartEdges) {
          let edgeNode = rootGetters["node/getNodeById"](edge.headVertex?.id);
          if (edgeNode) {
            // Kubernetes Deployment Entity Update
            if (edge.headVertex?.typeName == "kubernetes_cluster_entity") {
              // Do nothing! :)
            }
          } else {
            console.log(
              "broken edge! the node does not exist, but the edge does!",
              { edge, edgeNode },
            );
          }
        }
      } else {
        const currentSystem = rootGetters["system/current"];
        const systemEdges = rootGetters["edge/allRelatedEdges"](
          currentSystem.id,
        );
        const possibleTargetEdges: Edge[] = _.filter(systemEdges, edge => {
          if (edge.headVertex.typeName == "kubernetes_cluster_entity") {
            return true;
          } else {
            return false;
          }
        });

        let targetNode: Node | undefined;

        if (possibleTargetEdges) {
          // Find the correct target node, since one might exist
          for (const possibleTargetEdge of possibleTargetEdges) {
            const possibleTargetNode = rootGetters["node/getNodeById"](
              possibleTargetEdge.headVertex?.id,
            );
            if (
              rootState.changeSet.current?.id &&
              possibleTargetNode.display[rootState.changeSet.current.id]
            ) {
              targetNode = possibleTargetNode;
              break;
            } else if (possibleTargetNode.display["saved"]) {
              targetNode = possibleTargetNode;
              break;
            }
          }
        }
        if (!targetNode) {
          // Create the new target node, since one does not exist
          let newEntityPayload: CreateEntityPayload = {
            typeName: "kubernetesClusterEntity",
            data: {
              name: `${currentSystem.name}-cluster`,
              description: `${currentSystem.name}-cluster`,
              properties: {
                class: "minikube",
              },
            },
          };
          const createdEntity = await dispatch("create", newEntityPayload);
          targetNode = rootGetters["node/getNodeByEntityId"](
            createdEntity.siStorable?.itemId,
          );
          let nodeSizeY = 100;
          let nodeOffsetY = nodeSizeY * 0.5;
          let nodeOffsetX = 0;

          await dispatch(
            "node/setNodePosition",
            {
              position: {
                x: smartNode.position.x + nodeOffsetX,
                y: smartNode.position.y + nodeSizeY + nodeOffsetY,
              },
              id: targetNode?.id,
            },
            { root: true },
          );
        }
        await dispatch(
          "edge/create",
          {
            tailVertex: {
              id: smartNode.id,
              socket: "output",
              typeName: smartNode.stack[0].siStorable?.typeName,
            },
            headVertex: {
              id: targetNode?.id,
              socket: "input",
              typeName: targetNode?.stack[0].siStorable?.typeName,
            },
            bidirectional: true,
          },
          { root: true },
        );
      }
    },
    async serviceEntityIntelligence(
      { state, dispatch, rootGetters },
      { entity }: { entity: ServiceEntity },
    ): Promise<void> {
      if (entity.properties?.deploymentTarget == "none") {
      } else if (entity.properties?.deploymentTarget == "kubernetes") {
        let serviceNode = rootGetters["node/getNodeByEntityId"](entity.id);
        if (!serviceNode) {
          serviceNode = rootGetters["node/getNodeByEntityId"](
            entity.siStorable?.itemId,
          );
        }
        const serviceDeploymentEdges: Edge[] = rootGetters["edge/filter"](
          (edge: Edge) => {
            if (
              edge.tailVertex?.id == serviceNode.id &&
              (edge.headVertex?.typeName == "kubernetes_deployment_entity" ||
                edge.headVertex?.typeName == "kubernetes_service_entity")
            ) {
              return true;
            } else {
              return false;
            }
          },
        );
        if (serviceDeploymentEdges.length > 0) {
          for (const kubeEdge of serviceDeploymentEdges) {
            let kubeNode = rootGetters["node/getNodeById"](
              kubeEdge.headVertex?.id,
            );
            if (kubeNode) {
              // Kubernetes Deployment Entity Update
              if (
                kubeEdge.headVertex?.typeName == "kubernetes_deployment_entity"
              ) {
                for (const pathEntry of [
                  ["name"],
                  ["description"],
                  ["displayName"],
                  ["properties", "kubernetesObject", "metadata", "name"],
                  [
                    "properties",
                    "kubernetesObject",
                    "spec",
                    "template",
                    "spec",
                    "containers",
                    0,
                    "name",
                  ],
                ]) {
                  await dispatch(
                    "node/setFieldValueByNode",
                    {
                      nodeId: kubeNode.id,
                      path: pathEntry,
                      value: `${entity.name}-deployment`,
                    },
                    { root: true },
                  );
                }
                for (const pathEntry of [
                  ["properties", "kubernetesObject", "metadata", "labels", 0],
                  [
                    "properties",
                    "kubernetesObject",
                    "spec",
                    "selector",
                    "matchLabels",
                    0,
                  ],
                  [
                    "properties",
                    "kubernetesObject",
                    "spec",
                    "template",
                    "metadata",
                    "labels",
                    0,
                  ],
                ]) {
                  await dispatch(
                    "node/setFieldValueByNode",
                    {
                      nodeId: kubeNode.id,
                      path: pathEntry,
                      value: { key: "app", value: `${entity.name}-deployment` },
                    },
                    { root: true },
                  );
                }
                await dispatch(
                  "node/setFieldValueByNode",
                  {
                    nodeId: kubeNode.id,
                    path: [
                      "properties",
                      "kubernetesObject",
                      "spec",
                      "replicas",
                    ],
                    value: parseInt(`${entity.properties?.replicas || "0"}`),
                  },
                  { root: true },
                );
                await dispatch(
                  "node/setFieldValueByNode",
                  {
                    nodeId: kubeNode.id,
                    path: [
                      "properties",
                      "kubernetesObject",
                      "spec",
                      "template",
                      "spec",
                      "containers",
                      0,
                      "ports",
                      0,
                      "containerPort",
                    ],
                    value: parseInt(entity.properties.port || "0"),
                  },
                  { root: true },
                );
              } else if (
                kubeEdge.headVertex?.typeName == "kubernetes_service_entity"
              ) {
                for (const pathEntry of [
                  ["name"],
                  ["description"],
                  ["displayName"],
                  ["properties", "kubernetesObject", "metadata", "name"],
                ]) {
                  await dispatch(
                    "node/setFieldValueByNode",
                    {
                      nodeId: kubeNode.id,
                      path: pathEntry,
                      value: `${entity.name}-service`,
                    },
                    { root: true },
                  );
                }
                await dispatch(
                  "node/setFieldValueByNode",
                  {
                    nodeId: kubeNode.id,
                    path: [
                      "properties",
                      "kubernetesObject",
                      "metadata",
                      "labels",
                      0,
                    ],
                    value: { key: "app", value: `${entity.name}-service` },
                  },
                  { root: true },
                );
                await dispatch(
                  "node/setFieldValueByNode",
                  {
                    nodeId: kubeNode.id,
                    path: [
                      "properties",
                      "kubernetesObject",
                      "spec",
                      "ports",
                      0,
                      "name",
                    ],
                    value: `${entity.name}-port`,
                  },
                  { root: true },
                );
                await dispatch(
                  "node/setFieldValueByNode",
                  {
                    nodeId: kubeNode.id,
                    path: [
                      "properties",
                      "kubernetesObject",
                      "spec",
                      "ports",
                      0,
                      "port",
                    ],
                    value: parseInt(entity.properties.port || "0"),
                  },
                  { root: true },
                );
                await dispatch(
                  "node/setFieldValueByNode",
                  {
                    nodeId: kubeNode.id,
                    path: [
                      "properties",
                      "kubernetesObject",
                      "spec",
                      "selector",
                      0,
                    ],
                    value: { key: "app", value: `${entity.name}-deployment` },
                  },
                  { root: true },
                );
              }
            } else {
              console.log(
                "broken edge! the node does not exist, but the edge does!",
                { kubeEdge, kubeNode },
              );
            }
          }
        } else {
          // Create a Kubernetes Deployment if needed, filling in the blanks
          // Create a Kubernetes Service if needed, filling in the blanks
          // Create the edges for both, if needed
          let newEntity: CreateEntityPayload = {
            typeName: "kubernetesDeploymentEntity",
            data: {
              name: `${entity.name}-deployment`,
              description: `${entity.name}-deployment`,
              properties: {
                kubernetesObject: {
                  apiVersion: "apps/v1",
                  kind: "Deployment",
                  metadata: {
                    name: `${entity.name}-deployment`,
                    labels: [
                      { key: "app", value: `${entity.name}-deployment` },
                    ],
                  },
                  spec: {
                    replicas: parseInt(`${entity.properties?.replicas || "0"}`),
                    selector: {
                      matchLabels: [
                        { key: "app", value: `${entity.name}-deployment` },
                      ],
                    },
                    template: {
                      metadata: {
                        labels: [
                          { key: "app", value: `${entity.name}-deployment` },
                        ],
                      },
                      spec: {
                        containers: [
                          {
                            name: `${entity.name}-deployment`,
                            image: `${entity.properties.image}`,
                            ports: [
                              {
                                containerPort: parseInt(
                                  entity.properties.port || "0",
                                ),
                              },
                            ],
                          },
                        ],
                      },
                    },
                  },
                },
              },
            },
          };
          let createdDeployment = await dispatch("create", newEntity);
          let deploymentNode = rootGetters["node/getNodeByEntityId"](
            createdDeployment.id,
          );
          if (!deploymentNode) {
            deploymentNode = rootGetters["node/getNodeByEntityId"](
              createdDeployment.siStorable?.itemId,
            );
          }
          await dispatch(
            "edge/create",
            {
              tailVertex: {
                id: serviceNode.id,
                socket: "output",
                typeName: serviceNode.stack[0].siStorable?.typeName,
              },
              headVertex: {
                id: deploymentNode.id,
                socket: "input",
                typeName: deploymentNode.stack[0].siStorable?.typeName,
              },
              bidirectional: true,
            },
            { root: true },
          );

          let nodeSizeX = 140;
          let nodeSizeY = 100;
          let nodeOffsetY = nodeSizeY * 0.5;
          let nodeOffsetX = nodeSizeX * 0.65;

          await dispatch(
            "node/setNodePosition",
            {
              position: {
                x: serviceNode.position.x + nodeOffsetX,
                y: serviceNode.position.y + nodeSizeY + nodeOffsetY,
              },
              id: deploymentNode.id,
            },
            { root: true },
          );

          // kubernetesService! You're welcome. :)
          let newService: CreateEntityPayload = {
            typeName: "kubernetesServiceEntity",
            data: {
              name: `${entity.name}-service`,
              description: `${entity.name}-service`,
              properties: {
                kubernetesObject: {
                  apiVersion: "v1",
                  kind: "Service",
                  metadata: {
                    name: `${entity.name}-service`,
                    labels: [{ key: "app", value: `${entity.name}-service` }],
                  },
                  spec: {
                    selector: [
                      { key: "app", value: `${entity.name}-deployment` },
                    ],
                    ports: [
                      {
                        name: `${entity.name}-port`,
                        port: parseInt(entity.properties.port || "0"),
                      },
                    ],
                    type: "NodePort",
                  },
                },
              },
            },
          };
          let createdService = await dispatch("create", newService);
          let k8sServiceNode = rootGetters["node/getNodeByEntityId"](
            createdService.id,
          );
          if (!k8sServiceNode) {
            k8sServiceNode = rootGetters["node/getNodeByEntityId"](
              createdService.siStorable?.itemId,
            );
          }
          await dispatch(
            "edge/create",
            {
              tailVertex: {
                id: serviceNode.id,
                socket: "output",
                typeName: serviceNode.stack[0].siStorable?.typeName,
              },
              headVertex: {
                id: k8sServiceNode.id,
                socket: "input",
                typeName: k8sServiceNode.stack[0].siStorable?.typeName,
              },
              bidirectional: true,
            },
            { root: true },
          );

          await dispatch(
            "node/setNodePosition",
            {
              position: {
                x: serviceNode.position.x + -1 * nodeOffsetX,
                y: serviceNode.position.y + nodeSizeY + nodeOffsetY,
              },
              id: k8sServiceNode.id,
            },
            { root: true },
          );
        }
      }
    },
    async update(
      { state, commit, dispatch, rootGetters },
      payload: UpdateEntityPayload,
    ): Promise<void> {
      const variables: Record<string, any> = {
        id: payload.data.id,
        update: {
          name: payload.data.name,
          displayName: payload.data.displayName,
          description: payload.data.description,
          properties: payload.data.properties,
        },
      };
      const workspaceId = rootGetters["workspace/current"].id;
      const changeSetId = rootGetters["changeSet/current"].id;
      variables.changeSetId = changeSetId;
      variables.workspaceId = workspaceId;
      if (variables.update.properties?.kubernetesObjectYaml != undefined) {
        delete variables.update.properties.kubernetesObjectYaml;
      }

      const result = await graphqlMutation({
        typeName: payload.typeName,
        methodName: "update",
        variables,
      });
      const entity = result["item"];
      commit("add", { entities: [entity] });
      let node = {
        entityId: entity.siStorable?.itemId,
        name: entity.name,
        nodeType: NodeNodeKind.Entity,
        object: entity,
      };
      await dispatch(
        "node/add",
        {
          items: [node],
        },
        { root: true },
      );
      if (entity.siStorable?.typeName == "service_entity") {
        await dispatch("serviceEntityIntelligence", { entity });
      } else if (
        entity.siStorable?.typeName == "kubernetes_service_entity" ||
        entity.siStorable?.typeName == "kubernetes_deployment_entity"
      ) {
        await dispatch("kubernetesObjectIntelligence", { entity });
      } else if (entity.siStorable?.typeName == "kubernetes_cluster_entity") {
        await dispatch("kubernetesClusterIntelligence", { entity });
      } else if (entity.siStorable?.typeName == "minikube_entity") {
        await dispatch("minikubeIntelligence", { entity });
      } else if (entity.siStorable?.typeName == "server_entity") {
        await dispatch("serverIntelligence", { entity });
      }
      await dispatch("changeSet/get", { changeSetId }, { root: true });
    },
    async create(
      { commit, dispatch, rootGetters },
      payload: CreateEntityPayload,
    ): Promise<Entity> {
      const variables: Record<string, any> = {};
      const workspaceId = rootGetters["workspace/current"].id;
      let changeSetId: string;
      try {
        changeSetId = rootGetters["changeSet/currentId"];
      } catch (err) {
        await dispatch("changeSet/createDefault", {}, { root: true });
        changeSetId = rootGetters["changeSet/currentId"];
      }
      variables.changeSetId = changeSetId;
      variables.workspaceId = workspaceId;
      let name: string;
      if (payload.data?.name) {
        name = payload.data?.name;
      } else {
        name = generateName();
      }
      variables.name = name;
      variables.displayName = name;
      variables.description = name;
      if (payload.data?.properties) {
        variables.properties = payload.data.properties;
      } else if (payload.typeName == "kubernetesDeploymentEntity") {
        variables.properties = {
          kubernetesObject: {
            apiVersion: "apps/v1",
            kind: "Deployment",
          },
        };
      } else if (payload.typeName == "kubernetesServiceEntity") {
        variables.properties = {
          kubernetesObject: {
            apiVersion: "apps/v1",
            kind: "Service",
          },
        };
      } else {
        variables.properties = {};
      }
      if (payload.data?.constraints) {
        variables.constraints = payload.data.constraints;
      } else {
        variables.constraints = {};
      }
      const result = await graphqlMutation({
        typeName: payload.typeName,
        methodName: "create",
        variables,
      });
      const entity = result["item"];
      const addPayload: AddMutation = {
        entities: [entity],
      };
      commit("add", addPayload);
      if (payload.typeName == "application_entity") {
        await dispatch(
          "application/add",
          { applications: [entity] },
          { root: true },
        );
      }
      let entityId: string;
      if (entity.siStorable.itemId) {
        entityId = entity.siStorable.itemId;
      } else {
        entityId = entity.id;
      }
      let node = {
        entityId: entityId,
        name: entity.name,
        nodeType: NodeNodeKind.Entity,
        object: entity,
      };
      await dispatch(
        "node/add",
        {
          items: [node],
        },
        { root: true },
      );
      if (entity.siStorable?.typeName == "service_entity") {
        await dispatch("serviceEntityIntelligence", { entity });
      } else if (
        entity.siStorable?.typeName == "kubernetes_service_entity" ||
        entity.siStorable?.typeName == "kubernetes_deployment_entity"
      ) {
        await dispatch("kubernetesObjectIntelligence", { entity });
      } else if (entity.siStorable?.typeName == "kubernetes_cluster_entity") {
        await dispatch("kubernetesClusterIntelligence", { entity });
      } else if (entity.siStorable?.typeName == "minikube_entity") {
        await dispatch("minikubeIntelligence", { entity });
      } else if (entity.siStorable?.typeName == "server_entity") {
        await dispatch("serverIntelligence", { entity });
      }
      const newNode = rootGetters["node/getNodeByEntityId"](entityId);
      //await dispatch(
      //  "node/setMouseTrackSelection",
      //  { id: newNode.id },
      //  { root: true },
      //);
      //await dispatch("changeSet/get", { changeSetId }, { root: true });

      return entity;
    },
    async delete(
      { commit, getters, rootGetters, rootState, dispatch },
      payload: DeleteEntityAction,
    ) {
      let changeSetId: string;
      try {
        changeSetId = rootGetters["changeSet/current"].id;
      } catch (err) {
        await dispatch("changeSet/createDefault", {}, { root: true });
        changeSetId = rootGetters["changeSet/current"].id;
      }
      const result = await graphqlMutation({
        typeName: payload.typeName,
        methodName: "delete",
        variables: {
          id: payload.id,
          changeSetId,
        },
      });
      const entity = result["item"];
      commit("add", { entities: [entity] });
      await dispatch(
        "node/add",
        {
          items: [
            {
              entityId: entity.siStorable.itemId,
              name: entity.name,
              nodeType: NodeNodeKind.Entity,
              object: entity,
            },
          ],
        },
        { root: true },
      );
      await dispatch("changeSet/get", { changeSetId }, { root: true });
    },
    async get(
      { state, commit, rootGetters, dispatch },
      { id, typeName }: { id: string; typeName: string },
    ): Promise<void> {
      const entityGetResult = await graphqlQuery({
        typeName,
        methodName: "get",
        variables: {
          id,
        },
      });
      const entity = entityGetResult["item"];
      commit("add", { entities: [entity] });

      let node;
      if (entity.siStorable.itemId) {
        node = {
          entityId: entity.siStorable.itemId,
          name: entity.name,
          nodeType: NodeNodeKind.Entity,
          object: entity,
        };
      } else {
        node = {
          entityId: entity.id,
          name: entity.name,
          nodeType: NodeNodeKind.Entity,
          object: entity,
        };
      }
      await dispatch("node/add", { items: [node] }, { root: true });
    },
    async load({ commit, dispatch, rootState }): Promise<void> {
      let workspaceIdList = _.map(rootState.workspace.workspaces, "id");

      // HACK: For now, we load all the changeset data by just loading all
      // the data a fuckload of times. This isn't what we want long term, but
      // its just fine for now.
      let changeSetIdList = _.map(rootState.changeSet.changeSets, "id");
      // Make sure we get the raw data, too. Probably overkill.
      changeSetIdList.push(undefined);

      let fullEntities: Entity[] = [];

      // Load all the data for every workspace, for every changeSet.
      //
      // Right now, the API is wrong, as we don't require you to specify the workspace!!
      for (let _workspaceId of workspaceIdList) {
        for (let changeSetId of changeSetIdList) {
          let remainingItems = true;
          let nextPageToken = "";
          let defaultVariables: Record<string, any> = {};
          if (changeSetId) {
            defaultVariables["query"] = {
              changeSetId,
            };
          }

          while (remainingItems) {
            let itemList;
            if (nextPageToken) {
              itemList = await graphqlQuery({
                typeName: "item",
                methodName: "list",
                variables: {
                  pageToken: nextPageToken,
                  ...defaultVariables,
                },
              });
            } else {
              itemList = await graphqlQuery({
                typeName: "item",
                methodName: "list",
                variables: {
                  pageSize: "100",
                  ...defaultVariables,
                },
              });
            }
            let entities = _.filter(itemList["items"], (item): boolean => {
              if (/_entity$/.exec(item["siStorable"]["typeName"])) {
                return true;
              } else {
                return false;
              }
            });
            for (let entity of entities) {
              if (!_.find(fullEntities, entity.id)) {
                let fullEntity = await graphqlQuery({
                  typeName: entity.siStorable.typeName,
                  methodName: "get",
                  variables: {
                    id: entity.id,
                  },
                });
                fullEntities.push(fullEntity.item);
              }
            }
            nextPageToken = itemList["nextPageToken"];
            if (!nextPageToken) {
              remainingItems = false;
            }
          }
        }
      }
      commit("add", {
        entities: fullEntities,
      });
      // Populate the application store
      await dispatch(
        "application/add",
        {
          applications: _.filter(fullEntities, [
            "siStorable.typeName",
            "application_entity",
          ]),
        },
        { root: true },
      );
      let addEntitiesToNodes: Item[] = _.map(fullEntities, entity => {
        if (entity.siStorable.itemId) {
          return {
            entityId: entity.siStorable.itemId,
            name: entity.name,
            nodeType: NodeNodeKind.Entity,
            object: entity,
          };
        } else {
          return {
            entityId: entity.id,
            name: entity.name,
            nodeType: NodeNodeKind.Entity,
            object: entity,
          };
        }
      });
      await dispatch("node/add", { items: addEntitiesToNodes }, { root: true });
    },
  },
};
