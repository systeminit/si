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
    <EditFormComponent
      v-if="editFields"
      :edit-fields="editFields"
      :component-identification="componentIdentification"
    />
  </div>
</template>

<script setup lang="ts">
import * as Rx from "rxjs";
import EditFormComponent from "@/organisims/EditFormComponent.vue";
import { toRefs, computed } from "vue";
import { fromRef, refFrom } from "vuse-rx";
import { GlobalErrorService } from "@/service/global_error";
import VueFeather from "vue-feather";
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";
import { EditFieldService } from "@/service/edit_field";
import { ResourceHealth } from "@/api/sdf/dal/resource";
import { ChangedEditFieldCounterVisitor } from "@/utils/edit_field_visitor";
import { ComponentIdentification } from "@/api/sdf/dal/component";
import { componentsMetadata$ } from "@/organisims/SchematicViewer/data/observable";
import { ComponentMetadata } from "@/service/component/get_components_metadata";
import { Visibility } from "@/api/sdf/dal/visibility";
import { visibility$ } from "@/observable/visibility";

const visibility = refFrom<Visibility>(visibility$);

// TODO(nick): we technically only need one prop. We're sticking with two to not mess
// with the reactivity guarentees in place.
const props = defineProps<{
  componentId: number;
  componentIdentification: ComponentIdentification;
}>();

const { componentId, componentIdentification } = toRefs(props);

// We need an observable stream of props.componentId. We also want
// that stream to emit a value immediately (the first value, as well as all
// subsequent values)
const componentId$ = fromRef<number>(componentId, { immediate: true });

const editFields = refFrom<EditFields | undefined>(
  Rx.combineLatest([componentId$]).pipe(
    Rx.switchMap(([componentId]) => {
      if (!visibility.value) return Rx.from([null]);
      return EditFieldService.getEditFields({
        id: componentId,
        objectKind: EditFieldObjectKind.Component,
        // We aren't reactive to visibility as our parent will retrigger this by updating componentId or our key
        ...visibility.value,
      });
    }),
    Rx.switchMap((response) => {
      if (response === null) {
        return Rx.from([[]]);
      } else if (response.error) {
        GlobalErrorService.set(response);
        return Rx.from([[]]);
      } else {
        return Rx.from([response.fields]);
      }
    }),
  ),
);

const componentMetadata = refFrom<ComponentMetadata | null>(
  Rx.combineLatest([componentId$, componentsMetadata$]).pipe(
    Rx.map(([componentId, componentsMetadata]) => {
      if (!componentsMetadata) return null;

      for (const metadata of componentsMetadata) {
        if (metadata.componentId === componentId) {
          return metadata;
        }
      }
      return null;
    }),
  ),
);

const editCount = computed(() => {
  if (editFields.value === undefined) {
    return undefined;
  } else {
    const counter = new ChangedEditFieldCounterVisitor();
    counter.visitEditFields(editFields.value);
    return counter.count();
  }
});

const qualificationStatus = computed(() => {
  let style: Record<string, boolean> = {};

  if (
    !componentMetadata.value ||
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
    !componentMetadata.value ||
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
