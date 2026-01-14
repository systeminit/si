import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { FuncId, FuncKind } from "@/api/sdf/dal/func";
import { SchemaId, SchemaVariant, SchemaVariantId, UninstalledVariant } from "@/api/sdf/dal/schema";
import { Visibility } from "@/api/sdf/dal/visibility";
import keyedDebouncer from "@/utils/keyedDebouncer";
import router from "@/router";
import { PropKind } from "@/api/sdf/dal/prop";
import { nonNullable } from "@/utils/typescriptLinter";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { useFuncStore } from "./func/funcs.store";
import { useChangeSetsStore } from "./change_sets.store";
import { useModuleStore } from "./module.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import handleStoreError from "./errors";
import { useRouterStore } from "./router.store";
import { useAuthStore } from "./auth.store";

export interface InstalledPkgAssetView {
  assetId: string;
  assetHash: string;
  assetKind: string;
}

export type DetachedAttributePrototypeKind =
  | {
      type: "OutputSocketSocket";
      data: {
        name: string;
        kind: "ConfigurationInput" | "ConfigurationOutput";
      };
    }
  | {
      type: "InputSocketSocket";
      data: {
        name: string;
        kind: "ConfigurationInput" | "ConfigurationOutput";
      };
    }
  | { type: "InputSocketProp"; data: { path: string; kind: PropKind } }
  | { type: "Prop"; data: { path: string; kind: PropKind } };

export interface DetachedAttributePrototype {
  id: string;
  funcId: FuncId;
  funcName: string;
  key: string | null;
  kind: FuncKind;
  context: DetachedAttributePrototypeKind;
}

export interface DetachedValidationPrototype {
  id: string;
  funcId: FuncId;
  funcName: string;
  args: unknown;
  link: string | null;
  propPath: string;
  propKind: PropKind;
}

export type SchemaVariantSaveRequest = Visibility & { code?: string } & {
  variant: Omit<SchemaVariant, "created_at" | "updated_at">;
};
export type SchemaVariantCreateRequest = { name: string; color: string };
export type SchemaVariantCloneRequest = Visibility & {
  id: SchemaVariantId;
  name: string;
};

export const schemaVariantDisplayName = (schemaVariant: SchemaVariant) =>
  (schemaVariant.displayName ?? "").length === 0 ? schemaVariant.schemaName : schemaVariant.displayName;

