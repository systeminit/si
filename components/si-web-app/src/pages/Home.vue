<template>
  <div class="flex flex-col h-screen w-screen bg-black text-white vld-parent">
    <SiLoader :isLoading="isLoading" />
    <DebugRoute testId="location-display-homepage" />
    <div class="flex flex-row w-full h-full overflow-hidden">
      <div
        class="flex flex-col flex-no-wrap items-center justify-between flex-shrink-0 bg-primary w-54"
      >
        <Nav />
      </div>
      <div class="flex flex-col w-full h-full bg-gray-900">
        <SiError :message="errorMessage" />
        <router-view class="w-full h-full overflow-auto" />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import Nav from "@/organisims/Nav.vue";
import SiError from "@/atoms/SiError.vue";
import SiLoader from "@/atoms/SiLoader.vue";
import DebugRoute from "@/atoms/DebugRoute.vue";
import Bottle from "bottlejs";
import { SDF } from "@/api/sdf";
import { ISetDefaultsReply } from "@/store/modules/session";

interface IData {
  errorMessage: string;
  isLoading: boolean;
}

export default Vue.extend({
  name: "Home",
  components: {
    Nav,
    SiError,
    SiLoader,
    DebugRoute,
  },
  data(): IData {
    return {
      errorMessage: "",
      isLoading: true,
    };
  },
  async created() {
    const bottle = Bottle.pop("default");
    const sdf: SDF = bottle.container.SDF;
    // Start the websocket
    await sdf.startUpdate();

    // If this is the home page route, then we need to load
    // the users default organization and workspace.
    let response: ISetDefaultsReply = await this.$store.dispatch(
      "session/setDefaults",
    );
    this.isLoading = false;
    if (response.error) {
      this.errorMessage = response.error.message;
    } else {
      if (this.$route.name == "home") {
        this.$router.push({
          name: "workspace",
          params: {
            organizationId: response.organization.id,
            workspaceId: response.workspace.id,
          },
        });
      }
    }
  },
});
</script>
