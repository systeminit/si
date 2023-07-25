import { defineStore } from "pinia";
// import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import { ApiRequest } from "@si/vue-lib/pinia";

export interface Asset {
  id: number;
  contentType: string;
  size: number;
  name: string;
  url: string;
}

interface Release {
  id: number;
  version: string;
  name: string;
  description: string;
  assets: Asset[];
  publishedAt: string;
}

export const useGithubStore = defineStore("github", {
  state: () => ({
    releases: [] as Release[],
  }),
  actions: {
    async LOAD_RELEASES() {
      return new ApiRequest<Release[]>({
        url: "/github/releases",
        onSuccess: (response) => {
          this.releases = response;
        },
      });
    },
  },
});
