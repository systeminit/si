<template>
  <div class="flex flex-col w-full overflow-auto h-full">
    <div
      v-for="pv in propertyValuesInOrder"
      :key="pv.id"
      class="flex flex-col w-full"
    >
      <PropertyWidget
        :schema-prop="schemaForPropId(pv.id)"
        :prop-value="pv"
        :path="pathForValueId(pv.id)"
        :collapsed-paths="collapsed"
        @toggle-collapsed="toggleCollapsed($event)"
        @updated-property="updatedProperty($event)"
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
  UpdatedProperty,
} from "@/api/sdf/dal/property_editor";
import { GlobalErrorService } from "@/service/global_error";
import PropertyWidget from "./PropertyEditor/PropertyWidget.vue";
import { ref, computed } from "vue";
import _ from "lodash";

const emits = defineEmits<{
  (e: "updatedProperty", v: UpdatedProperty): void;
}>();

const schema = ref<PropertyEditorSchema>({
  rootPropId: 0,
  props: {
    0: {
      id: 0,
      name: "attributes",
      kind: PropertyEditorPropKind.Object,
      widgetKind: PropertyEditorPropWidgetKind.Header,
    },
    1: {
      id: 1,
      name: "name",
      kind: PropertyEditorPropKind.String,
      widgetKind: PropertyEditorPropWidgetKind.Text,
    },
    2: {
      id: 2,
      name: "love",
      kind: PropertyEditorPropKind.String,
      widgetKind: PropertyEditorPropWidgetKind.Text,
    },
    3: {
      id: 3,
      name: "bleeds",
      kind: PropertyEditorPropKind.String,
      widgetKind: PropertyEditorPropWidgetKind.Text,
    },
    4: {
      id: 4,
      name: "snoop",
      kind: PropertyEditorPropKind.Object,
      widgetKind: PropertyEditorPropWidgetKind.Header,
    },
    5: {
      id: 5,
      name: "lion",
      kind: PropertyEditorPropKind.Object,
      widgetKind: PropertyEditorPropWidgetKind.Header,
    },
    6: {
      id: 6,
      name: "death row",
      kind: PropertyEditorPropKind.String,
      widgetKind: PropertyEditorPropWidgetKind.Text,
    },
    7: {
      id: 7,
      name: "dreams",
      kind: PropertyEditorPropKind.String,
      widgetKind: PropertyEditorPropWidgetKind.Text,
    },
  },
  childProps: {
    0: [1, 2, 3, 4],
    4: [5],
    5: [6, 7],
  },
});

const values = ref<PropertyEditorValues>({
  rootValueId: 0,
  values: {
    0: {
      id: 0,
      propId: 0,
      value: {},
    },
    1: {
      id: 1,
      propId: 1,
      value: "def leppard",
    },
    2: {
      id: 2,
      propId: 2,
      value: "love bites",
    },
    3: {
      id: 3,
      propId: 3,
      value: null,
    },
    4: {
      id: 4,
      propId: 4,
      value: {},
    },
    5: {
      id: 5,
      propId: 5,
      value: {},
    },
    6: {
      id: 6,
      propId: 6,
      value: "too late",
    },
    7: {
      id: 7,
      propId: 7,
      value: "for love",
    },
  },
  childValues: {
    0: [1, 2, 3, 4],
    4: [5],
    5: [6, 7],
  },
});

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

const collapsed = ref<Array<Array<string>>>([["snoop", "attributes"]]);
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

const pathForValueId = (valueId: number) => {
  const path = [];
  let toCheck: string | null = String(valueId);
  CHECK: while (toCheck !== null) {
    for (const [key, childValueIds] of Object.entries(
      values.value.childValues,
    )) {
      for (const childValueId of childValueIds) {
        if (String(childValueId) == toCheck) {
          const parentValue = values.value.values[parseInt(key, 10)];
          if (parentValue) {
            const schemaProp = schema.value.props[parentValue.propId];
            if (parentValue.key) {
              path.push(parentValue.key);
            } else {
              path.push(schemaProp.name);
            }
            toCheck = key;
          } else {
            GlobalErrorService.set({
              error: {
                code: 56,
                message: `missing parent value for value id ${childValueId}; bug!`,
                statusCode: 55,
              },
            });
          }
          continue CHECK;
        }
      }
    }
    toCheck = null;
  }
  return path;
};

const propertyValuesInOrder = computed(() => {
  const results = [];
  const lookup = [values.value.rootValueId];
  while (lookup.length) {
    const nextValueId = lookup.shift();
    if (nextValueId !== undefined) {
      results.push(values.value.values[nextValueId]);
      const childValuesList = values.value.childValues[nextValueId];
      if (childValuesList) {
        for (const childValueId of childValuesList) {
          lookup.push(childValueId);
        }
      }
    }
  }
  return results;
});

const updatedProperty = (event: UpdatedProperty) => {
  values.value.values[event.valueId].value = event.value;
  emits("updatedProperty", event);
};
</script>
