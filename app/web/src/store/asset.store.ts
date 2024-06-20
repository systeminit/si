import { watch } from "vue";
import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import { useWorkspacesStore } from "@/store/workspaces.store";
import { FuncKind } from "@/api/sdf/dal/func";
import { Visibility } from "@/api/sdf/dal/visibility";
import { nilId } from "@/utils/nilId";
import keyedDebouncer from "@/utils/keyedDebouncer";
import router from "@/router";
import { PropKind } from "@/api/sdf/dal/prop";
import { ComponentType } from "@/api/sdf/dal/diagram";
import { nonNullable } from "@/utils/typescriptLinter";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import {
  FuncId,
  FuncSummary,
  FuncWithDetails,
  useFuncStore,
} from "./func/funcs.store";
import handleStoreError from "./errors";
import { useComponentsStore } from "./components.store";

export type AssetId = string;

export interface ListVariantsResponse {
  variants: ListedVariant[];
}

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

export interface ListedVariant {
  id: AssetId;
  defaultSchemaVariantId: string;
  name: string;
  displayName?: string;
  category: string;
  componentType: ComponentType;
  color: string;
  description: string;
  funcs: FuncSummary[];
  createdAt: IsoDateString;
  updatedAt: IsoDateString;
}

export interface Variant extends ListedVariant {
  link?: string;
  code: string;
  types?: string;
  hasComponents: boolean;
}

export type Asset = Variant;
export type AssetListEntry = ListedVariant & {
  canUpdate: boolean;
  canContribute: boolean;
};
export type AssetSaveRequest = Visibility &
  Omit<Asset, "createdAt" | "updatedAt" | "variantExists" | "hasComponents">;
export type AssetCreateRequest = Omit<
  AssetSaveRequest,
  "id" | "definition" | "variantExists"
>;
export type AssetCloneRequest = Visibility & { id: AssetId };

const LOCAL_STORAGE_LAST_SELECTED_ASSET_ID_KEY = "si-open-asset-id";

export const assetDisplayName = (asset: Asset | AssetListEntry) =>
  (asset.displayName ?? "").length === 0 ? asset.name : asset.displayName;

