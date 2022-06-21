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

        <div v-if="editCount" class="flex flex-row items-center">
          <SiIcon tooltip-text="Number of edit fields" color="#ce7f3e">
            <PencilAltIcon />
          </SiIcon>
          <div class="ml-1 text-center">{{ editCount }}</div>
        </div>
      </div>
    </div>

    <PropertyEditor
      v-if="editorContext"
      :editor-context="editorContext"
      @updated-property="updateProperty($event)"
      @add-to-array="addToArray($event)"
      @add-to-map="addToMap($event)"
    />

    <!--
    <EditFormComponent
      v-if="editFields"
      :edit-fields="editFields"
      :component-identification="componentIdentification"
    /> -->
  </div>
</template>

<script setup lang="ts">
import * as Rx from "rxjs";
//import EditFormComponent from "@/organisims/EditFormComponent.vue";
import { toRefs, computed } from "vue";
import { fromRef, refFrom, untilUnmounted } from "vuse-rx";
import { GlobalErrorService } from "@/service/global_error";
import { ResourceHealth } from "@/api/sdf/dal/resource";
import { ComponentIdentification } from "@/api/sdf/dal/component";
import { componentsMetadata$ } from "@/observable/component";
import { ComponentMetadata } from "@/service/component/get_components_metadata";
import SiLink from "@/atoms/SiLink.vue";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import SiIcon from "@/atoms/SiIcon.vue";
import { CheckCircleIcon } from "@heroicons/vue/solid";
import {
  CubeIcon,
  QuestionMarkCircleIcon,
  PencilAltIcon,
} from "@heroicons/vue/outline";
import PropertyEditor, { PropertyEditorContext } from "./PropertyEditor.vue";
import {
  PropertyEditorSchema,
  PropertyEditorValues,
  PropertyEditorValidations,
  UpdatedProperty,
  AddToArray,
  AddToMap,
} from "@/api/sdf/dal/property_editor";
import { ComponentService } from "@/service/component";
import { SystemService } from "@/service/system";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import _, { parseInt } from "lodash";

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

