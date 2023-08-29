<template>
  <div class="flex flex-col w-full h-full pb-5 text-left">
    <div
      v-for="(pv, index) in propertyValuesInOrder"
      :key="pv.id"
      class="flex flex-col w-full"
    >
      <PropertyWidget
        v-if="schemasByPropId[pv.propId]"
        :schemaProp="schemasByPropId[pv.propId]!"
        :propValue="pv"
        :path="paths[pv.id]"
        :collapsedPaths="collapsed"
        :validation="validationsByValueId[pv.id]"
        :arrayIndex="arrayIndicesByValueId[pv.id]"
        :arrayLength="arrayLengthsByPropId[pv.propId]"
        :isFirstProp="index === 0"
        :disabled="props.disabled"
        @toggle-collapsed="toggleCollapsed($event)"
        @updated-property="updatedProperty($event)"
        @add-to-array="addToArray($event)"
        @add-to-map="addToMap($event)"
      />
    </div>
    <!-- temporary code for testing secrets popover -->
    <div
      v-if="featureFlagsStore.SECRETS"
      class="flex flex-col w-full pt-sm px-lg"
    >
      <VButton label="Add Secret" @click="(e) => popoverRef.open(e)" />
      <Popover ref="popoverRef" anchorDirectionX="left" anchorAlignY="bottom">
        <SecretsList definitionName="AWS Credential" />
      </Popover>
    </div>
    <!-- temporary code for testing secrets popover -->
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import * as _ from "lodash-es";
import { VButton } from "@si/vue-lib/design-system";
import {
  PropertyEditorSchema,
  PropertyEditorValues,
  PropertyEditorValue,
  UpdatedProperty,
  AddToArray,
  AddToMap,
  PropertyPath,
  PropertyEditorValidation,
} from "@/api/sdf/dal/property_editor";
import { useComponentsStore } from "@/store/components.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import PropertyWidget from "./PropertyEditor/PropertyWidget.vue";
import Popover from "./Popover.vue";
import SecretsList from "./SecretsList.vue";

export interface PropertyEditorContext {
  schema: PropertyEditorSchema;
  values: PropertyEditorValues;
  validations: PropertyEditorValidation[];
}

const featureFlagsStore = useFeatureFlagsStore();

const popoverRef = ref();

const props = defineProps<{
  editorContext: PropertyEditorContext;
  disabled?: boolean;
}>();

const emits = defineEmits<{
  (e: "updatedProperty", v: UpdatedProperty): void;
  (e: "addToArray", v: AddToArray): void;
  (e: "addToMap", v: AddToMap): void;
}>();

const values = computed(() => props.editorContext.values);
const validations = computed(() => props.editorContext.validations);

const schemasByPropId = computed(() => {
  return props.editorContext.schema.props;
});
const validationsByValueId = computed(() => {
  return _.keyBy(validations.value, (v) => v.valueId);
});

const collapsed = ref<Array<Array<string>>>([]);
const toggleCollapsed = (path: string[]) => {
  for (let x = 0; x < collapsed.value.length; x++) {
    const c = collapsed.value[x];
    if (_.isEqual(c, path)) {
      collapsed.value = _.filter(collapsed.value, (v) => {
        return !_.isEqual(v, path);
      });
      return;
    }
  }
  collapsed.value.push(path);
};

const findParentProp = (propId: string) => {
  for (const [parentPropId, childPropIds] of Object.entries(
    props.editorContext.schema.childProps,
  )) {
    for (const childProp of childPropIds) {
      if (childProp === propId) {
        const parentProp = schemasByPropId.value[parentPropId];
        if (parentProp) {
          return parentProp;
        } else {
          return undefined;
        }
      }
    }
  }
};

const findParentValue = (valueId: string) => {
  for (const [parentValueId, childValueIds] of Object.entries(
    values.value.childValues,
  )) {
    for (const childValueId of childValueIds) {
      if (childValueId === valueId) {
        const parentValue = values.value.values[parentValueId];
        if (parentValue) {
          return parentValue;
        } else {
          return undefined;
        }
      }
    }
  }
};

const pathPartForValueId = (valueId: string) => {
  let displayPathPart = "bug";
  let triggerPathPart = "bug";
  const currentValue = values.value.values[valueId];
  if (!currentValue) {
    return { displayPathPart, triggerPathPart };
  }
  const currentProp = schemasByPropId.value[currentValue.propId];
  const parentProp = findParentProp(currentValue.propId);

  if (currentValue && currentProp) {
    if (parentProp) {
      if (parentProp.kind === "array") {
        const index = findArrayIndex(currentValue.id);
        if (!_.isUndefined(index)) {
          if (currentProp.kind === "object") {
            displayPathPart = `[${index}](${currentProp.name})`;
            triggerPathPart = `[${index}]`;
          } else {
            displayPathPart = `[${index}]`;
            triggerPathPart = `[${index}]`;
          }
          return { displayPathPart, triggerPathPart };
        }
      } else if (parentProp.kind === "map") {
        if (currentProp.kind === "object") {
          displayPathPart = `{${currentValue.key}}(${currentProp.name})`;
          triggerPathPart = `{${currentValue.key}}`;
        } else {
          displayPathPart = `{${currentValue.key}}`;
          triggerPathPart = `{${currentValue.key}}`;
        }
        return { displayPathPart, triggerPathPart };
      }
    }
    if (currentProp.kind === "array") {
      const childCount = values.value.childValues[valueId];
      const arrayLength = childCount ? childCount.length : 0;
      displayPathPart = `${currentProp.name}[${arrayLength}]`;
      triggerPathPart = `${currentProp.name}[]`;
    } else if (currentProp.kind === "object") {
      displayPathPart = currentProp.name;
      triggerPathPart = currentProp.name;
    } else if (currentProp.kind === "map") {
      const childCount = values.value.childValues[valueId];
      const mapLength = childCount ? childCount.length : 0;
      displayPathPart = `${currentProp.name}{${mapLength}}`;
      triggerPathPart = `${currentProp.name}{}`;
    } else {
      displayPathPart = currentProp.name;
      triggerPathPart = currentProp.name;
    }
  }
  return { displayPathPart, triggerPathPart };
};

