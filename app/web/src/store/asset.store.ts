import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";
import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import { Visibility } from "@/api/sdf/dal/visibility";
import { nilId } from "@/utils/nilId";
import keyedDebouncer from "@/utils/keyedDebouncer";
import router from "@/router";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import {
  FuncSummary,
  FuncId,
  useFuncStore,
  FuncWithDetails,
} from "./func/funcs.store";
import { useRouterStore } from "./router.store";

export type AssetId = string;
export type AssetSlug = string;

export interface ListVariantDefsResponse {
  variantDefs: ListedVariantDef[];
}

export interface InstalledPkgAssetView {
  assetId: string;
  assetHash: string;
  assetKind: string;
}

export type ComponentType =
  | "aggregationFrame"
  | "component"
  | "configurationFrame";

export interface ListedVariantDef {
  id: AssetId;
  name: string;
  menuName?: string;
  category: string;
  componentType: ComponentType;
  color: string;
  description: string;
  funcs: FuncSummary[];
  createdAt: IsoDateString;
  updatedAt: IsoDateString;
}

export interface VariantDef extends ListedVariantDef {
  link?: string;
  schemaVariantId?: string;
  code: string;
  handler: string;
  types?: string;
  hasComponents: boolean;
  hasAttrFuncs: boolean;
}

export type Asset = VariantDef;
export type AssetListEntry = ListedVariantDef;
export type AssetSaveRequest = Visibility &
  Omit<
    Asset,
    | "createdAt"
    | "updatedAt"
    | "variantExists"
    | "hasComponents"
    | "hasAttrFuncs"
  >;
export type AssetCreateRequest = Omit<
  AssetSaveRequest,
  "id" | "definition" | "variantExists"
>;
export type AssetCloneRequest = Visibility & { id: AssetId };

const LOCAL_STORAGE_LAST_SELECTED_ASSET_ID_KEY = "si-open-asset-id";

export const assetDisplayName = (asset: Asset | AssetListEntry) =>
  (asset.menuName ?? "").length === 0 ? asset.name : asset.menuName;

export const useAssetStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const visibility = {
    // changeSetId should not be empty if we are actually using this store
    // so we can give it a bad value and let it throw an error
    visibility_change_set_pk: changeSetId || "XXX",
  };

  const funcsStore = useFuncStore();

  let assetSaveDebouncer: ReturnType<typeof keyedDebouncer> | undefined;

  return addStoreHooks(
    defineStore(`cs${changeSetId || "NONE"}/asset`, {
      state: () => ({
        assetList: [] as AssetListEntry[],
        assetsById: {} as Record<AssetId, Asset>,
        openAssetFuncIds: {} as { [key: AssetId]: FuncId[] },
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
      },
      actions: {
        setSchemaVariantIdForAsset(assetId: AssetId, schemaVariantId: string) {
          const asset = this.assetsById[assetId];
          if (asset) {
            asset.schemaVariantId = schemaVariantId;
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
          const assets = {} as Record<AssetId, Asset>;

          return assets;
        },

        createNewAsset(): Asset {
          return {
            id: nilId(),
            name: `new asset ${Math.floor(Math.random() * 10000)}`,
            code: "",
            handler: "",
            color: this.generateMockColor(),
            description: "",
            category: "",
            componentType: "component",
            link: "https://www.systeminit.com/",
            funcs: [],
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            schemaVariantId: undefined,
            hasComponents: false,
            hasAttrFuncs: false,
          };
        },

        async CREATE_ASSET(asset: Asset) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetId === nilId()) changeSetsStore.creatingChangeSet = true;

          return new ApiRequest<
            { id: AssetId; success: boolean },
            AssetCreateRequest
          >({
            method: "post",
            url: "/variant_def/create_variant_def",
            params: {
              ...visibility,
              ..._.omit(asset, [
                "id",
                "schemaVariantId",
                "hasComponents",
                "hasAttrFuncs",
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
            url: "/variant_def/clone_variant_def",
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
            assetSaveDebouncer = keyedDebouncer(() => {
              this.SAVE_ASSET(asset);
            }, 2000);
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
            url: "/variant_def/save_variant_def",
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
              ..._.omit(asset, [
                "schemaVariantId",
                "hasComponents",
                "hasAttrFuncs",
                "createdAt",
                "updatedAt",
              ]),
            },
          });
        },

        async LOAD_ASSET(assetId: AssetId) {
          return new ApiRequest<Asset, Visibility & { id: AssetId }>({
            url: "/variant_def/get_variant_def",
            keyRequestStatusBy: assetId,
            params: { id: assetId, ...visibility },
            onSuccess: (response) => {
              this.assetsById[response.id] = response;
            },
          });
        },

        async LOAD_ASSET_LIST() {
          return new ApiRequest<ListVariantDefsResponse, Visibility>({
            url: "/variant_def/list_variant_defs",
            params: { ...visibility },
            onSuccess: (response) => {
              this.assetList = response.variantDefs;
            },
          });
        },

        async EXEC_ASSET(assetId: AssetId) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          const asset = this.assetsById[assetId];
          return new ApiRequest<
            { success: true; schemaVariantId: string },
            AssetSaveRequest
          >({
            method: "post",
            url: "/variant_def/exec_variant_def",
            keyRequestStatusBy: assetId,
            params: {
              ...visibility,
              ..._.omit(asset, [
                "schemaVariantId",
                "hasComponents",
                "hasAttrFuncs",
                "createdAt",
                "updatedAt",
              ]),
            },
          });
        },
      },
      onActivated() {
        this.LOAD_ASSET_LIST();
        const realtimeStore = useRealtimeStore();
        realtimeStore.subscribe(this.$id, `changeset/${changeSetId}`, [
          {
            eventType: "ChangeSetWritten",
            callback: (writtenChangeSetId) => {
              if (writtenChangeSetId !== changeSetId) return;
              this.LOAD_ASSET_LIST();
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