const editorContext = refFrom<PropertyEditorContext | undefined>(
  Rx.combineLatest([
    componentId$,
    SystemService.currentSystem(),
    standardVisibilityTriggers$,
  ]).pipe(
    Rx.switchMap(([componentId, system, _triggers]) => {
      const schema = ComponentService.getPropertyEditorSchema({
        componentId: componentId,
      }).pipe(Rx.take(1));
      const values = ComponentService.getPropertyEditorValues({
        componentId,
        systemId: system?.id ?? -1,
      }).pipe(Rx.take(1));
      const validations = ComponentService.getPropertyEditorValidations({
        componentId,
        systemId: system?.id ?? -1,
      }).pipe(Rx.take(1));
      return Rx.from([[schema, values, validations]]);
    }),
    Rx.switchMap((calls) => {
      return Rx.forkJoin(calls);
    }),
    Rx.switchMap(
      ([
        propertyEditorSchema,
        propertyEditorValues,
        propertyEditorValidations,
      ]) => {
        if (
          propertyEditorSchema.error?.statusCode === 404 &&
          propertyEditorSchema.error?.message === "invalid visibility"
        ) {
          return Rx.from([]);
        } else if (propertyEditorSchema.error) {
          GlobalErrorService.set(propertyEditorSchema);
          return Rx.from([]);
        }

        if (
          propertyEditorValues.error?.statusCode === 404 &&
          propertyEditorValues.error?.message === "invalid visibility"
        ) {
          return Rx.from([]);
        } else if (propertyEditorValues.error) {
          GlobalErrorService.set(propertyEditorValues);
          return Rx.from([]);
        }

        if (
          propertyEditorValidations.error?.statusCode === 404 &&
          propertyEditorValidations.error?.message === "invalid visibility"
        ) {
          return Rx.from([]);
        } else if (propertyEditorValidations.error) {
          GlobalErrorService.set(propertyEditorValidations);
          return Rx.from([]);
        }

        const propertyEditorContext: PropertyEditorContext = {
          schema: propertyEditorSchema as PropertyEditorSchema,
          values: propertyEditorValues as PropertyEditorValues,
          validations: propertyEditorValidations as PropertyEditorValidations,
        };
        return Rx.from([propertyEditorContext]);
      },
    ),
    Rx.switchMap((propertyEditorContext) => {
      return Rx.from([
        hackAwayTheZeroElementOfContainers(propertyEditorContext),
      ]);
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

componentId$.pipe(untilUnmounted).subscribe(() => {
  editorContext.value = undefined;
  componentMetadata.value = null;
});

const editCount = computed(() => {
  return 0;
  //  if (editFields.value === undefined) {
  //    return 0;
  //  } else {
  //    const counter = new ChangedEditFieldCounterVisitor();
  //    counter.visitEditFields(editFields.value);
  //    return counter.count();
  //  }
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
  if (!componentMetadata.value || componentMetadata.value.qualified === null) {
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

const updateProperty = (update: UpdatedProperty) => {
  console.log("updating", { update });
  ComponentService.updateFromEditField({
    attributeValueId: update.valueId,
    parentAttributeValueId: update.parentValueId,
    value: update.value,
    key: update.key,
    attributeContext: {
      attribute_context_prop_id: update.propId,
      attribute_context_internal_provider_id: -1,
      attribute_context_external_provider_id: -1,
      attribute_context_schema_id: componentIdentification.value.schemaId,
      attribute_context_schema_variant_id:
        componentIdentification.value.schemaVariantId,
      attribute_context_component_id: componentIdentification.value.componentId,
      attribute_context_system_id: -1,
    },
  }).subscribe((result) => {
    if (result.error) {
      GlobalErrorService.set(result);
    }
  });
};

const addToArray = (event: AddToArray) => {
  ComponentService.insertFromEditField({
    parentAttributeValueId: event.valueId,
    attributeContext: {
      attribute_context_prop_id: event.propId,
      attribute_context_internal_provider_id: -1,
      attribute_context_external_provider_id: -1,
      attribute_context_schema_id: componentIdentification.value.schemaId,
      attribute_context_schema_variant_id:
        componentIdentification.value.schemaVariantId,
      attribute_context_component_id: componentIdentification.value.componentId,
      attribute_context_system_id: -1,
    },
  }).subscribe((result) => {
    if (result.error) {
      GlobalErrorService.set(result);
    }
  });
};
const addToMap = (event: AddToMap) => {
  ComponentService.insertFromEditField({
    parentAttributeValueId: event.valueId,
    key: event.key,
    attributeContext: {
      attribute_context_prop_id: event.propId,
      attribute_context_internal_provider_id: -1,
      attribute_context_external_provider_id: -1,
      attribute_context_schema_id: componentIdentification.value.schemaId,
      attribute_context_schema_variant_id:
        componentIdentification.value.schemaVariantId,
      attribute_context_component_id: componentIdentification.value.componentId,
      attribute_context_system_id: -1,
    },
  }).subscribe((result) => {
    if (result.error) {
      GlobalErrorService.set(result);
    }
  });
};

const hackAwayTheZeroElementOfContainers = (
  propertyEditorContext: PropertyEditorContext | undefined,
): PropertyEditorContext | undefined => {
  if (_.isUndefined(propertyEditorContext)) {
    return undefined;
  }

  const filteredChildValues: { [key: number]: Array<number> } = {};

  for (const [parentValueId, childValuesIds] of Object.entries(
    propertyEditorContext.values.childValues,
  )) {
    const parentValue =
      propertyEditorContext.values.values[parseInt(parentValueId, 10)];
    if (!parentValue) {
      // If we don't find a value, then don't filter and continue
      filteredChildValues[parseInt(parentValueId, 10)] = childValuesIds;
      continue;
    }
    const parentProp = propertyEditorContext.schema.props[parentValue.propId];
    if (!parentProp) {
      // If we don't find a prop, then don't filter and continue
      filteredChildValues[parentValue.id] = childValuesIds;
      continue;
    }

    if (parentProp.kind == "array" || parentProp.kind == "map") {
      filteredChildValues[parentValue.id] = childValuesIds.filter(
        (childValueId) => {
          const childValue = propertyEditorContext.values.values[childValueId];
          if (childValue && _.isNull(childValue.key)) {
            // If we don't find a value, then don't filter it out
            return false;
          } else {
            return true;
          }
        },
      );
    } else {
      filteredChildValues[parentValue.id] = childValuesIds;
    }
  }

  propertyEditorContext.values.childValues = filteredChildValues;

  return propertyEditorContext;
};
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
