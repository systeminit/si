<template>
  <div class="flex flex-col w-full overflow-auto h-full pb-5">
    <div
      v-for="pv in propertyValuesInOrder"
      :key="pv.id"
      class="flex flex-col w-full"
    >
      <PropertyWidget
        :schema-prop="schemaForPropId(pv.propId)"
        :prop-value="pv"
        :path="paths[pv.id]"
        :collapsed-paths="collapsed"
        :validation="validationForValueId(pv.id)"
        :disabled="disabled"
        :array-index="arrayIndex[pv.id]"
        :array-length="arrayLength[pv.propId]"
        @toggle-collapsed="toggleCollapsed($event)"
        @updated-property="updatedProperty($event)"
        @add-to-array="addToArray($event)"
        @add-to-map="addToMap($event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  PropertyEditorPropKind,
  PropertyEditorPropWidgetKind,
  PropertyEditorSchema,
  PropertyEditorValues,
  PropertyEditorValue,
  PropertyEditorValidations,
  UpdatedProperty,
  AddToArray,
  AddToMap,
  PropertyPath,
} from "@/api/sdf/dal/property_editor";
import { GlobalErrorService } from "@/service/global_error";
import PropertyWidget from "./PropertyEditor/PropertyWidget.vue";
import { ref, computed, toRefs, watch } from "vue";
import _ from "lodash";
import { ChangeSetService } from "@/service/change_set";
import { refFrom } from "vuse-rx";
import { switchMap, from } from "rxjs";

export interface PropertyEditorContext {
  schema: PropertyEditorSchema;
  values: PropertyEditorValues;
  validations: PropertyEditorValidations;
}

const props = defineProps<{
  editorContext: PropertyEditorContext;
}>();

const emits = defineEmits<{
  (e: "updatedProperty", v: UpdatedProperty): void;
  (e: "addToArray", v: AddToArray): void;
  (e: "addToMap", v: AddToMap): void;
}>();

const disabled = refFrom<boolean>(
  ChangeSetService.currentEditMode().pipe(
    switchMap((value) => {
      return from([!value]);
    }),
  ),
);

const { editorContext } = toRefs(props);

const schema = ref<PropertyEditorSchema>(props.editorContext.schema);
const values = ref<PropertyEditorValues>(props.editorContext.values);
const validations = ref<PropertyEditorValidations>(
  props.editorContext.validations,
);
watch(editorContext, (newValue) => {
  schema.value = newValue.schema;
  values.value = newValue.values;
  validations.value = newValue.validations;
});