export const useAssetStore = (forceChangeSetId?: ChangeSetId) => {
  const changeSetStore = useChangeSetsStore();
  let changeSetId: ChangeSetId | undefined;
  if (forceChangeSetId) {
    changeSetId = forceChangeSetId;
  } else {
    changeSetId = changeSetStore.selectedChangeSetId;
  }
  const visibility = {
    // changeSetId should not be empty if we are actually using this store
    // so we can give it a bad value and let it throw an error
    visibility_change_set_pk: changeSetId || "XXX",
  };

  const workspaceStore = useWorkspacesStore();
  const workspaceId = workspaceStore.selectedWorkspacePk;

  const changeSetsStore = useChangeSetsStore();
  const selectedChangeSetId = changeSetsStore.selectedChangeSet?.id;

  const funcStore = useFuncStore();
  const moduleStore = useModuleStore();
  const routerStore = useRouterStore();
  const authStore = useAuthStore();
  const realtimeStore = useRealtimeStore();

  let assetSaveDebouncer: ReturnType<typeof keyedDebouncer> | undefined;

  const API_PREFIX = ["v2", "workspaces", { workspaceId }, "change-sets", { selectedChangeSetId }, "schema-variants"];

  return addStoreHooks(
    workspaceId,
    selectedChangeSetId,
    defineStore(`ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/asset`, {
      state: () => ({
        variantList: [] as SchemaVariant[],
        uninstalledVariantList: [] as UninstalledVariant[],

        executeSchemaVariantTaskId: undefined as string | undefined,
        executeSchemaVariantTaskRunning: false as boolean,
        executeSchemaVariantTaskError: undefined as string | undefined,

        editingFuncLatestCode: {} as Record<SchemaVariantId, string>,
        codeSaveIsDebouncing: false,

        // represents state of the left rail lists and all open editor tabs
        selectedSchemaVariants: [] as SchemaVariantId[],
        selectedFuncs: [] as FuncId[],

        detachmentWarnings: [] as {
          message: string;
          funcId: FuncId;
          kind?: FuncKind;
        }[],
      }),
      getters: {
        variantFromListById: (state) => _.keyBy(state.variantList, (a) => a.schemaVariantId),
        schemaVariants: (state) => state.variantList,
        selectedVariantId(state): SchemaVariantId | undefined {
          if (state.selectedSchemaVariants.length === 1) return state.selectedSchemaVariants[0];
          else return undefined;
        },
        selectedSchemaVariant(): SchemaVariant | undefined {
          if (this.selectedVariantId) return this.variantFromListById[this.selectedVariantId];
        },
        selectedSchemaVariantRecords(): SchemaVariant[] {
          return this.selectedSchemaVariants.map((id) => this.variantFromListById[id]).filter(nonNullable);
        },
        selectedFuncId(state): FuncId | undefined {
          if (state.selectedFuncs.length === 1) return state.selectedFuncs[0];
          else return undefined;
        },
        unlockedVariantIdForId: (state) => {
          type VariantsBySchemaId = Record<SchemaId, SchemaVariantId>;
          const unlockedVariantsBySchema: VariantsBySchemaId = state.variantList
            .filter((v) => !v.isLocked)
            .reduce((obj, v) => {
              obj[v.schemaId] = v.schemaVariantId;
              return obj;
            }, {} as VariantsBySchemaId);

          return state.variantList.reduce((obj, v) => {
            obj[v.schemaVariantId] = unlockedVariantsBySchema[v.schemaId];
            return obj;
          }, {} as Record<SchemaVariantId, SchemaVariantId | undefined>);
        },
        schemaVariantOptionsUnlocked: (state): { label: string; value: string }[] => {
          return state.variantList
            .filter((v) => !v.isLocked)
            .map((sv) => ({
              label: sv.displayName || sv.schemaName,
              value: sv.schemaVariantId,
            }));
        },
        schemaVariantOptions: (state): { label: string; value: string }[] => {
          return state.variantList.map((sv) => ({
            label: sv.displayName || sv.schemaName,
            value: sv.schemaVariantId,
          }));
        },
      },
      actions: {
        addSchemaVariantSelection(id: SchemaVariantId) {
          if (!this.selectedSchemaVariants.includes(id)) {
            this.selectedSchemaVariants.push(id);
            // we don't load schema variant here, because we aren't showing the editor
            this.syncSelectionIntoUrl();
            this.selectedFuncs = [];
          }
        },
        removeSchemaVariantSelection(id: SchemaVariantId) {
          const thisAssetIdx = this.selectedSchemaVariants.findIndex((thisId) => thisId === id);
          if (thisAssetIdx !== -1) {
            this.selectedSchemaVariants.splice(thisAssetIdx, 1);
            this.syncSelectionIntoUrl();
          }
        },
        clearSchemaVariantSelection() {
          this.setFuncSelection(undefined);
          this.selectedSchemaVariants = [];
          this.syncSelectionIntoUrl();
        },
        setSchemaVariantSelection(id: SchemaVariantId) {
          this.setFuncSelection(undefined);

          if (this.selectedSchemaVariants.length === 1 && this.selectedSchemaVariants[0] === id) {
            return;
          }
          if (!this.selectedSchemaVariants.includes(id)) {
            this.selectedFuncs = [];
          }
          this.selectedSchemaVariants = [id];
          this.syncSelectionIntoUrl();
          const variant = this.variantFromListById[id];
          if (variant?.assetFuncId) funcStore.FETCH_CODE(variant.assetFuncId);
        },
        async setFuncSelection(id?: FuncId) {
          // ignore the old func selections and replace with one func or no funcs
          funcStore.selectedFuncId = id;
          if (id) {
            await funcStore.FETCH_CODE(id);
            this.selectedFuncs = [id];
          } else {
            this.selectedFuncs = [];
          }
          this.syncSelectionIntoUrl();
        },
        async addFuncSelection(id: FuncId) {
          if (!this.selectedFuncs.includes(id)) this.selectedFuncs.push(id);
          await funcStore.FETCH_CODE(id);
          funcStore.selectedFuncId = id;
          this.syncSelectionIntoUrl();
        },
        removeFuncSelection(id: FuncId) {
          const idx = this.selectedFuncs.indexOf(id);
          if (idx !== -1) this.selectedFuncs.splice(idx, 1);
          this.syncSelectionIntoUrl();
        },
        syncSelectionIntoUrl(returnQuery?: boolean) {
          let selectedIds: string[] = [];
          selectedIds = _.map(this.selectedSchemaVariants, (id) => `a_${id}`);
          selectedIds = selectedIds.concat(_.map(this.selectedFuncs, (id) => `f_${id}`));

          const newQueryObj = {
            ...(selectedIds.length && { s: selectedIds.join("|") }),
          };
          if (returnQuery) return newQueryObj;

          if (!_.isEqual(routerStore.currentRoute?.query, newQueryObj)) {
            routerStore.replace(changeSetId, {
              params: { ...routerStore.currentRoute?.params },
              query: newQueryObj,
            });
          }
        },
        async syncUrlIntoSelection() {
          this.selectedSchemaVariants = [];
          this.selectedFuncs = [];
          funcStore.selectedFuncId = undefined;
          const ids = ((router.currentRoute.value.query?.s as string) || "").split("|").filter(Boolean);
          if (ids.length > 0) {
            /* eslint-disable @typescript-eslint/no-explicit-any */
            const promises: Promise<any>[] = [];
            const fnIds = [] as FuncId[];
            ids.sort().forEach((id) => {
              if (id.startsWith("a_")) {
                id = id.substring(2);
                this.selectedSchemaVariants.push(id);
                const variant = this.variantFromListById[id];
                if (variant?.assetFuncId) promises.push(funcStore.FETCH_CODE(variant.assetFuncId));
              } else if (id.startsWith("f_")) {
                id = id.substring(2);
                this.selectedFuncs.push(id);
                promises.push(funcStore.FETCH_CODE(id));
                fnIds.push(id);
              }
            });
            await Promise.all(promises);
            funcStore.selectedFuncId = fnIds[fnIds.length - 1];
          }
        },

        // MOCK DATA GENERATION
        generateMockColor() {
          return `#${_.sample(["FF0000", "FFFF00", "FF00FF", "00FFFF", "FFAA00", "AAFF00", "00FFAA", "00AAFF"])}`;
        },

        replaceFuncForVariant(schemaVariantId: SchemaVariantId, oldFuncId: FuncId, newFuncId: FuncId) {
          // TODO assets should be represented in a single state variable, we shouldn't be doing this operation twice here
          // Copy bindings for list variant
          const listVariantIdx = this.variantList.findIndex((v) => v.schemaVariantId === schemaVariantId);
          const listVariant = this.variantList[listVariantIdx];

          if (!listVariant) {
            // eslint-disable-next-line no-console
            console.warn(
              `could not find variant ${schemaVariantId}, for binding with ${newFuncId} (previously ${oldFuncId})`,
            );
            return;
          }

          const funcIdxForListVariant = listVariant.funcIds.findIndex((f) => f === oldFuncId);

          if (funcIdxForListVariant === -1) {
            listVariant.funcIds.push(newFuncId);
          } else {
            listVariant.funcIds[funcIdxForListVariant] = newFuncId;
          }
        },

        async CREATE_VARIANT(name: string) {
          if (changeSetStore.creatingChangeSet) throw new Error("race, wait until the change set is created");
          if (changeSetId === changeSetStore.headChangeSetId) changeSetStore.creatingChangeSet = true;
          return new ApiRequest<SchemaVariant, SchemaVariantCreateRequest>({
            method: "post",
            url: "/variant/create_variant",
            params: {
              ...visibility,
              name,
              color: this.generateMockColor(),
            },
            onSuccess: (variant) => {
              const savedAssetIdx = this.variantList.findIndex((a) => a.schemaVariantId === variant.schemaVariantId);
              if (savedAssetIdx === -1) this.variantList.push(variant);
              else this.variantList.splice(savedAssetIdx, 1, variant);
            },
          });
        },

        async CLONE_VARIANT(schemaVariantId: SchemaVariantId, name: string) {
          if (changeSetStore.creatingChangeSet) throw new Error("race, wait until the change set is created");
          if (changeSetStore.headSelected) changeSetStore.creatingChangeSet = true;

          return new ApiRequest<SchemaVariant, SchemaVariantCloneRequest>({
            method: "post",
            keyRequestStatusBy: schemaVariantId,
            url: "/variant/clone_variant",
            params: {
              ...visibility,
              id: schemaVariantId,
              name,
            },
          });
        },

        enqueueVariantSave(schemaVariant: SchemaVariant, code: string, debounce: boolean) {
          if (!debounce) {
            return this.SAVE_SCHEMA_VARIANT(schemaVariant, code);
          }

          this.codeSaveIsDebouncing = true;
          this.editingFuncLatestCode[schemaVariant.schemaVariantId] = code;

          // don't see how this should ever happen
          /* if (changeSetsStore.headSelected)
            return this.SAVE_SCHEMA_VARIANT(schemaVariant, code); */

          if (!assetSaveDebouncer) {
            assetSaveDebouncer = keyedDebouncer((id: SchemaVariantId) => {
              const variant = this.variantFromListById[id];
              if (!variant) return;
              const code = this.editingFuncLatestCode[variant.schemaVariantId];

              if (!code) throw Error(`No asset code for variant ${variant.schemaVariantId}`);

              this.SAVE_SCHEMA_VARIANT(variant, code);
              this.codeSaveIsDebouncing = false;
            }, 1000);
          }
          const assetSaveFunc = assetSaveDebouncer(schemaVariant.schemaVariantId);
          if (assetSaveFunc) {
            assetSaveFunc(schemaVariant.schemaVariantId);
          }
        },

        async SAVE_SCHEMA_VARIANT(schemaVariant: SchemaVariant, code?: string) {
          if (changeSetStore.creatingChangeSet) throw new Error("race, wait until the change set is created");
          if (changeSetStore.headSelected) changeSetStore.creatingChangeSet = true;

          if (schemaVariant.isLocked)
            throw new Error(
              `cant save locked schema variant (${schemaVariant.displayName},${schemaVariant.schemaVariantId})`,
            );

          return new ApiRequest<{ success: boolean; assetFuncId: FuncId }, SchemaVariantSaveRequest>({
            method: "post",
            keyRequestStatusBy: schemaVariant.schemaVariantId,
            url: "/variant/save_variant",
            params: {
              ...visibility,
              code,
              variant: schemaVariant,
            },
            onSuccess: () => {
              if (code) {
                const f = funcStore.funcCodeById[schemaVariant.assetFuncId];
                if (f) f.code = code;
              }
            },
          });
        },
        async REGENERATE_VARIANT(schemaVariantId: SchemaVariantId) {
          if (changeSetStore.creatingChangeSet) throw new Error("race, wait until the change set is created");
          if (changeSetStore.headSelected) changeSetStore.creatingChangeSet = true;

          this.detachmentWarnings = [];
          const variant = this.variantFromListById[schemaVariantId];
          if (!variant) throw new Error(`${schemaVariantId} Variant does not exist`);

          return new ApiRequest<{
            schemaVariantId: SchemaVariantId;
          }>({
            method: "post",
            url: "/variant/regenerate_variant",
            keyRequestStatusBy: schemaVariantId,
            params: {
              ...visibility,
              variant,
            },
          });
        },

        async LOAD_SCHEMA_VARIANT_LIST() {
          return new ApiRequest<{ installed: SchemaVariant[]; uninstalled: UninstalledVariant[] }, Visibility>({
            url: API_PREFIX,
            params: { ...visibility },
            onSuccess: (response) => {
              this.variantList = response.installed;
              this.uninstalledVariantList = response.uninstalled;
            },
          });
        },

        async CREATE_UNLOCKED_COPY(id: SchemaVariantId) {
          if (changeSetStore.creatingChangeSet) throw new Error("race, wait until the change set is created");
          if (changeSetStore.headSelected) changeSetStore.creatingChangeSet = true;

          this.detachmentWarnings = [];

          return new ApiRequest<SchemaVariant>({
            method: "post",
            url: API_PREFIX.concat([id]),
            keyRequestStatusBy: id,
          });
        },
        async DELETE_UNLOCKED_VARIANT(id: SchemaVariantId) {
          if (changeSetStore.creatingChangeSet) throw new Error("race, wait until the change set is created");
          if (changeSetStore.headSelected) changeSetStore.creatingChangeSet = true;

          return new ApiRequest<SchemaVariant>({
            method: "delete",
            url: API_PREFIX.concat([id]),
            keyRequestStatusBy: id,
            params: {
              // ...visibility,
            },
            onSuccess: (variant) => {
              const deletedVariantIdx = this.variantList.findIndex(
                (a) => a.schemaVariantId === variant.schemaVariantId,
              );
              if (deletedVariantIdx !== -1) this.variantList.splice(deletedVariantIdx, 1, variant);
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
      async onActivated() {
        realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
          {
            eventType: "SchemaVariantCreated",
            callback: (variant, metadata) => {
              if (metadata.change_set_id !== changeSetId) return;
              const savedAssetIdx = this.variantList.findIndex((a) => a.schemaVariantId === variant.schemaVariantId);
              if (savedAssetIdx === -1) this.variantList.push(variant);
              else this.variantList.splice(savedAssetIdx, 1, variant);

              const actionWhichCreatedView = realtimeStore.inflightRequests.get(metadata.request_ulid);
              if (
                metadata.actor !== "System" &&
                metadata.actor.User === authStore.userPk &&
                actionWhichCreatedView !== "CREATE_TEMPLATE_FUNC_FROM_COMPONENTS"
              ) {
                this.setSchemaVariantSelection(variant.schemaVariantId);
              }
            },
          },
          {
            eventType: "SchemaVariantDeleted",
            callback: (data) => {
              if (data.changeSetId !== changeSetId) return;
              const savedAssetIdx = this.variantList.findIndex((a) => a.schemaVariantId === data.schemaVariantId);
              this.variantList.splice(savedAssetIdx, 1);
            },
          },
          {
            eventType: "SchemaVariantCloned",
            callback: async (data, metadata) => {
              // TODO: ship the actual variant
              if (data.changeSetId !== changeSetId) return;
              await Promise.all([this.LOAD_SCHEMA_VARIANT_LIST(), moduleStore.SYNC()]);
              if (metadata.actor !== "System" && metadata.actor.User === authStore.userPk) {
                this.setSchemaVariantSelection(data.schemaVariantId);
              }
            },
          },
          {
            eventType: "SchemaVariantSaved",
            callback: (data) => {
              if (data.changeSetId !== changeSetId) return;
              const savedAssetIdx = this.variantList.findIndex((a) => a.schemaVariantId === data.schemaVariantId);
              const savedAsset = this.variantList[savedAssetIdx];
              if (savedAsset) {
                savedAsset.schemaName = data.name;
                savedAsset.category = data.category;
                savedAsset.color = data.color;
                savedAsset.componentType = data.componentType;
                savedAsset.displayName = data.displayName || null;
                this.variantList.splice(savedAssetIdx, 1, savedAsset);
              }

              const existingAsset = this.variantFromListById[data.schemaVariantId];
              if (existingAsset) {
                existingAsset.schemaName = data.name;
                existingAsset.category = data.category;
                existingAsset.color = data.color;
                existingAsset.componentType = data.componentType;
                existingAsset.displayName = data.displayName || null;
                existingAsset.description = data.description || "";
                existingAsset.link = data.link || null;
              }
            },
          },
          {
            eventType: "SchemaVariantUpdateFinished",
            callback: async (data) => {
              if (data.changeSetId !== changeSetId) return;
              this.LOAD_SCHEMA_VARIANT_LIST();
              moduleStore.SYNC();
            },
          },
          {
            eventType: "ModulesUpdated",
            callback: async (data) => {
              if (data.changeSetId !== changeSetId) return;
              this.LOAD_SCHEMA_VARIANT_LIST();
            },
          },
          {
            eventType: "SchemaVariantUpdated",
            callback: (variant, metadata) => {
              if (metadata.change_set_id !== changeSetId) return;
              const savedAssetIdx = this.variantList.findIndex((a) => a.schemaVariantId === variant.schemaVariantId);

              if (variant?.assetFuncId) funcStore.FETCH_CODE(variant.assetFuncId);

              if (savedAssetIdx !== -1) {
                this.variantList.splice(savedAssetIdx, 1, variant);
              } else this.variantList.push(variant);
            },
          },
          {
            eventType: "SchemaVariantReplaced",
            callback: ({ newSchemaVariant: newVariant, oldSchemaVariantId: oldVariantId }, metadata) => {
              if (metadata.change_set_id !== changeSetId) return;

              const oldAssetIdx = this.variantList.findIndex((a) => a.schemaVariantId === oldVariantId);
              if (oldAssetIdx !== -1) {
                this.variantList.splice(oldAssetIdx, 1, newVariant);
              } else this.variantList.push(newVariant);

              if (this.selectedSchemaVariants.includes(oldVariantId)) {
                if (this.selectedSchemaVariants.length === 1) {
                  const selectedFuncs = _.clone(this.selectedFuncs);
                  this.setSchemaVariantSelection(newVariant.schemaVariantId);
                  // Right now you can't multi select functions but they're in an array,
                  // so we just set the values in the array, which will leave the last one selected
                  for (const funcId of selectedFuncs) {
                    this.setFuncSelection(funcId);
                  }
                } else {
                  this.removeSchemaVariantSelection(oldVariantId);
                  this.addSchemaVariantSelection(newVariant.schemaVariantId);
                }
              }
            },
          },
          {
            eventType: "ModuleImported",
            callback: (schemaVariants, metadata) => {
              if (metadata.change_set_id !== changeSetId) return;

              for (const variant of schemaVariants) {
                this.uninstalledVariantList = this.uninstalledVariantList.filter(
                  (uninstalledVariant) => uninstalledVariant.schemaId !== variant.schemaId,
                );

                const savedAssetIdx = this.variantList.findIndex((a) => a.schemaId === variant.schemaId);
                if (savedAssetIdx !== -1) {
                  this.variantList.splice(savedAssetIdx, 1, variant);
                } else this.variantList.push(variant);

                if (metadata.actor !== "System" && metadata.actor.User === authStore.userPk) {
                  this.setSchemaVariantSelection(variant.schemaVariantId);
                }
              }
            },
          },
          {
            eventType: "ChangeSetApplied",
            callback: () => {
              this.LOAD_SCHEMA_VARIANT_LIST();
              moduleStore.SYNC();
            },
          },
          // For the async api endpoints
          {
            eventType: "AsyncError",
            callback: ({ id, error }) => {
              if (id === this.executeSchemaVariantTaskId) {
                this.executeSchemaVariantTaskRunning = false;
                this.executeSchemaVariantTaskId = undefined;

                let errorMessage = error;
                {
                  const match = error.match("function execution result failure:.*message=(.*?),")?.[1];

                  if (match) {
                    errorMessage = match;
                  }
                }
                {
                  const match = error.match("func execution failure error: (.*)")?.[1];

                  if (match) {
                    errorMessage = match;
                  }
                }
                this.executeSchemaVariantTaskError = errorMessage;
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
    }),
  )();
};
