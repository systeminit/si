<template>
  <div class="flex flex-col w-full overflow-hidden">
    <div
      class="flex flex-row items-center h-10 px-6 py-2 text-base text-white align-middle property-section-bg-color"
    >
      <div v-if="componentMetadata?.schemaName" class="text-lg">
        {{ componentMetadata.schemaName }}
      </div>

      <div class="ml-2 flex">
        <SiIcon
          :tooltip-text="qualificationTooltip"
          :color="qualificationColor"
        >
          <CheckCircleIcon />
        </SiIcon>
      </div>

      <div class="ml-2 flex">
        <SiIcon :tooltip-text="resourceTooltip" :color="resourceColor">
          <CubeIcon />
        </SiIcon>
      </div>

      <div
        class="flex flow-row items-center justify-end flex-grow h-full text-xs text-center"
      >
        <SiLink
          v-if="componentMetadata?.schemaLink"
          :uri="componentMetadata.schemaLink"
          :blank-target="true"
          class="m-2 flex"
        >
          <SiButtonIcon tooltip-text="Go to documentation">
            <QuestionMarkCircleIcon />
          </SiButtonIcon>
        </SiLink>

        <div class="flex flex-row items-center">
          <SiIcon tooltip-text="Number of edit fields" color="#ce7f3e">
            <PencilAltIcon />
          </SiIcon>
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
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";
import { EditFieldService } from "@/service/edit_field";
import { ResourceHealth } from "@/api/sdf/dal/resource";
import { ChangedEditFieldCounterVisitor } from "@/utils/edit_field_visitor";
import { ComponentIdentification } from "@/api/sdf/dal/component";
import { componentsMetadata$ } from "@/organisims/SchematicViewer/data/observable";
import { ComponentMetadata } from "@/service/component/get_components_metadata";
//import { Visibility } from "@/api/sdf/dal/visibility";
import {
  standardVisibilityTriggers$,
  //visibility$,
} from "@/observable/visibility";
import { editSessionWritten$ } from "@/observable/edit_session";
import SiLink from "@/atoms/SiLink.vue";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import SiIcon from "@/atoms/SiIcon.vue";
import {
  CheckCircleIcon,
  CubeIcon,
  QuestionMarkCircleIcon,
  PencilAltIcon,
} from "@heroicons/vue/outline";

//const visibility = refFrom<Visibility>(visibility$);

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
  Rx.combineLatest([
    componentId$,
    standardVisibilityTriggers$,
    editSessionWritten$,
  ]).pipe(
    Rx.switchMap(([componentId, [visibility]]) => {
      return EditFieldService.getEditFields({
        id: componentId,
        objectKind: EditFieldObjectKind.Component,
        // We aren't reactive to visibility as our parent will retrigger this by updating componentId or our key
        ...visibility,
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

const qualificationTooltip = computed(() => {
  if (
    !componentMetadata.value ||
    componentMetadata.value.qualified === undefined
  ) {
    return "Qualification is unknown";
  } else if (componentMetadata.value.qualified) {
    return "Qualification succeeded";
  } else {
    return "Qualification failed";
  }
});

const qualificationColor = computed(() => {
  if (
    !componentMetadata.value ||
    componentMetadata.value.qualified === undefined
  ) {
    return "#5b6163";
  } else if (componentMetadata.value.qualified) {
    return "#86f0ad";
  } else {
    return "#f08686";
  }
});

const resourceTooltip = computed(() => {
  if (
    !componentMetadata.value ||
    componentMetadata.value.resourceHealth === undefined
  ) {
    return "Resource Health Status is: Unknown";
  }

  const health = componentMetadata.value.resourceHealth;
  if (health == ResourceHealth.Ok) {
    return "Resource Health Status is: Ok";
  } else if (health == ResourceHealth.Warning) {
    return "Resource Health Status is: Warning";
  } else if (health == ResourceHealth.Error) {
    return "Resource Health Status is: Error";
  } else {
    return "Resource Health Status is: Unknown";
  }
});

const resourceColor = computed(() => {
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

.property-section-bg-color {
  background-color: #292c2d;
}
</style>