//const schema = ref<PropertyEditorSchema>({
//  rootPropId: 0,
//  props: {
//    0: {
//      id: 0,
//      name: "attributes",
//      kind: PropertyEditorPropKind.Object,
//      widgetKind: PropertyEditorPropWidgetKind.Header,
//    },
//    1: {
//      id: 1,
//      name: "name",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//      docLink: "http://slashdot.org",
//    },
//    2: {
//      id: 2,
//      name: "love",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//    },
//    3: {
//      id: 3,
//      name: "bleeds",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//    },
//    4: {
//      id: 4,
//      name: "snoop",
//      kind: PropertyEditorPropKind.Object,
//      widgetKind: PropertyEditorPropWidgetKind.Header,
//    },
//    5: {
//      id: 5,
//      name: "lion",
//      kind: PropertyEditorPropKind.Object,
//      widgetKind: PropertyEditorPropWidgetKind.Header,
//    },
//    6: {
//      id: 6,
//      name: "death row",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//    },
//    7: {
//      id: 7,
//      name: "dreams",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//    },
//    8: {
//      id: 8,
//      name: "songs",
//      kind: PropertyEditorPropKind.Array,
//      widgetKind: PropertyEditorPropWidgetKind.Array,
//    },
//    9: {
//      id: 9,
//      name: "song name",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//    },
//    10: {
//      id: 10,
//      name: "books",
//      kind: PropertyEditorPropKind.Array,
//      widgetKind: PropertyEditorPropWidgetKind.Array,
//    },
//    11: {
//      id: 11,
//      name: "book",
//      kind: PropertyEditorPropKind.Object,
//      widgetKind: PropertyEditorPropWidgetKind.Header,
//    },
//    12: {
//      id: 12,
//      name: "author",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//    },
//    13: {
//      id: 13,
//      name: "title",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//    },
//    14: {
//      id: 14,
//      name: "volumes",
//      kind: PropertyEditorPropKind.Array,
//      widgetKind: PropertyEditorPropWidgetKind.Array,
//    },
//    15: {
//      id: 15,
//      name: "volume",
//      kind: PropertyEditorPropKind.Object,
//      widgetKind: PropertyEditorPropWidgetKind.Header,
//    },
//    16: {
//      id: 16,
//      name: "thingyboober",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//    },
//    17: {
//      id: 17,
//      name: "env",
//      kind: PropertyEditorPropKind.Map,
//      widgetKind: PropertyEditorPropWidgetKind.Map,
//    },
//    18: {
//      id: 18,
//      name: "value",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//    },
//    19: {
//      id: 19,
//      name: "slayer songs",
//      kind: PropertyEditorPropKind.Map,
//      widgetKind: PropertyEditorPropWidgetKind.Map,
//    },
//    20: {
//      id: 20,
//      name: "slayer song",
//      kind: PropertyEditorPropKind.Object,
//      widgetKind: PropertyEditorPropWidgetKind.Header,
//    },
//    21: {
//      id: 21,
//      name: "kills it",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//    },
//    22: {
//      id: 22,
//      name: "every time",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//    },
//    23: {
//      id: 23,
//      name: "reasons",
//      kind: PropertyEditorPropKind.Array,
//      widgetKind: PropertyEditorPropWidgetKind.Array,
//    },
//    24: {
//      id: 24,
//      name: "reason",
//      kind: PropertyEditorPropKind.String,
//      widgetKind: PropertyEditorPropWidgetKind.Text,
//    },
//  },
//  childProps: {
//    0: [1, 2, 3, 4, 8, 10, 17, 19],
//    4: [5],
//    5: [6, 7],
//    8: [9],
//    10: [11],
//    11: [12, 13, 14],
//    14: [15],
//    15: [16],
//    17: [18],
//    19: [20],
//    20: [21, 22, 23],
//    23: [24],
//  },
//});
//
//const values = ref<PropertyEditorValues>({
//  rootValueId: 0,
//  values: {
//    0: {
//      id: 0,
//      propId: 0,
//      value: {},
//    },
//    1: {
//      id: 1,
//      propId: 1,
//      value: "def leppard",
//    },
//    2: {
//      id: 2,
//      propId: 2,
//      value: "love bites",
//    },
//    3: {
//      id: 3,
//      propId: 3,
//      value: null,
//    },
//    4: {
//      id: 4,
//      propId: 4,
//      value: {},
//    },
//    5: {
//      id: 5,
//      propId: 5,
//      value: {},
//    },
//    6: {
//      id: 6,
//      propId: 6,
//      value: "too late",
//    },
//    7: {
//      id: 7,
//      propId: 7,
//      value: "for love",
//    },
//    8: {
//      // strings
//      id: 8,
//      propId: 8,
//      value: [],
//    },
//    9: {
//      // books
//      id: 9,
//      propId: 10,
//      value: [],
//    },
//    10: {
//      id: 10,
//      propId: 17,
//      value: {},
//    },
//    11: {
//      id: 11,
//      propId: 19,
//      value: {},
//    },
//  },
//  childValues: {
//    0: [1, 2, 3, 4, 8, 9, 10, 11],
//    4: [5],
//    5: [6, 7],
//  },
//});
//
//const validations = ref<PropertyEditorValidations>({
//  validations: {
//    [1]: {
//      valueId: 1,
//      valid: false,
//      errors: [
//        {
//          message: "You should not be def leppard, dummy",
//        },
//      ],
//    },
//  },
//});

const validationForValueId = (valueId: number) => {
  return validations.value.validations[valueId];
};

const schemaForPropId = (propId: number) => {
  const schemaForProp = schema.value.props[propId];
  if (schemaForProp) {
    return schemaForProp;
  } else {
    GlobalErrorService.set({
      error: {
        code: 55,
        message: `Schema not found for prop ${propId}; bug!`,
        statusCode: 55,
      },
    });
    return {
      id: propId,
      name: "error",
      kind: PropertyEditorPropKind.String,
      widgetKind: PropertyEditorPropWidgetKind.Text,
    };
  }
};

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
  console.log("new collapsed", { collapsed: JSON.stringify(collapsed.value) });
};

const findParentProp = (propId: number) => {
  for (const [parentPropId, childPropIds] of Object.entries(
    schema.value.childProps,
  )) {
    for (const childProp of childPropIds) {
      if (childProp == propId) {
        const parentProp = schema.value.props[parseInt(parentPropId, 10)];
        if (parentProp) {
          return parentProp;
        } else {
          return undefined;
        }
      }
    }
  }
};

