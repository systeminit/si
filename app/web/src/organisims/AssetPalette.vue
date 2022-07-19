<template>
  <div class="p-2">
    <SiSearch />

    <p>
      Get Started by dragging the assets that you wish to include in your
      application into the canvas to the right
    </p>

    <ul>
      <SiCollapsibleList
        v-for="(category, index) in assetCategories"
        :key="index"
        as="li"
        :label="category.name"
      >
        <SiListItem v-for="i in 5" :key="i" />
      </SiCollapsibleList>
    </ul>
  </div>
</template>

<script setup lang="ts">
import SiListItem from "@/molecules/SiAssetListItem.vue";
import SiCollapsibleList from "@/atoms/SiCollapsibleList.vue";
import SiSearch from "@/molecules/SiSearch.vue";

const assetCategories = [
  {
    name: "Kubernetes",
    color: "text-red",
    assets: [],
  },
  {
    name: "Blip",
    color: "text-blue",
    assets: [],
  },
  {
    name: "Something with a really really long name oh no oh no",
    color: "text-purple",
    assets: [],
  },
];
// NodeSprite
// NodeSprite normal hover focus disabled
// NodeCategory
</script>
