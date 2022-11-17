import { defineStore } from "pinia";
import _ from "lodash";
import { watch } from "vue";
import { ApiRequest } from "@/utils/pinia_api_tools";
import { addStoreHooks } from "@/utils/pinia_hooks_plugin";

import {
  PropertyEditorSchema,
  PropertyEditorValidation,
  PropertyEditorValue,
  PropertyEditorValues,
} from "@/api/sdf/dal/property_editor";
import { AttributeContext } from "@/api/sdf/dal/attribute";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useComponentsStore } from "./components.store";

export interface UpdatePropertyEditorValueArgs {
  attributeValueId: number;
  parentAttributeValueId?: number;
  attributeContext: AttributeContext;
  value?: unknown;
  key?: string;
}
export interface InsertPropertyEditorValueArgs {
  parentAttributeValueId: number;
  attributeContext: AttributeContext;
  value?: unknown;
  key?: string;
}

export const useComponentAttributesStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const workspaceId = changeSetsStore.selectedWorkspaceId;

  const visibilityParams = {
    visibility_change_set_pk: changeSetId,
    workspaceId,
  };

  return addStoreHooks(
    defineStore(`cs${changeSetId}/component_attributes`, {
      state: () => ({
        // TODO: likely want to restructure how this data is sent and stored
        // but we'll just move into a pinia store as the first step...
        schema: null as PropertyEditorSchema | null,
        validations: null as PropertyEditorValidation[] | null,
        values: null as PropertyEditorValues | null,
      }),
      getters: {
        currentValueForValueId:
          (state) =>
          (valueId: number): PropertyEditorValue | undefined =>
            state.values?.values[valueId],
        // puts the schema, validations, values all together in a format used by the property editor
        editorContext: (state) => {
          const { schema, validations, values } = state;
          if (!schema || !validations || !values) return undefined;

          // previously called hackAwayTheZeroElementOfContainers - not entirely clear what it's doing
          // can likely refactor how we store/retrieve the data so we wont need this...
          const filteredChildValues: { [key: number]: Array<number> } = {};

          for (const [parentValueId, childValuesIds] of Object.entries(
            values.childValues,
          )) {
            const parentValue = values.values[parseInt(parentValueId)];
            if (!parentValue) {
              // If we don't find a value, then don't filter and continue
              filteredChildValues[parseInt(parentValueId)] = childValuesIds;
              continue;
            }
            const parentProp = schema.props[parentValue.propId];
            if (!parentProp) {
              // If we don't find a prop, then don't filter and continue
              filteredChildValues[parentValue.id] = childValuesIds;
              continue;
            }

            if (parentProp.kind === "array" || parentProp.kind === "map") {
              filteredChildValues[parentValue.id] = childValuesIds.filter(
                (childValueId) => {
                  const childValue = values.values[childValueId];
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
          return {
            schema,
            validations,
            values: {
              ...values,
              childValues: filteredChildValues,
            },
          };
        },

        // getter to be able to quickly grab selected component id
        selectedComponentId: () => {
          const componentsStore = useComponentsStore();
          return componentsStore.selectedComponentId;
        },
      },
      actions: {
        async FETCH_PROPERTY_EDITOR_SCHEMA() {
          return new ApiRequest<PropertyEditorSchema>({
            url: "component/get_property_editor_schema",
            params: {
              componentId: this.selectedComponentId,
              ...visibilityParams,
            },
            onSuccess: (response) => {
              this.schema = response;
            },
          });
        },
        async FETCH_PROPERTY_EDITOR_VALUES() {
          return new ApiRequest<PropertyEditorValues>({
            url: "component/get_property_editor_values",
            params: {
              componentId: this.selectedComponentId,
              ...visibilityParams,
            },
            onSuccess: (response) => {
              this.values = response;
            },
          });
        },
        async FETCH_PROPERTY_EDITOR_VALIDATIONS() {
          return new ApiRequest<{ validations: PropertyEditorValidation[] }>({
            url: "component/get_property_editor_validations",
            params: {
              componentId: this.selectedComponentId,
              ...visibilityParams,
            },
            onSuccess: (response) => {
              this.validations = response.validations;
            },
          });
        },

        reloadPropertyEditorData(skipClearingData = false) {
          // resetting the values here so we dont have unrelated data in the store at the same time
          if (!skipClearingData) {
            this.schema = null;
            this.values = null;
            this.validations = null;
          }
          this.FETCH_PROPERTY_EDITOR_SCHEMA();
          this.FETCH_PROPERTY_EDITOR_VALUES();
          this.FETCH_PROPERTY_EDITOR_VALIDATIONS();
        },

        // combined these 2 api endpoints so they will get tracked under the same key, can revisit this later...
        async UPDATE_PROPERTY_VALUE(
          updatePayload:
            | { update: UpdatePropertyEditorValueArgs }
            | { insert: InsertPropertyEditorValueArgs },
        ) {
          const isInsert = "insert" in updatePayload;
          // If the valueid for this update does not exist in the values tree,
          // we shouldn't perform the update!
          if (
            this.currentValueForValueId(
              isInsert
                ? updatePayload.insert.parentAttributeValueId
                : updatePayload.update.attributeValueId,
            ) === undefined
          ) {
            return;
          }
          return new ApiRequest<{ success: true }>({
            method: "post",
            url: isInsert
              ? "component/insert_property_editor_value"
              : "component/update_property_editor_value",
            params: {
              ...(isInsert ? updatePayload.insert : updatePayload.update),
              ...visibilityParams,
            },
          });
        },
      },
      onActivated() {
        this.reloadPropertyEditorData();

        const realtimeStore = useRealtimeStore();

        const stopWatchSelectedComponent = watch(
          () => this.selectedComponentId,
          () => {
            if (!this.selectedComponentId) return;
            this.reloadPropertyEditorData();
          },
        );

        realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
          {
            eventType: "ChangeSetWritten",
            callback: (writtenChangeSetId) => {
              if (writtenChangeSetId !== changeSetId) return;
              // for now we'll re-fetch everything without clearing the data first
              // but longterm this should be much more targeted and smarter...
              this.reloadPropertyEditorData(true);
            },
          },
        ]);

        return () => {
          stopWatchSelectedComponent();
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
