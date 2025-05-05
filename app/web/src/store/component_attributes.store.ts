import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";

import { useWorkspacesStore } from "@/store/workspaces.store";
import {
  PropertyEditorProp,
  PropertyEditorSchema,
  PropertyEditorValue,
  PropertyEditorValues,
  ValidationOutput,
} from "@/api/sdf/dal/property_editor";
import { ComponentId } from "@/api/sdf/dal/component";
import { ComponentType } from "@/api/sdf/dal/schema";
import handleStoreError from "./errors";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useComponentsStore } from "./components.store";

export interface UpdatePropertyEditorValueArgs {
  attributeValueId: string;
  parentAttributeValueId?: string;
  propId: string;
  componentId: string;
  value?: unknown;
  key?: string;
  isForSecret: boolean;
}

export interface InsertPropertyEditorValueArgs {
  parentAttributeValueId: string;
  propId: string;
  componentId: string;
  value?: unknown;
  key?: string;
}

export interface DeletePropertyEditorValueArgs {
  attributeValueId: string;
  propId: string;
  componentId: string;
  value?: unknown;
  key?: string;
}

export interface ResetPropertyEditorValueArgs {
  attributeValueId: string;
}

export interface SetTypeArgs {
  componentId: string;
  componentType: ComponentType;
}

export interface OutputStream {
  stream: string;
  level: string;
  group: string | null;
  message: string;
}

export type AttributeTreeItem = {
  propDef: PropertyEditorProp;
  children: AttributeTreeItem[];
  value: PropertyEditorValue | undefined;
  valueId: string;
  parentValueId: string;
  validation: ValidationOutput | null;
  propId: string;
  mapKey?: string;
  arrayKey?: string;
  arrayIndex?: number;
};

