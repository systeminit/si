import { defineStore } from "pinia";
import _ from "lodash";
import { addStoreHooks } from "@/store/lib/pinia_hooks_plugin";
import { ApiRequest } from "@/store/lib/pinia_api_tools";
import { useChangeSetsStore } from "./change_sets.store";

export type AssetId = string;

export type Asset = {
  id: AssetId;
  displayName: string;
  slug: string;
  code: string;
  color: string;
  version: string;
  createdAt: Date;
  createdBy: string;
  description: string;
  category: string;
  documentationUrl: string;
};

export const useAssetStore = () => {
  const changeSetsStore = useChangeSetsStore();
  const changeSetId = changeSetsStore.selectedChangeSetId;
  return addStoreHooks(
    defineStore(`cs${changeSetId || "NONE"}/asset`, {
      state: () => ({
        assetsById: {} as Record<AssetId, Asset>,
        selectedAssetId: null as AssetId | null,
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
        setSelectedAssetBySlug(selection: string | null) {
          if (!selection) {
            this.selectedAssetId = null;
          } else {
            const pkg = _.find(this.assets, (p) => p.slug === selection);
            if (pkg) {
              this.selectedAssetId = pkg.id;
            }
          }
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
          const amount = 5 + Math.floor(Math.random() * 20);

          for (let i = 0; i < amount; i++) {
            assets[i] = {
              id: `${i}`,
              displayName: `test asset ${Math.floor(Math.random() * 10000)}${
                Math.floor(Math.random() * 20) === 0
                  ? " omg has such a long name the name is so long you can't even believe how long it is!"
                  : ""
              }`,
              slug: `test${i}`,
              code: "here is where the code goes! this is just a mock\n\n\n\n\nYeah nothing to see here\n\n\n\n\n\n\n\n\n\n\n\n\njust testing the display of the CodeViewer for this spot\n\n\n\n\n\n\n\nCool.",
              color: this.generateMockColor(),
              version: "13.12",
              createdAt: new Date(
                new Date().getTime() - Math.random() * 10000000000,
              ),
              createdBy: "Fake McMock",
              description:
                "this is where the description will go, currently this is just mock data",
              category: "mock AWS",
              documentationUrl: "https://www.systeminit.com/",
            };
          }

          return assets;
        },

        async LOAD_ASSETS() {
          return new ApiRequest({
            url: "/session/restore_authentication", // TODO - replace with real API request
            onSuccess: () => {
              if (!this.assetsById[0]) {
                // only generate mock assets if we haven't done so yet
                this.assetsById = this.generateMockAssets();
              }
            },
          });
        },
      },
      onActivated() {
        this.LOAD_ASSETS();
      },
    }),
  )();
};