const findParentValue = (valueId: number) => {
  for (const [parentValueId, childValueIds] of Object.entries(
    values.value.childValues,
  )) {
    for (const childValueId of childValueIds) {
      if (childValueId == valueId) {
        const parentValue = values.value.values[parseInt(parentValueId, 10)];
        if (parentValue) {
          return parentValue;
        } else {
          return undefined;
        }
      }
    }
  }
};

const pathPartForValueId = (valueId: number) => {
  let displayPathPart = "bug";
  let triggerPathPart = "bug";
  const currentValue = values.value.values[valueId];
  if (!currentValue) {
    return "bugbug returns!";
  }
  const currentProp = schema.value.props[currentValue.propId];
  const parentProp = findParentProp(currentValue.propId);

  if (currentValue && currentProp) {
    if (parentProp) {
      if (parentProp.kind == "array") {
        const index = findArrayIndex(currentValue.id);
        if (!_.isUndefined(index)) {
          if (currentProp.kind == "object") {
            displayPathPart = `[${index}](${currentProp.name})`;
            triggerPathPart = `[${index}]`;
          } else {
            displayPathPart = `[${index}]`;
            triggerPathPart = `[${index}]`;
          }
          return { displayPathPart, triggerPathPart };
        }
      } else if (parentProp.kind == "map") {
        if (currentProp.kind == "object") {
          displayPathPart = `{${currentValue.key}}(${currentProp.name})`;
          triggerPathPart = `{${currentValue.key}}`;
        } else {
          displayPathPart = `{${currentValue.key}}`;
          triggerPathPart = `{${currentValue.key}}`;
        }
        return { displayPathPart, triggerPathPart };
      }
    }
    if (currentProp.kind == "array") {
      const childCount = values.value.childValues[valueId];
      let arrayLength = childCount ? childCount.length : 0;
      displayPathPart = `${currentProp.name}[${arrayLength}]`;
      triggerPathPart = `${currentProp.name}[]`;
    } else if (currentProp.kind == "object") {
      displayPathPart = currentProp.name;
      triggerPathPart = currentProp.name;
    } else if (currentProp.kind == "map") {
      const childCount = values.value.childValues[valueId];
      let mapLength = childCount ? childCount.length : 0;
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
  valueId: number,
) => {
  for (const [parentValueId, childValueIds] of Object.entries(
    values.value.childValues,
  )) {
    for (const childValueId of childValueIds) {
      if (childValueId == valueId) {
        const pathPart = pathPartForValueId(parseInt(parentValueId, 10));
        displayPath.push(pathPart.displayPathPart);
        triggerPath.push(pathPart.triggerPathPart);
        findParentPath(displayPath, triggerPath, parseInt(parentValueId, 10));
      }
    }
  }
};

const paths = computed<{ [valueId: number]: PropertyPath | undefined }>(() => {
  const result: { [valueId: number]: PropertyPath } = {};
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
  childValueIds: number[],
): PropertyEditorValue[] => {
  for (const childValueId of childValueIds) {
    const child = values.value.values[childValueId];
    order.push(child);
    const childValuesList = values.value.childValues[childValueId];
    if (childValuesList) {
      determineOrder(order, childValuesList);
    }
  }
  return order;
};

const propertyValuesInOrder = computed(() => {
  const results = determineOrder([], [values.value.rootValueId]);

  console.log("property results", { results });
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

const findArrayIndex = (valueId: number) => {
  let parentProp;
  let index;
  for (const [parentValueId, childValues] of Object.entries(
    values.value.childValues,
  )) {
    for (let x = 0; x < childValues.length; x++) {
      const cv = childValues[x];
      if (cv == valueId) {
        index = x;
        const parentValue = values.value.values[parseInt(parentValueId, 10)];
        if (parentValue) {
          parentProp = schema.value.props[parentValue.propId];
        }
        break;
      }
    }
  }
  console.log("lets try this", { index, parentProp, valueId });
  if (parentProp?.kind == "array") {
    return index;
  } else {
    return undefined;
  }
};

const findArrayLength = (propId: number) => {
  const prop = schema.value.props[propId];
  if (prop) {
    if (prop.kind == "array") {
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

const arrayIndex = computed(() => {
  const result: { [valueId: number]: number } = {};
  for (const propValue of Object.values(values.value.values)) {
    const length = findArrayIndex(propValue.id);
    if (!_.isUndefined(length)) {
      result[propValue.id] = length;
    }
  }
  console.log("array index", { result });
  return result;
});

const arrayLength = computed(() => {
  const result: { [propId: number]: number } = {};
  for (const propId in Object.keys(schema.value.props)) {
    const length = findArrayLength(parseInt(propId, 10));
    if (!_.isUndefined(length)) {
      result[propId] = length;
    }
  }
  return result;
});
</script>
