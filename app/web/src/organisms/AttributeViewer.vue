<template>
  <div class="flex flex-col w-full">
    <div
      class="flex flex-row items-center h-10 px-6 py-2 text-base align-middle"
    >
      <div class="text-lg whitespace-nowrap overflow-hidden text-ellipsis">
        {{ selectedComponent.schemaName }}
      </div>
      <div class="ml-2 flex" :aria-label="qualificationTooltip">
        <Icon name="check-circle" :class="qualificationColorClass" />
      </div>

      <div class="ml-2 flex" :aria-label="resourceTooltip">
        <Icon name="component" :class="resourceIconColorClass" />
      </div>

      <div
        class="flex flow-row items-center justify-end flex-grow h-full text-xs text-center"
      >
        <SiLink
          v-if="componentMetadata?.schemaLink"
          :uri="componentMetadata.schemaLink"
          blank-target
          class="m-2 flex"
        >
          <SiButtonIcon tooltip-text="Go to documentation" icon="help-circle" />
        </SiLink>

        <div
          v-if="editCount"
          class="flex flex-row items-center"
          aria-label="Number of edit fields"
        >
          <Icon name="edit" class="text-warning-600" />
          <div class="ml-1 text-center">{{ editCount }}</div>
        </div>
      </div>
    </div>

    <PropertyEditor
      v-if="editorContext"
      :editor-context="editorContext"
      @updated-property="updateProperty"
      @add-to-array="addToArray"
      @add-to-map="addToMap"
      @create-attribute-func="onCreateAttributeFunc"
    />
  </div>
</template>

<script setup lang="ts">
// import EditFormComponent from "@/organisms/EditFormComponent.vue";
import { computed } from "vue";
import { fromRef, refFrom, untilUnmounted } from "vuse-rx";
import _, { parseInt } from "lodash";
import { tag } from "rxjs-spy/operators";
import { combineLatest, forkJoin, from, map, take } from "rxjs";
import { switchMap } from "rxjs/operators";
import { GlobalErrorService } from "@/service/global_error";
import { ResourceHealth } from "@/api/sdf/dal/resource";
import { componentsMetadata$ } from "@/observable/component";
import { ComponentMetadata } from "@/service/component/get_components_metadata";
import SiLink from "@/atoms/SiLink.vue";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import {
  PropertyEditorSchema,
  PropertyEditorValues,
  PropertyEditorValidations,
  UpdatedProperty,
  AddToArray,
  AddToMap,
  FuncWithPrototypeContext,
} from "@/api/sdf/dal/property_editor";
import { ComponentService } from "@/service/component";
import { SystemService } from "@/service/system";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Icon from "@/ui-lib/Icon.vue";
import { FuncBackendKind } from "@/api/sdf/dal/func";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import { useComponentsStore } from "@/store/components.store";
import { useFuncStore } from "@/store/funcs.store";
import PropertyEditor, { PropertyEditorContext } from "./PropertyEditor.vue";

const funcStore = useFuncStore();
const routeToFunc = useRouteToFunc();

const componentsStore = useComponentsStore();
const selectedComponent = computed(() => componentsStore.selectedComponent);
const selectedComponentId = computed(() => componentsStore.selectedComponentId);

const componentId$ = fromRef(selectedComponentId, { immediate: true });

