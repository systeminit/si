import { defineStore } from "pinia";
import * as _ from "lodash-es";
import { ApiRequest, addStoreHooks } from "@si/vue-lib/pinia";
import { Visibility } from "@/api/sdf/dal/visibility";
import { nilId } from "@/utils/nilId";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";

export type AssetId = string;
export type AssetSlug = string;

export interface ListVariantDefsResponse {
  variantDefs: ListedVariantDef[];
}

export interface ListedVariantDef {
  id: AssetId;
  name: string;
  menuName?: string;
  category: string;
  componentType: string;
  color: string;
  description: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface VariantDef extends ListedVariantDef {
  link?: string;
  definition: string;
  variantExists: boolean;
}

export type Asset = VariantDef;
export type AssetListEntry = ListedVariantDef;
export type AssetSaveRequest = Visibility &
  Omit<Asset, "createdAt" | "updatedAt" | "variantExists">;
export type AssetCreateRequest = Omit<
  AssetSaveRequest,
  "id" | "definition" | "variantExists"
>;
export type AssetCloneRequest = Visibility & { id: AssetId };

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
  return addStoreHooks(
    defineStore(`cs${changeSetId || "NONE"}/asset`, {
      state: () => ({
        assetList: [] as ListedVariantDef[],
        assetsById: {} as Record<AssetId, Asset>,
        selectedAssetId: null as AssetId | null,
      }),
      getters: {
        assets: (state) => _.values(state.assetsById),
        selectedAsset(): Asset | undefined {
          return this.assetsById[this.selectedAssetId || 0];
        },
      },
      actions: {
        setSelectedAssetId(selection: AssetId | null) {
          if (!selection) {
            this.selectedAssetId = null;
          } else {
            if (this.assetsById[selection]) {
              this.selectedAssetId = selection;
            }
          }
        },

        async SELECT_ASSET(selection: AssetId | null) {
          if (!selection) {
            this.setSelectedAssetId(selection);
            return;
          }

          if (!this.assetsById[selection]) {
            await this.LOAD_ASSET(selection);
          }

          this.setSelectedAssetId(selection);
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

        createNewAsset() {
          return {
            id: nilId(),
            name: `new asset ${Math.floor(Math.random() * 10000)}${
              Math.floor(Math.random() * 20) === 0
                ? " omg has such a long name the name is so long you can't even believe how long it is!"
                : ""
            }`,
            definition: "",
            color: this.generateMockColor(),
            description: "",
            category: "",
            componentType: "component",
            link: "https://www.systeminit.com/",
            createdAt: new Date(),
            updatedAt: new Date(),
            variantExists: false,
          };
        },

        async CREATE_ASSET(asset: Asset) {
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
                "variantExists",
                "createdAt",
                "updatedAt",
                "definition",
              ]),
            },
          });
        },

        async CLONE_ASSET(assetId: AssetId) {
          return new ApiRequest<
            { id: AssetId; success: boolean },
            AssetCloneRequest
          >({
            method: "post",
            url: "/variant_def/clone_variant_def",
            params: {
              ...visibility,
              id: assetId,
            },
          });
        },

        async SAVE_ASSET(asset: Asset) {
          this.assetsById[asset.id] = asset;
          return new ApiRequest<{ success: boolean }, AssetSaveRequest>({
            method: "post",
            url: "/variant_def/save_variant_def",
            params: {
              ...visibility,
              ..._.omit(asset, ["variantExists", "createdAt", "updatedAt"]),
            },
          });
        },

        async LOAD_ASSET(assetId: AssetId) {
          return new ApiRequest<Asset, Visibility & { id: AssetId }>({
            url: "/variant_def/get_variant_def",
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
          return new ApiRequest<{ id: string }, Visibility & { id: AssetId }>({
            method: "post",
            url: "/variant_def/exec_variant_def",
            params: { ...visibility, id: assetId },
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
