<template>
  <div id="panelTreeRoot" class="flex flex-col w-full h-full">
    <PanelContainer
      v-for="(panelContainer, panelContainerIndex) in panelContainers"
      :key="panelContainerIndex"
      :maximized-full-panel="maximizedData"
      :panel-container="panelContainer"
      parent-prefix="root"
      :index="panelContainerIndex"
      @panel-maximize-full="maximizePanelFull($event)"
      @panel-minimize-full="minimizePanelFull($event)"
    />
  </div>
</template>

<script setup lang="ts">
import type { IPanelContainer, PanelMaximized } from "./PanelTree/panel_types";
import { PanelType, PanelAttributeSubType } from "./PanelTree/panel_types";
import PanelContainer from "./PanelTree/PanelContainer.vue";
import { ref, watch, onBeforeMount } from "vue";

import * as Rx from "rxjs";
import _ from "lodash";
import { ComponentService } from "@/service/component";
import { GlobalErrorService } from "@/service/global_error";
import { componentsMetadata$ } from "@/observable/component";
import { untilUnmounted } from "vuse-rx";
import {
  schematicData$,
  schematicSchemaVariants$,
} from "@/observable/schematic";
import { system$ } from "@/observable/system";
import { eventResourceSynced$ } from "@/observable/resource";
import { eventCheckedQualifications$ } from "@/observable/qualification";
import { SchematicService } from "@/service/schematic";
import { SchematicKind } from "@/api/sdf/dal/schematic";
import { SchematicSchemaVariants, Schematic } from "@/api/sdf/dal/schematic";
import { GetSchematicArgs } from "@/service/schematic/get_schematic";

const maximizedData = ref<PanelMaximized | null>(null);

watch(
  maximizedData,
  (maximizedData) => {
    sessionStorage.setItem(
      "panelTreeRootMaximized",
      JSON.stringify(maximizedData),
    );
  },
  { deep: true },
);

onBeforeMount(() => {
  const item = sessionStorage.getItem("panelTreeRootMaximized");
  if (item) {
    maximizedData.value = JSON.parse(item);
  }
});

const panelContainers = ref<IPanelContainer[]>([
  {
    orientation: "row",
    type: "panelContainer",
    panels: [
      {
        orientation: "column",
        type: "panelContainer",
        width: 60,
        panels: [
          {
            name: PanelType.Schematic,
            type: "panel",
            subType: SchematicKind.Deployment,
          },
          {
            name: PanelType.Schematic,
            type: "panel",
            subType: SchematicKind.Component,
          },
        ],
      },
      {
        orientation: "column",
        type: "panelContainer",
        panels: [
          {
            name: PanelType.Attribute,
            type: "panel",
            subType: PanelAttributeSubType.Attributes,
          },
          {
            name: PanelType.Attribute,
            type: "panel",
            subType: PanelAttributeSubType.Qualifications,
          },
        ],
      },
    ],
  },
]);

const minimizePanelFull = (_event: PanelMaximized) => {
  maximizedData.value = null;
};

const maximizePanelFull = (event: PanelMaximized) => {
  maximizedData.value = event;
};

// Hardcoded schematic schema variants, should come from the DB with the appropriate ids
// For now we use dummy ids and update it below when the metadata arrives
const schemaVariants = ref<SchematicSchemaVariants>([
  {
    id: -1,
    name: "v0",
    schemaName: "kubernetes_deployment",
    color: 0x921ed6,
    inputSockets: [
      {
        id: -1,
        name: "input",
        schematicKind: SchematicKind.Component,
        provider: {
          ty: "docker_image",
          color: 0xd61e8c,
        },
      },
    ],
    outputSockets: [],
  },
  {
    id: -2,
    name: "v0",
    schemaName: "docker_image",
    color: 0xd61e8c,
    inputSockets: [
      {
        id: -2,
        name: "input",
        schematicKind: SchematicKind.Component,
        provider: {
          ty: "docker_hub_credential",
          color: 0x1e88d6,
        },
      },
    ],
    outputSockets: [
      {
        id: -3,
        name: "output",
        schematicKind: SchematicKind.Component,
        provider: {
          ty: "docker_image",
          color: 0xd61e8c,
        },
      },
    ],
  },
  {
    id: -3,
    name: "v0",
    schemaName: "service",
    color: 0x00b0bc,
    inputSockets: [],
    outputSockets: [],
  },
  {
    id: -4,
    name: "v0",
    color: 0x00b0bc,
    schemaName: "application",
    inputSockets: [],
    outputSockets: [],
  },
  {
    id: -5,
    name: "v0",
    schemaName: "docker_hub_credential",
    color: 0x1e88d6,
    inputSockets: [],
    outputSockets: [
      {
        id: -3,
        name: "output",
        schematicKind: SchematicKind.Component,
        provider: {
          ty: "docker_hub_credential",
          color: 0x1e88d6,
        },
      },
    ],
  },
  {
    id: -6,
    name: "v0",
    schemaName: "bobão",
    color: 0x1e88d6,
    inputSockets: [],
    outputSockets: [],
  },
]);

