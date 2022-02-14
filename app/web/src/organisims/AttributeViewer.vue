<template>
  <div class="flex flex-col w-full overflow-hidden">
    <div
      class="flex flex-row items-center h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div v-if="componentMetadata?.schemaName" class="text-lg">
        {{ componentMetadata.schemaName }}
      </div>

      <div class="ml-2 text-base">
        <VueFeather
          type="check-square"
          size="1em"
          :class="qualificationStatus"
        />
      </div>

      <div class="ml-2 text-base">
        <VueFeather type="box" size="1em" :stroke="resourceSyncStatusStroke" />
      </div>

      <div
        class="flex flow-row items-center justify-end flex-grow h-full text-xs text-center"
      >
        <div class="flex flex-row items-center">
          <VueFeather type="edit" size="0.75rem" class="gold-bars-icon" />
          <div v-if="editCount" class="ml-1 text-center">{{ editCount }}</div>
        </div>
      </div>
    </div>
    <EditFormComponent v-if="editFields" :edit-fields="editFields" />
  </div>
</template>

<script setup lang="ts">
import EditFormComponent from "@/organisims/EditFormComponent.vue";
import { ComponentService } from "@/service/component";
import { GetComponentMetadataResponse } from "@/service/component/get_component_metadata";
import { toRefs, computed } from "vue";
import { fromRef, refFrom } from "vuse-rx";
import { from, combineLatest, ReplaySubject } from "rxjs";
import { switchMap, tap } from "rxjs/operators";
import { system$ } from "@/observable/system";
import { eventResourceSynced$ } from "@/observable/resource";
import { GlobalErrorService } from "@/service/global_error";
import VueFeather from "vue-feather";
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";
import { EditFieldService } from "@/service/edit_field";
import { ResourceHealth } from "@/api/sdf/dal/resource";

const props = defineProps<{
  componentId: number;
}>();

const { componentId } = toRefs(props);

// We need an observable stream of props.componentId. We also want
// that stream to emit a value immediately (the first value, as well as all
// subsequent values)
const componentId$ = fromRef<number>(componentId, { immediate: true });

const editFields = refFrom<EditFields | undefined>(
  combineLatest([componentId$]).pipe(
    switchMap(([componentId]) => {
      return EditFieldService.getEditFields({
        id: componentId,
        objectKind: EditFieldObjectKind.Component,
      });
    }),
    switchMap((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return from([[]]);
      } else {
        return from([response.fields]);
      }
    }),
  ),
);

// TODO: we should re-fetch the metadata when qualifications change

// We should re-fetch the metadata if the resource was synced
const resourceSynced$ = new ReplaySubject<true>();
resourceSynced$.next(true); // We must fetch on setup
eventResourceSynced$.subscribe((resourceSyncId) => {
  combineLatest([system$]).pipe(
    tap(([system]) => {
      const data = resourceSyncId?.payload.data;
      const sameComponent = props.componentId === data?.componentId;
      const sameSystem = system?.id === data?.systemId;
      if (sameComponent && sameSystem) {
        resourceSynced$.next(true);
      }
    }),
  );
});

const componentMetadata = refFrom<GetComponentMetadataResponse | undefined>(
  combineLatest([componentId$, system$, resourceSynced$]).pipe(
    switchMap(([componentId, system]) => {
      if (system) {
        return ComponentService.getComponentMetadata({
          componentId,
          systemId: system.id,
        });
      } else {
        return from([null]);
      }
    }),
    switchMap((response) => {
      if (response === null) {
        return from([undefined]);
      } else if (response.error) {
        GlobalErrorService.set(response);
        return from([undefined]);
      } else {
        return from([response]);
      }
    }),
  ),
);

const editCount = computed(() => {
  if (editFields === undefined) {
    return undefined;
  } else {
    // TODO(fnichol): Implement the logic to count edited fields.
    //
    // To accomplish this, we can interate through each `EditField` and filter
    // only entries that have:
    //
    // `editField.visibility_diff != VisibilityDiffNone`
    //
    // The tricky part is that `EditField`s nest, so we need to visit and count
    // inside of each `PropObject`, `PropArray`, and `PropMap` type. That's the
    // same traversal/visit logic needed for other info such as computing the
    // deepest path in a Component, so I suspect there's something
    // generalizable once we get the first iteration of an implementation.
    return 666;
  }
});

const qualificationStatus = computed(() => {
  let style: Record<string, boolean> = {};

  if (
    componentMetadata.value === undefined ||
    componentMetadata.value.qualified === undefined
  ) {
    style["unknown"] = true;
  } else if (componentMetadata.value.qualified) {
    style["ok"] = true;
  } else {
    style["error"] = true;
  }

  return style;
});

const resourceSyncStatusStroke = computed(() => {
  if (
    componentMetadata.value === undefined ||
    componentMetadata.value.resourceHealth === undefined
  ) {
    return "#bbbbbb";
  }

  const health = componentMetadata.value.resourceHealth;
  if (health == ResourceHealth.Ok) {
    return "#86f0ad";
  } else if (health == ResourceHealth.Warning) {
    return "#f0d286";
  } else if (health == ResourceHealth.Error) {
    return "#f08686";
  } else {
    return "#bbbbbb";
  }
});
</script>

<style scoped>
.scrollbar {
  -ms-overflow-style: none; /* edge, and ie */
  scrollbar-width: none; /* firefox */
}

.scrollbar::-webkit-scrollbar {
  display: none; /*chrome, opera, and safari */
}

.gold-bars-icon {
  color: #ce7f3e;
}

.property-section-bg-color {
  background-color: #292c2d;
}

.ok {
  color: #86f0ad;
}

.warning {
  color: #f0d286;
}

.error {
  color: #f08686;
}

.unknown {
  color: #5b6163;
}
</style>