const findParentPath = (
  displayPath: string[],
  triggerPath: string[],
  valueId: string,
) => {
  for (const [parentValueId, childValueIds] of Object.entries(
    values.value.childValues,
  )) {
    for (const childValueId of childValueIds) {
      if (childValueId === valueId) {
        const pathPart = pathPartForValueId(parentValueId);
        displayPath.push(pathPart.displayPathPart);
        triggerPath.push(pathPart.triggerPathPart);
        findParentPath(displayPath, triggerPath, parentValueId);
      }
    }
  }
};

const paths = computed<{ [valueId: string]: PropertyPath | undefined }>(() => {
  const result: { [valueId: string]: PropertyPath } = {};
  for (const propValue of Object.values(values.value.values)) {
    // First, do ourselves - then our parents
    const pathPart = pathPartForValueId(propValue.id);
    const displayPath: string[] = [pathPart.displayPathPart];
    const triggerPath: string[] = [pathPart.triggerPathPart];
    findParentPath(displayPath, triggerPath, propValue.id);
    result[propValue.id] = {
      displayPath,
      triggerPath,
    };
  }
  return result;
});

const determineOrder = (
  order: PropertyEditorValue[],
  childValueIds: string[],
): PropertyEditorValue[] => {
  for (const childValueId of childValueIds) {
    const child = values.value.values[childValueId];
    if (child) {
      order.push(child);
    }
    const childValuesList = values.value.childValues[childValueId];
    if (childValuesList) {
      determineOrder(order, childValuesList);
    }
  }
  return order;
};

const propertyValuesInOrder = computed(() => {
  const results = determineOrder([], [values.value.rootValueId]);

  const component = useComponentsStore().selectedComponent;

  if (component?.nodeType === "aggregationFrame") {
    return _.filter(results, (r) => {
      const path = paths.value[r.id];
      if (!path) return false;
      const penultimateItem = path.displayPath.at(-2);
      return penultimateItem !== "domain";
    });
  }

  return results;
});

const updatedProperty = (event: UpdatedProperty) => {
  const parentAttributeValue = findParentValue(event.valueId);
  const attributeValue = values.value.values[event.valueId];
  if (parentAttributeValue) {
    event.parentValueId = parentAttributeValue.id;
  }
  if (attributeValue) {
    event.key = attributeValue.key;
  }
  emits("updatedProperty", event);
};

const addToArray = (event: AddToArray) => {
  emits("addToArray", event);
};

const addToMap = (event: AddToMap) => {
  emits("addToMap", event);
};

const findArrayIndex = (valueId: string) => {
  let parentProp;
  let index;
  for (const [parentValueId, childValues] of Object.entries(
    values.value.childValues,
  )) {
    for (let x = 0; x < childValues.length; x++) {
      const cv = childValues[x];
      if (cv === valueId) {
        index = x;
        const parentValue = values.value.values[parentValueId];
        if (parentValue) {
          parentProp = schemasByPropId.value[parentValue.propId];
        }
        break;
      }
    }
  }
  if (parentProp?.kind === "array") {
    return index;
  } else {
    return undefined;
  }
};

const findArrayLength = (propId: string) => {
  const prop = schemasByPropId.value[propId];
  if (prop) {
    if (prop.kind === "array") {
      const arrayValue = _.find(values.value.values, ["propId", propId]);
      if (arrayValue) {
        const childrenOfArray = values.value.childValues[arrayValue.id];
        if (childrenOfArray) {
          return childrenOfArray.length;
        } else {
          return 0;
        }
      } else {
        return undefined;
      }
    } else {
      return undefined;
    }
  } else {
    return undefined;
  }
};

const arrayIndicesByValueId = computed(() => {
  const result: { [valueId: string]: number } = {};
  for (const propValue of Object.values(values.value.values)) {
    const length = findArrayIndex(propValue.id);
    if (!_.isUndefined(length)) {
      result[propValue.id] = length;
    }
  }
  // console.log("array index", { result });
  return result;
});

const arrayLengthsByPropId = computed(() => {
  const result: { [propId: string]: number } = {};
  for (const propId in Object.keys(schemasByPropId.value)) {
    const length = findArrayLength(propId);
    if (!_.isUndefined(length)) {
      result[propId] = length;
    }
  }
  return result;
});
</script>
