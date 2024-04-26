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
import { ComponentType } from "@/components/ModelingDiagram/diagram_types";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import {
  FuncId,
  FuncSummary,
  FuncWithDetails,
  useFuncStore,
} from "./func/funcs.store";
import { useRouterStore } from "./router.store";

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
export type AssetListEntry = ListedVariant;
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

        detachmentWarnings: [] as {
          message: string;
          funcId: FuncId;
          kind?: FuncKind;
        }[],
      }),
      getters: {
        assets: (state) => _.values(state.assetsById),
        selectedAsset(): Asset | undefined {
          return this.assetsById[this.urlSelectedAssetId ?? ""];
        },
        urlSelectedAssetId(): AssetId | undefined {
          const route = useRouterStore().currentRoute;
          const id = route?.params?.assetId as string;
          if (id) {
            storage.setItem(LOCAL_STORAGE_LAST_SELECTED_ASSET_ID_KEY, id);
          }
          return id as AssetId | undefined;
        },
        selectedAssetId(): AssetId | undefined {
          return this.selectedAsset?.id;
        },
        selectedFunc(): FuncWithDetails | undefined {
          return funcsStore.funcDetailsById[this.urlSelectedFuncId ?? ""];
        },
        urlSelectedFuncId(): FuncId | undefined {
          const route = useRouterStore().currentRoute;
          return route?.params?.funcId as FuncId | undefined;
        },
        selectedFuncId(): FuncId | undefined {
          return this.selectedFunc?.id;
        },
        assetListEntryById: (state) => (assetId: AssetId) =>
          state.assetList.find((asset) => asset.id === assetId),
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
        setSchemaVariantIdForAsset(assetId: AssetId, schemaVariantId: string) {
          const asset = this.assetsById[assetId];
          if (asset) {
            asset.id = schemaVariantId;
            this.assetsById[assetId] = asset;
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
          this.selectAsset(assetId, (this.openAssetFuncIds[assetId] ?? [])[0]);
        },

        async selectAsset(assetId: AssetId | undefined, funcId?: FuncId) {
          if (assetId === undefined) funcId = undefined;

          const route = router.currentRoute.value;
          await router.push({
            name: route.name ?? undefined,
            params: {
              ...route.params,
              assetId,
              funcId,
            },
          });
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

        generateMockAssets() {
          return {} as Record<AssetId, Asset>;
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
            }, 500);
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
              this.assetList = response.variants;
            },
          });
        },

        async EXEC_ASSET(assetId: AssetId) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          this.executeAssetTaskRunning = true;
          this.executeAssetTaskError = undefined;
          this.executeAssetTaskId = undefined;
          this.detachmentWarnings = [];

          const asset = this.assetsById[assetId];
          return new ApiRequest<
            {
              taskId: string;
            },
            AssetSaveRequest
          >({
            method: "post",
            url: "/variant/exec_variant",
            keyRequestStatusBy: assetId,
            params: {
              ...visibility,
              ..._.omit(asset, ["hasComponents", "createdAt", "updatedAt"]),
            },
            onSuccess: (response) => {
              this.executeAssetTaskId = response.taskId;
            },
          });
        },
      },
      onActivated() {
        this.LOAD_ASSET_LIST();
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
          {
            eventType: "SchemaVariantFinished",
            callback: async ({
              taskId,
              schemaVariantId,
              detachedAttributePrototypes,
            }) => {
              if (taskId === this.executeAssetTaskId) {
                this.executeAssetTaskRunning = false;
                this.executeAssetTaskError = undefined;

                for (const detached of detachedAttributePrototypes) {
                  if (
                    detached.context.type === "OutputSocketSocket" ||
                    detached.context.type === "InputSocketSocket"
                  ) {
                    this.detachmentWarnings.push({
                      funcId: detached.funcId,
                      kind: detached.kind ?? undefined,
                      message: `Attribute ${detached.funcName} detached from asset because the property associated to it changed. Socket=${detached.context.data.name} of Kind=${detached.context.data.kind}`,
                    });
                  } else if (
                    detached.context.type === "InputSocketProp" ||
                    detached.context.type === "Prop"
                  ) {
                    this.detachmentWarnings.push({
                      funcId: detached.funcId,
                      kind: detached.kind ?? undefined,
                      message: `Attribute ${detached.funcName} detached from asset because the property associated to it changed. Path=${detached.context.data.path} of Kind=${detached.context.data.kind}`,
                    });
                  }
                }

                if (schemaVariantId !== nilId() && this.selectedAssetId) {
                  this.setSchemaVariantIdForAsset(
                    this.selectedAssetId,
                    schemaVariantId,
                  );
                  // We need to reload both schemas and assets since they're stored separately
                  await this.LOAD_ASSET(this.selectedAssetId);
                  await useComponentsStore().FETCH_AVAILABLE_SCHEMAS();
                  await useFuncStore().FETCH_INPUT_SOURCE_LIST(schemaVariantId); // a new asset means new input sources
                }
              }
            },
          },
        ]);
        return () => {
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
