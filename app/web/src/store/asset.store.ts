import { defineStore } from "pinia";
import _ from "lodash";
import { addStoreHooks } from "@/store/lib/pinia_hooks_plugin";
import { ApiRequest } from "@/store/lib/pinia_api_tools";
import { Visibility } from "@/api/sdf/dal/visibility";
import { nilId } from "@/utils/nilId";
import { useChangeSetsStore } from "./change_sets.store";

export type AssetId = string;

export interface ListVariantDefsResponse {
  variantDefs: ListedVariantDef[];
}

export interface ListedVariantDef {
  id: AssetId;
  name: string;
  menuName?: string;
  category: string;
  color: string;
  description: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface VariantDef extends ListedVariantDef {
  link?: string;
  definition: string;
}

export type Asset = VariantDef;
export type AssetListEntry = ListedVariantDef;

export const assetDisplayName = (asset: Asset | AssetListEntry) =>
  asset.menuName ?? asset.name;

export const useAssetStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  const visibility = {
    visibility_change_set_pk: changeSetId ?? nilId(),
  };
  return addStoreHooks(
    defineStore(`cs${changeSetId || "NONE"}/asset`, {
      state: () => ({
        assetList: [] as ListedVariantDef[],
        assetsById: {} as Record<AssetId, Asset>,
        selectedAssetId: null as AssetId | null,
        lastAssignedId: null as number | null, // TODO - this won't be needed once we have the backend involved
      }),
      getters: {
        assets: (state) => _.values(state.assetsById),
        selectedAsset(): Asset {
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
          if (this.lastAssignedId) {
            this.lastAssignedId += 1;
          } else {
            this.lastAssignedId = 1;
          }

          const newAsset = {
            id: `${this.lastAssignedId}`,
            name: `new asset ${Math.floor(Math.random() * 10000)}${
              Math.floor(Math.random() * 20) === 0
                ? " omg has such a long name the name is so long you can't even believe how long it is!"
                : ""
            }`,
            definition: "",
            color: this.generateMockColor(),
            description: "",
            category: "",
            link: "https://www.systeminit.com/",
            createdAt: new Date(),
            updatedAt: new Date(),
          };

          this.assetsById[`${this.lastAssignedId}`] = newAsset;
          return newAsset;
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
      },
      onActivated() {
        this.LOAD_ASSET_LIST();
      },
    }),
  )();
};
