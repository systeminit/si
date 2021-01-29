<template>
  <div class="flex flex-col justify-center h-screen bg-black">
    <div class="text-6xl text-center text-gray-300">
      <div>The System Initiative</div>
      <div class="flex lds-grid">
        <div></div>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
      </div>
      <div class="text-lg text-center text-orange-300">... {{ quote }} ...</div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";

import _ from "lodash";
import { RootStore } from "../store";

interface Data {
  quotes: string[];
  quote: string;
  interval: number | undefined;
}

export default Vue.extend({
  name: "LoadingPage",
  data() {
    return {
      quotes: [
        "is pulling your strings",
        "is checking the batteries",
        "is flying the plane",
        "is eating a delicious healthy breakfast",
        "is dancing like no-one is watching",
        "is making friends on the internet",
        "is listening to music",
        "is thinking about birds",
        "is wondering where the time goes",
        "is here now, to entertain you",
        "is in the jungle baby",
      ],
      quote: "is getting ready",
      interval: null,
    };
  },
  methods: {
    getQuote(): void {
      let quote = _.sample(this.quotes);
      if (!quote) {
        quote = "is unsure about itself";
      }
      this.quote = quote;
    },
  },
  computed: {
    ...mapState({
      loaded: (state: any) => state.loader.loading,
    }),
  },
  async mounted(): Promise<void> {
    // @ts-ignore
    const handle = setInterval(() => {
      this.getQuote();
    }, 2000);
    // @ts-ignore
    this.interval = handle;
    this.$store.dispatch("loader/load");
  },
  beforeDestroy(): void {
    if (this.interval != undefined) {
      // @ts-ignore
      clearInterval(this.interval);
    }
  },
  updated(): void {
    if (this.$store.state.loader.loaded) {
      if (this.$store.state.loader.nextUp) {
        const nextUp = this.$store.state.loader.nextUp;
        this.$store.commit("loader/nextUp", null);
        this.$router.push(nextUp);
      } else {
        this.$router.push({ name: "home" });
      }
    }
  },
});
</script>

<style scoped>
.lds-grid {
  display: inline-block;
  position: relative;
  width: 80px;
  height: 80px;
}
.lds-grid div {
  position: absolute;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  /* background: #fff; */
  animation: lds-grid 1.2s linear infinite;
  @apply bg-blue-300;
}
.lds-grid div:nth-child(1) {
  top: 8px;
  left: 8px;
  animation-delay: 0s;
}
.lds-grid div:nth-child(2) {
  top: 8px;
  left: 32px;
  animation-delay: -0.4s;
}
.lds-grid div:nth-child(3) {
  top: 8px;
  left: 56px;
  animation-delay: -0.8s;
}
.lds-grid div:nth-child(4) {
  top: 32px;
  left: 8px;
  animation-delay: -0.4s;
}
.lds-grid div:nth-child(5) {
  top: 32px;
  left: 32px;
  animation-delay: -0.8s;
}
.lds-grid div:nth-child(6) {
  top: 32px;
  left: 56px;
  animation-delay: -1.2s;
}
.lds-grid div:nth-child(7) {
  top: 56px;
  left: 8px;
  animation-delay: -0.8s;
}
.lds-grid div:nth-child(8) {
  top: 56px;
  left: 32px;
  animation-delay: -1.2s;
}
.lds-grid div:nth-child(9) {
  top: 56px;
  left: 56px;
  animation-delay: -1.6s;
}
@keyframes lds-grid {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}
</style>