export const useComponentAttributesStore = (componentId: ComponentId) => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;

  const visibilityParams = {
    visibility_change_set_pk: changeSetId,
  };
  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;
  const realtimeStore = useRealtimeStore();

  return addStoreHooks(
    workspaceId,
    changeSetId,
    defineStore(
      `ws${
        workspaceId || "NONE"
      }/cs${changeSetId}/c${componentId}/component_attributes`,
      {
        state: () => ({
          // TODO: likely want to restructure how this data is sent and stored
          // but we'll just move into a pinia store as the first step...
          schema: null as PropertyEditorSchema | null,
          values: null as PropertyEditorValues | null,
        }),
        getters: {
          // recombine the schema + values + validations into a single nested tree that can be used by the attributes panel
          attributesTree: (state): AttributeTreeItem | undefined => {
            if (!state.schema || !state.values) return;

            const valuesByValueId = state.values.values;
            const propsByPropId = state.schema.props;
            const rootValueId = state.values.rootValueId;

            if (!valuesByValueId || !propsByPropId || !rootValueId) return;

            function getAttributeValueWithChildren(
              valueId: string,
              parentValueId: string,
              ancestorManual = true,
              indexInParentArray?: number,
            ): AttributeTreeItem | undefined {
              /* eslint-disable @typescript-eslint/no-non-null-assertion,@typescript-eslint/no-explicit-any */
              const value = valuesByValueId![valueId]!;

              const propDef = propsByPropId![value.propId as any];
              const validation = value?.validation ?? null;

              // some values that we see are for props that are hidden, so we filter them out
              if (!propDef) return;

              value.ancestorManual = ancestorManual;
              const isAncestorManual =
                ancestorManual &&
                !value.isControlledByDynamicFunc &&
                !(value.canBeSetBySocket || value.isFromExternalSource);

              return {
                propDef,
                value,
                valueId,
                parentValueId,
                validation,
                // using isNil because its actually null (not undefined)
                ...(indexInParentArray === undefined &&
                  !_.isNil(value.key) && { mapKey: value.key }),
                ...(indexInParentArray !== undefined && {
                  arrayIndex: indexInParentArray,
                  arrayKey: value.key,
                }),
                propId: value.propId,
                children: _.compact(
                  _.map(state.values?.childValues[valueId], (cvId, index) =>
                    getAttributeValueWithChildren(
                      cvId,
                      valueId,
                      isAncestorManual,
                      propDef.kind === "array" ? index : undefined,
                    ),
                  ),
                ),
              };
            }

            // dummy parent root value id - not used by anything
            return getAttributeValueWithChildren(rootValueId, "ROOT");
          },
          domainTree(): AttributeTreeItem | undefined {
            if (!this.attributesTree) return undefined;
            return _.find(
              this.attributesTree.children,
              (c) => c.propDef.name === "domain",
            );
          },
          secretsTree(): AttributeTreeItem | undefined {
            if (!this.attributesTree) return undefined;
            return _.find(
              this.attributesTree.children,
              (c) => c.propDef.name === "secrets",
            );
          },
          resourceValueTree(): AttributeTreeItem | undefined {
            if (!this.attributesTree) return undefined;
            return _.find(
              this.attributesTree.children,
              (c) => c.propDef.name === "resource_value",
            );
          },
          siTreeByPropName(): Record<string, AttributeTreeItem> | undefined {
            if (!this.attributesTree) return undefined;
            const siTree = _.find(
              this.attributesTree.children,
              (c) => c.propDef.name === "si",
            );
            return _.keyBy(siTree?.children, (prop) => prop.propDef.name);
          },

          // getter to be able to quickly grab selected component id
          selectedComponentId: () => componentId,
          selectedComponent: () => {
            if (!componentId) return;
            const componentsStore = useComponentsStore();
            return componentsStore.allComponentsById[componentId];
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
                if (this.selectedComponent === undefined) {
                  this.schema = response;
                  return;
                }

                const props: { [id: string]: PropertyEditorProp } = {};

                for (const propKey in response.props) {
                  const prop = response.props[propKey];
                  if (prop) {
                    const isHidden =
                      prop.name === "type" &&
                      this.selectedComponent.def.schemaName === "Generic Frame";
                    const isReadonly =
                      prop.name === "type" &&
                      this.selectedComponent.def.childIds !== undefined &&
                      this.selectedComponent.def.childIds.length > 0;

                    props[propKey] = {
                      ...prop,
                      isHidden,
                      isReadonly,
                    };
                  }
                }

                this.schema = { ...response, props };
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

          reloadPropertyEditorData() {
            this.FETCH_PROPERTY_EDITOR_SCHEMA();
            this.FETCH_PROPERTY_EDITOR_VALUES();
          },

          async REMOVE_PROPERTY_VALUE(
            removePayload: DeletePropertyEditorValueArgs,
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            return new ApiRequest<{ success: true }>({
              method: "post",
              url: "component/delete_property_editor_value",
              params: {
                ...removePayload,
                ...visibilityParams,
              },
            });
          },

          // combined these 2 api endpoints so they will get tracked under the same key, can revisit this later...
          async UPDATE_PROPERTY_VALUE(
            updatePayload:
              | { update: UpdatePropertyEditorValueArgs }
              | { insert: InsertPropertyEditorValueArgs },
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            const isInsert = "insert" in updatePayload;

            if (!isInsert) {
              const propId = updatePayload.update.propId;
              const prop = this.schema?.props[propId];
              if (
                prop?.kind === "json" &&
                typeof updatePayload.update.value === "string"
              ) {
                try {
                  updatePayload.update.value = JSON.parse(
                    updatePayload.update.value,
                  );
                } catch (error) {
                  window.reportError(error);
                }
              }
            }

            // If the valueid for this update does not exist in the values tree,
            // we shouldn't perform the update!
            if (
              this.values?.values[
                isInsert
                  ? updatePayload.insert.parentAttributeValueId
                  : updatePayload.update.attributeValueId
              ] === undefined
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
          async SET_COMPONENT_TYPE(payload: SetTypeArgs) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;

            // NOTE Since views came in overriding geometries on this operation
            // became way more complex. Also frames start at the size of the
            // original component so this is not going to be a problem for now.

            // Make sure the component will not be bigger that its parent
            // let overriddenGeometry: APIComponentGeometry | undefined;
            // const componentStore = useComponentsStore();
            // const component =
            //   componentStore.allComponentsById[payload.componentId];
            //
            // if (!component)
            //   throw new Error("Could not find component in store");
            //
            // if (
            //   component.def.parentId &&
            //   payload.componentType !== ComponentType.Component
            // ) {
            //   const parent =
            //     componentStore.allComponentsById[component.def.parentId];
            //   if (!parent) throw new Error("Could not find parent in store");
            //
            //   const viewStore = useViewsStore();
            //   const componentGeometry = component.def.isGroup
            //     ? viewStore.groups[component.def.id]
            //     : viewStore.components[component.def.id];
            //
            //   if (!componentGeometry)
            //     throw new Error("Could not rendered geometry for component");
            //
            //   const parentGeometry = viewStore.groups[parent.def.id];
            //
            //   if (!parentGeometry)
            //     throw new Error("Could not rendered geometry for parent");
            //
            //   // Assuming that the component already fits in the parent
            //   // we need to shrink the group until it fits the parent
            //   // For the x-axis
            //   const originalLeft =
            //     componentGeometry.x - componentGeometry.width / 2;
            //   const containerLeft =
            //     parentGeometry.x -
            //     parentGeometry.width / 2 +
            //     GROUP_INTERNAL_PADDING;
            //
            //   const newLeft = Math.max(originalLeft, containerLeft);
            //
            //   const originalRight =
            //     componentGeometry.x + componentGeometry.width / 2;
            //   const containerRight =
            //     parentGeometry.x +
            //     parentGeometry.width / 2 -
            //     GROUP_INTERNAL_PADDING;
            //
            //   const newRight = Math.min(originalRight, containerRight);
            //
            //   const newWidth = newRight - newLeft;
            //   const newX = newLeft + newWidth / 2;
            //
            //   // For the y-axis
            //   const originalTop = componentGeometry.y;
            //   const containerTop = parentGeometry.y + GROUP_INTERNAL_PADDING;
            //
            //   const newTop = Math.max(originalTop, containerTop);
            //
            //   const originalBottom =
            //     componentGeometry.y + componentGeometry.height;
            //   const containerBottom =
            //     parentGeometry.y +
            //     parentGeometry.height -
            //     GROUP_BOTTOM_INTERNAL_PADDING;
            //
            //   const newBottom = Math.min(originalBottom, containerBottom);
            //
            //   const newHeight = newBottom - newTop;
            //   const newY = newTop;
            //
            //   overriddenGeometry = {
            //     x: Math.round(newX).toString(),
            //     y: Math.round(newY).toString(),
            //     width: Math.round(newWidth).toString(),
            //     height: Math.round(newHeight).toString(),
            //   };
            // }

            return new ApiRequest<{ success: true }>({
              method: "post",
              url: "component/set_type",
              params: {
                ...payload,
                ...visibilityParams,
              },
            });
          },
          async RESET_PROPERTY_VALUE(
            resetPayload: ResetPropertyEditorValueArgs,
          ) {
            if (changeSetsStore.creatingChangeSet)
              throw new Error("race, wait until the change set is created");
            if (changeSetId === changeSetsStore.headChangeSetId)
              changeSetsStore.creatingChangeSet = true;
            return new ApiRequest<{ success: true }>({
              method: "post",
              url: "component/restore_default_function",
              params: {
                ...resetPayload,
                ...visibilityParams,
              },
            });
          },
          registerRequestsBegin(requestUlid: string, actionName: string) {
            realtimeStore.inflightRequests.set(requestUlid, actionName);
          },
          registerRequestsEnd(requestUlid: string) {
            realtimeStore.inflightRequests.delete(requestUlid);
          },
        },
        onActivated() {
          // PSA: special case, this data loading can stay here
          this.reloadPropertyEditorData();

          realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
            {
              eventType: "ComponentUpdated",
              callback: (updated) => {
                if (updated.changeSetId !== changeSetId) return;
                if (updated.component.id !== this.selectedComponentId) return;
                this.reloadPropertyEditorData();
              },
            },
          ]);
          realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
            {
              eventType: "ChangeSetWritten",
              callback: (writtenChangeSetId) => {
                if (writtenChangeSetId !== changeSetId) return;
                this.reloadPropertyEditorData();
              },
            },
            {
              eventType: "ChangeSetApplied",
              callback: (data) => {
                // If the applied change set has rebased into this change set,
                // then refetch (i.e. there might be updates!)
                if (data.toRebaseChangeSetId === changeSetId) {
                  this.reloadPropertyEditorData();
                }
              },
            },
          ]);

          const actionUnsub = this.$onAction(handleStoreError);

          return () => {
            actionUnsub();
            realtimeStore.unsubscribe(this.$id);
          };
        },
      },
    ),
  )();
};