// Hack to adapt SDF response to the new datastructures (and augment it with the metadata above)
let oldSchematic: Schematic | undefined = undefined;
Rx.combineLatest([
  system$.pipe(
    Rx.switchMap((system) => {
      const request: GetSchematicArgs = {};
      if (system) {
        request.systemId = system.id;
      }
      return SchematicService.getSchematic(request);
    }),
  ),
  // Allows us to tie a node to a schema variant
  ComponentService.listComponentsIdentification().pipe(
    Rx.switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return Rx.from([[]]);
      } else {
        for (const entry of response.list) {
          for (const schemaVariant of schemaVariants.value) {
            if (
              entry.value.schemaName === schemaVariant.schemaName &&
              entry.value.schemaVariantName === schemaVariant.name
            ) {
              schemaVariant.id = entry.value.schemaVariantId;
              break;
            }
          }
        }
        return Rx.from([response.list]);
      }
    }),
  ),
])
  .pipe(untilUnmounted)
  .subscribe(async ([schematic, componentIdentificationList]) => {
    if (schematic) {
      if (schematic.error) {
        GlobalErrorService.set(schematic);
        return Rx.from([null]);
      } else {
        const schematicView: Schematic = { nodes: [], connections: [] };

        for (const node of schematic.nodes) {
          // Connect the two sources of data
          const identification = componentIdentificationList.find((entry) => {
            return entry.value.componentId === node.kind.componentId;
          });
          if (!identification) return;

          const variantId = identification.value.schemaVariantId;
          const variant = schemaVariants.value.find((v) => v.id === variantId);
          if (!variant) throw Error("schema variant not found: " + variantId);

          // Fix hardcoded data's ids

          if (variant.inputSockets.length) {
            for (const input of node.input) {
              if (input.name === "input") {
                variant.inputSockets[0].id = input.id;
                break;
              }
            }
          }

          if (variant.outputSockets.length) {
            for (const output of node.output) {
              if (output.name === "output") {
                variant.outputSockets[0].id = output.id;
                break;
              }
            }
          }

          // Convert from old position structure to new one

          const positions = [];
          for (const position of node.position) {
            positions.push({
              x: parseFloat(position.x as string),
              y: parseFloat(position.y as string),
              schematicKind: position.schematic_kind,
              deploymentNodeId: position.deployment_node_id ?? undefined,
              systemId: position.system_id ?? undefined,
            });
          }

          if (!node.kind.componentId) throw new Error("componentId missing");
          const kind = {
            kind: node.kind.kind,
            componentId: node.kind.componentId,
          };
          schematicView.nodes.push({
            id: node.id,
            kind,
            name: node.label.name,
            title: node.label.title,
            schemaVariantId: identification.value.schemaVariantId ?? undefined,
            positions,
          });
        }

        for (const connection of schematic.connections) {
          schematicView.connections.push({
            sourceNodeId: connection.source.nodeId,
            sourceSocketId: connection.source.socketId,
            destinationNodeId: connection.destination.nodeId,
            destinationSocketId: connection.destination.socketId,
          });
        }

        // If the schematic didn't change, but visibility change triggered the `listComponentsIdentification` call
        // We avoid passing this stale data around (it makes nodes teleport after a move, as the local data is more up to date)
        // This if should go away when the backend gives us the appropriate metadata so we can stop calling `listComponentsIdentification` here
        if (!oldSchematic || !_.isEqual(oldSchematic, schematicView)) {
          oldSchematic = schematicView;
          schematicData$.next(_.cloneDeep(schematicView));
          schematicSchemaVariants$.next(schemaVariants.value);
        }
      }
    }
  });

// We should re-fetch the metadata if any resource from this system was synced
const resourceSynced$ = new Rx.ReplaySubject<true>();
resourceSynced$.next(true); // We must fetch on setup
eventResourceSynced$.pipe(untilUnmounted).subscribe(async (resourceSyncId) => {
  const system = await Rx.firstValueFrom(system$);
  if ((system?.id ?? -1) === resourceSyncId?.payload?.data?.systemId) {
    // Note: we shouldn't actually retrigger getComponentsMetadata every time one resource syncs
    // But we generally sync all resources in batch, so it's ok for now, but we eventually will
    // want to refactor this logic
    resourceSynced$.next(true);
  }
});

// We should re-fetch the metadata if any qualification from this system was checked
const checkedQualifications$ = new Rx.ReplaySubject<true>();
checkedQualifications$.next(true); // We must fetch on setup
eventCheckedQualifications$
  .pipe(untilUnmounted)
  .subscribe(async (checkedQualificationId) => {
    const system = await Rx.firstValueFrom(system$);
    const sysId = checkedQualificationId?.payload?.data?.systemId;
    if ((system?.id ?? -1) === sysId) {
      // Note: we shouldn't actually retrigger getComponentsMetadata every time one qualification check runs
      checkedQualifications$.next(true);
    }
  });

Rx.combineLatest([system$, resourceSynced$, checkedQualifications$])
  .pipe(
    Rx.switchMap(([system]) => {
      return ComponentService.getComponentsMetadata({
        systemId: system?.id ?? -1,
      });
    }),
  )
  .pipe(untilUnmounted)
  .subscribe((response) => {
    if (response === null) {
      return;
    } else if (response.error) {
      GlobalErrorService.set(response);
      return;
    } else {
      componentsMetadata$.next(response.data);
      return;
    }
  });
</script>