const editorContext = refFrom<PropertyEditorContext | undefined>(
  combineLatest([
    componentId$,
    SystemService.currentSystem(),
    standardVisibilityTriggers$,
  ]).pipe(
    switchMap(([_component, system, _triggers]) => {
      const schema = ComponentService.getPropertyEditorSchema({
        componentId: selectedComponent.value.id,
      }).pipe(take(1));
      const values = ComponentService.getPropertyEditorValues({
        componentId: selectedComponent.value.id,
        systemId: system?.id ?? -1,
      }).pipe(take(1));
      const validations = ComponentService.getPropertyEditorValidations({
        componentId: selectedComponent.value.id,
        systemId: system?.id ?? -1,
      }).pipe(take(1));
      return from([[schema, values, validations]]);
    }),
    switchMap((calls) => {
      return forkJoin(calls);
    }),
    switchMap(
      ([
        propertyEditorSchema,
        propertyEditorValues,
        propertyEditorValidations,
      ]) => {
        if (
          propertyEditorSchema.error?.statusCode === 404 &&
          propertyEditorSchema.error?.message === "invalid visibility"
        ) {
          return from([]);
        } else if (propertyEditorSchema.error) {
          GlobalErrorService.set(propertyEditorSchema);
          return from([]);
        }

        if (
          propertyEditorValues.error?.statusCode === 404 &&
          propertyEditorValues.error?.message === "invalid visibility"
        ) {
          return from([]);
        } else if (propertyEditorValues.error) {
          GlobalErrorService.set(propertyEditorValues);
          return from([]);
        }

        if (
          propertyEditorValidations.error?.statusCode === 404 &&
          propertyEditorValidations.error?.message === "invalid visibility"
        ) {
          return from([]);
        } else if (propertyEditorValidations.error) {
          GlobalErrorService.set(propertyEditorValidations);
          return from([]);
        }

        const propertyEditorContext: PropertyEditorContext = {
          schema: propertyEditorSchema as PropertyEditorSchema,
          values: propertyEditorValues as PropertyEditorValues,
          validations: propertyEditorValidations as PropertyEditorValidations,
        };
        return from([propertyEditorContext]);
      },
    ),
    switchMap((propertyEditorContext) => {
      return from([hackAwayTheZeroElementOfContainers(propertyEditorContext)]);
    }),
    tag("properties"),
  ),
);

const componentMetadata = refFrom<ComponentMetadata | null>(
  combineLatest([componentId$, componentsMetadata$]).pipe(
    map(([_component, componentsMetadata]) => {
      if (!componentsMetadata) return null;

      for (const metadata of componentsMetadata) {
        if (metadata.componentId === selectedComponent.value.id) {
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

const qualificationColorClass = computed(() => {
  if (!componentMetadata.value || componentMetadata.value.qualified === null) {
    return "text-neutral-400";
  } else if (componentMetadata.value.qualified) {
    return "text-success-400";
  } else {
    return "text-destructive-400";
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
  if (health === ResourceHealth.Ok) {
    return "Resource Health Status is: Ok";
  } else if (health === ResourceHealth.Warning) {
    return "Resource Health Status is: Warning";
  } else if (health === ResourceHealth.Error) {
    return "Resource Health Status is: Error";
  } else {
    return "Resource Health Status is: Unknown";
  }
});

const resourceIconColorClass = computed(() => {
  const health = componentMetadata.value?.resourceHealth;
  if (health === ResourceHealth.Ok) {
    return "text-success-400";
  } else if (health === ResourceHealth.Warning) {
    return "text-warning-400";
  } else if (health === ResourceHealth.Error) {
    return "text-destructive-400";
  } else {
    return "text-neutral-400";
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
      attribute_context_schema_id: selectedComponent.value.schemaId,
      attribute_context_schema_variant_id:
        selectedComponent.value.schemaVariantId,
      attribute_context_component_id: selectedComponent.value.id,
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
      attribute_context_schema_id: selectedComponent.value.schemaId,
      attribute_context_schema_variant_id:
        selectedComponent.value.schemaVariantId,
      attribute_context_component_id: selectedComponent.value.id,
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
      attribute_context_schema_id: selectedComponent.value.schemaId,
      attribute_context_schema_variant_id:
        selectedComponent.value.schemaVariantId,
      attribute_context_component_id: selectedComponent.value.id,
      attribute_context_system_id: -1,
    },
  }).subscribe((result) => {
    if (result.error) {
      GlobalErrorService.set(result);
    }
  });
};

const onCreateAttributeFunc = async (
  currentFunc: FuncWithPrototypeContext,
  valueId: number,
  parentValueId?: number,
) => {
  funcStore.CREATE_FUNC(
    {
      kind: FuncBackendKind.JsAttribute,
      options: {
        valueId,
        parentValueId,
        componentId: selectedComponent.value.id,
        schemaVariantId: selectedComponent.value.schemaVariantId,
        schemaId: selectedComponent.value.schemaId,
        currentFuncId: currentFunc.id,
        type: "attributeOptions",
      },
    },
    (response) => routeToFunc(response.id),
  );
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

    if (parentProp.kind === "array" || parentProp.kind === "map") {
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
</style>
