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
    <SiToast />
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
import { Schematic, SchematicSchemaVariants } from "@/api/sdf/dal/schematic";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { GetSchematicArgs } from "@/service/schematic/get_schematic";
import SiToast from "@/atoms/SiToast.vue";

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

let oldSchematic: Schematic | undefined;
Rx.combineLatest([
  system$.pipe(
    Rx.map((system) => {
      const request: GetSchematicArgs = {};
      if (system) {
        request.systemId = system.id;
      }
      return request;
    }),
  ),
  standardVisibilityTriggers$,
])
  .pipe(
    Rx.switchMap(([request]) => {
      const variants = SchematicService.listSchemaVariants().pipe(Rx.take(1));
      const schematic = SchematicService.getSchematic(request).pipe(Rx.take(1));
      return Rx.from([[variants, schematic]]);
    }),
    Rx.switchMap((calls) => {
      return Rx.forkJoin(calls);
    }),
  )
  .pipe(untilUnmounted)
  .subscribe(([variants, schematic]) => {
    if (variants.error) {
      GlobalErrorService.set(variants);
      return Rx.from([]);
    }

    if (schematic.error) {
      GlobalErrorService.set(schematic);
      return Rx.from([]);
    }

    // If the schematic didn't change, but standard visibility triggers forced a refetch, we have to ignore it
    // We avoid passing this stale data around (it races making nodes teleport right after a move, as the local data is more up to date)
    if (!oldSchematic || !_.isEqual(oldSchematic, schematic)) {
      oldSchematic = schematic as Schematic;
      schematicData$.next(schematic as Schematic);
      schematicSchemaVariants$.next(variants as SchematicSchemaVariants);
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