export const useAssetStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const visibility = {
    // changeSetId should not be empty if we are actually using this store
    // so we can give it a bad value and let it throw an error
    visibility_change_set_pk: changeSetId || "XXX",
  };

  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  const funcsStore = useFuncStore();

  let assetSaveDebouncer: ReturnType<typeof keyedDebouncer> | undefined;

  return addStoreHooks(
    defineStore(`ws${workspaceId || "NONE"}/cs${changeSetId || "NONE"}/asset`, {
      state: () => ({
        assetList: [] as AssetListEntry[],
        assetsById: {} as Record<AssetId, Asset>,
        openAssetFuncIds: {} as { [key: AssetId]: FuncId[] },

        executeAssetTaskId: undefined as string | undefined,
        executeAssetTaskRunning: false as boolean,
        executeAssetTaskError: undefined as string | undefined,

        // represents state of the left rail lists and all open editor tabs
        selectedAssets: [] as AssetId[],
        selectedFuncs: [] as FuncId[],

        detachmentWarnings: [] as {
          message: string;
          funcId: FuncId;
          kind?: FuncKind;
        }[],
      }),
      getters: {
        assetFromListById: (state) => _.keyBy(state.assetList, (a) => a.id),
        assets: (state) => _.values(state.assetsById),
        selectedAssetId(state): AssetId | undefined {
          if (state.selectedAssets.length === 1) return state.selectedAssets[0];
          else return undefined;
        },
        selectedAsset(): Asset | undefined {
          if (this.selectedAssetId)
            return this.assetsById[this.selectedAssetId];
        },
        selectedAssetRecords(): AssetListEntry[] {
          return this.selectedAssets
            .map((id) => this.assetFromListById[id])
            .filter(nonNullable);
        },
        selectedFuncId(state): FuncId | undefined {
          if (state.selectedFuncs.length === 1) return state.selectedFuncs[0];
          else return undefined;
        },
        selectedFunc(): FuncWithDetails | undefined {
          if (this.selectedFuncId)
            return funcsStore.funcDetailsById[this.selectedFuncId];
          else return undefined;
        },
        assetBySchemaVariantId(): Record<string, Asset> {
          const assetsWithSchemaVariantId = _.filter(
            this.assets,
            (a) => a.id !== undefined,
          ) as (Variant & {
            schemaVariantId: string;
          })[];

          return _.keyBy(assetsWithSchemaVariantId, (a) => a.schemaVariantId);
        },
      },
      actions: {
        addAssetSelection(id: AssetId) {
          this.selectedAssets.push(id);
          this.LOAD_ASSET(id);
          this.syncSelectionIntoUrl();
          this.selectedFuncs = [];
        },
        setAssetSelection(id: AssetId) {
          if (!this.selectedAssets.includes(id)) {
            this.selectedFuncs = [];
          }
          this.selectedAssets = [id];
          this.LOAD_ASSET(id);
          this.syncSelectionIntoUrl();
          // no last selected func
          funcsStore.selectedFuncId = undefined;
        },
        async addFuncSelection(id: FuncId) {
          if (!this.selectedFuncs.includes(id)) this.selectedFuncs.push(id);
          await funcsStore.FETCH_FUNC(id);
          if (this.selectedAsset) this.openFunc(this.selectedAsset?.id, id);
          funcsStore.selectedFuncId = id;
          this.syncSelectionIntoUrl();
        },
        removeFuncSelection(id: FuncId) {
          const idx = this.selectedFuncs.indexOf(id);
          if (idx !== -1) this.selectedFuncs.splice(idx, 1);
        },
        syncSelectionIntoUrl(returnQuery?: boolean) {
          let selectedIds: string[] = [];
          selectedIds = _.map(this.selectedAssets, (id) => `a_${id}`);
          selectedIds = selectedIds.concat(
            _.map(this.selectedFuncs, (id) => `f_${id}`),
          );

          const newQueryObj = {
            ...(selectedIds.length && { s: selectedIds.join("|") }),
          };
          if (returnQuery) return newQueryObj;

          if (!_.isEqual(router.currentRoute.value.query, newQueryObj)) {
            router.replace({
              query: newQueryObj,
            });
          }
        },
        async syncUrlIntoSelection() {
          this.selectedAssets = [];
          this.selectedFuncs = [];
          funcsStore.selectedFuncId = undefined;
          const ids = ((router.currentRoute.value.query?.s as string) || "")
            .split("|")
            .filter(Boolean);
          if (ids.length > 0) {
            /* eslint-disable @typescript-eslint/no-explicit-any */
            const promises: Promise<any>[] = [];
            const fnIds = [] as FuncId[];
            ids.sort().forEach((id) => {
              if (id.startsWith("a_")) {
                id = id.substring(2);
                this.selectedAssets.push(id);
                promises.push(this.LOAD_ASSET(id));
              } else if (id.startsWith("f_")) {
                id = id.substring(2);
                this.selectedFuncs.push(id);
                promises.push(funcsStore.FETCH_FUNC(id));
                fnIds.push(id);
              }
            });
            await Promise.all(promises);
            for (const id of fnIds)
              if (this.selectedAssets[0])
                this.openFunc(this.selectedAssets[0], id);

            funcsStore.selectedFuncId = fnIds[fnIds.length - 1];
          }
        },

        getLastSelectedAssetId(): AssetId | undefined {
          return storage.getItem(
            LOCAL_STORAGE_LAST_SELECTED_ASSET_ID_KEY,
          ) as AssetId;
        },

        openFunc(assetId: AssetId, funcId: FuncId) {
          const funcs = this.openAssetFuncIds[assetId] ?? [];
          if (!funcs.includes(funcId)) {
            funcs.push(funcId);
          }

          this.openAssetFuncIds[assetId] = funcs;
        },

        closeFunc(assetId: AssetId, funcId: FuncId) {
          const funcs = this.openAssetFuncIds[assetId] ?? [];
          this.openAssetFuncIds[assetId] = funcs.filter(
            (fId) => fId !== funcId,
          );
          this.removeFuncSelection(funcId);
        },

        // MOCK DATA GENERATION
        generateMockColor() {
          return `#${_.sample([
            "FF0000",
            "FFFF00",
            "FF00FF",
            "00FFFF",
            "FFAA00",
            "AAFF00",
            "00FFAA",
            "00AAFF",
          ])}`;
        },

        createNewAsset(): Asset {
          return {
            id: nilId(),
            defaultSchemaVariantId: "",
            name: `new asset ${Math.floor(Math.random() * 10000)}`,
            code: "",
            color: this.generateMockColor(),
            description: "",
            category: "",
            componentType: ComponentType.Component,
            link: "https://www.systeminit.com/",
            funcs: [],
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            hasComponents: false,
          };
        },

        async CREATE_ASSET(asset: Asset) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetId === changeSetsStore.headChangeSetId)
            changeSetsStore.creatingChangeSet = true;
          return new ApiRequest<
            { id: AssetId; success: boolean },
            AssetCreateRequest
          >({
            method: "post",
            url: "/variant/create_variant",
            params: {
              ...visibility,
              ..._.omit(asset, [
                "id",
                "hasComponents",
                "createdAt",
                "updatedAt",
              ]),
            },
          });
        },

        async CLONE_ASSET(assetId: AssetId) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          return new ApiRequest<
            { id: AssetId; success: boolean },
            AssetCloneRequest
          >({
            method: "post",
            keyRequestStatusBy: assetId,
            url: "/variant/clone_variant",
            params: {
              ...visibility,
              id: assetId,
            },
          });
        },

        enqueueAssetSave(asset: Asset) {
          if (changeSetsStore.headSelected) return this.SAVE_ASSET(asset);

          this.assetsById[asset.id] = asset;

          if (!assetSaveDebouncer) {
            assetSaveDebouncer = keyedDebouncer((id: AssetId) => {
              const a = this.assetsById[id];
              if (!a) return;
              this.SAVE_ASSET(a);
            }, 1000);
          }
          const assetSaveFunc = assetSaveDebouncer(asset.id);
          if (assetSaveFunc) {
            assetSaveFunc(asset.id);
          }
        },

        async SAVE_ASSET(asset: Asset) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;
          const isHead = changeSetsStore.headSelected;

          return new ApiRequest<{ success: boolean }, AssetSaveRequest>({
            method: "post",
            keyRequestStatusBy: asset.id,
            url: "/variant/save_variant",
            optimistic: () => {
              if (isHead) return () => {};

              const current = this.assetsById[asset.id];
              this.assetsById[asset.id] = asset;
              return () => {
                if (current) {
                  this.assetsById[asset.id] = current;
                } else {
                  delete this.assetsById[asset.id];
                }
              };
            },
            params: {
              ...visibility,
              ..._.omit(asset, ["hasComponents", "createdAt", "updatedAt"]),
            },
          });
        },

        async LOAD_ASSET(assetId: AssetId) {
          return new ApiRequest<
            Asset,
            Visibility & {
              id: AssetId;
            }
          >({
            url: "/variant/get_variant",
            keyRequestStatusBy: assetId,
            params: {
              id: assetId,
              ...visibility,
            },
            onSuccess: (response) => {
              this.assetsById[response.id] = response;
            },
          });
        },

        async LOAD_ASSET_LIST() {
          return new ApiRequest<ListVariantsResponse, Visibility>({
            url: "/variant/list_variants",
            params: { ...visibility },
            onSuccess: (response) => {
              this.assetList = response.variants.map((v) => {
                const a = v as AssetListEntry;
                a.canContribute = false;
                a.canUpdate = false;
                return a;
              });
            },
          });
        },

        async EXEC_ASSET(assetId: AssetId) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          this.detachmentWarnings = [];
          const asset = this.assetsById[assetId];

          return new ApiRequest<null>({
            method: "post",
            url: "/variant/update_variant",
            keyRequestStatusBy: assetId,
            params: {
              ...visibility,
              ..._.omit(asset, ["hasComponents", "createdAt", "updatedAt"]),
            },
          });
        },
      },
      async onActivated() {
        await this.LOAD_ASSET_LIST();
        const stopWatchingUrl = watch(
          () => {
            return router.currentRoute.value.name;
          },
          () => {
            if (
              router.currentRoute.value.name === "workspace-lab-assets" &&
              Object.values(router.currentRoute.value.query).length > 0
            ) {
              this.syncUrlIntoSelection(); // handles PAGE LOAD
            }
          },
          {
            immediate: true,
          },
        );

        const realtimeStore = useRealtimeStore();
        realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
          {
            eventType: "SchemaVariantCreated",
            callback: (data) => {
              if (data.changeSetId !== changeSetId) return;
              this.LOAD_ASSET_LIST();
            },
          },
          {
            eventType: "SchemaVariantCloned",
            callback: (data) => {
              if (data.changeSetId !== changeSetId) return;
              this.LOAD_ASSET_LIST();
            },
          },
          {
            eventType: "SchemaVariantSaved",
            callback: (data) => {
              if (data.changeSetId !== changeSetId) return;
              this.LOAD_ASSET_LIST();
            },
          },
          {
            eventType: "SchemaVariantUpdateFinished",
            callback: async (data) => {
              if (data.changeSetId !== changeSetId) return;
              for (const asset of Object.values(this.assetsById)) {
                if (asset.defaultSchemaVariantId === data.oldSchemaVariantId) {
                  asset.defaultSchemaVariantId = data.newSchemaVariantId;
                }
              }
              this.LOAD_ASSET_LIST();

              if (this.selectedAssetId) {
                this.LOAD_ASSET(this.selectedAssetId);
                await useComponentsStore().FETCH_AVAILABLE_SCHEMAS();
              }

              await funcsStore.FETCH_INPUT_SOURCE_LIST(data.newSchemaVariantId);
            },
          },
          {
            eventType: "ChangeSetApplied",
            callback: () => {
              this.LOAD_ASSET_LIST();
            },
          },
          // For the async api endpoints
          {
            eventType: "AsyncError",
            callback: ({ id, error }) => {
              if (id === this.executeAssetTaskId) {
                this.executeAssetTaskRunning = false;
                this.executeAssetTaskId = undefined;

                let errorMessage = error;
                {
                  const match = error.match(
                    "function execution result failure:.*message=(.*?),",
                  )?.[1];

                  if (match) {
                    errorMessage = match;
                  }
                }
                {
                  const match = error.match(
                    "func execution failure error: (.*)",
                  )?.[1];

                  if (match) {
                    errorMessage = match;
                  }
                }
                this.executeAssetTaskError = errorMessage;
              }
            },
          },
        ]);

        const actionUnsub = this.$onAction(handleStoreError);
        return () => {
          stopWatchingUrl();
          actionUnsub();
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
