<template>
  <div class="">
    <SiSearch />

    <p>
      Get Started by dragging the assets that you wish to include in your
      application into the canvas to the right
    </p>

    <ul>
      <SiCollapsible
        v-for="(category, index) in assetCategories"
        :key="index"
        as="li"
        content-as="ul"
        :label="category.name"
      >
        <li v-for="i in 5" :key="i">
          <SiNodeSprite :color="category.color" :name="category.name" />
        </li>
      </SiCollapsible>
    </ul>
  </div>
</template>

<script setup lang="ts">
import SiNodeSprite from "@/molecules/SiNodeSprite.vue";
import SiCollapsible from "@/organisims/SiCollapsible.vue";
import SiSearch from "@/molecules/SiSearch.vue";

const assetCategories = [
  {
    name: "Kubernetes",
    color: "#00F",
    assets: [],
  },
  {
    name: "Blip",
    color: "#F00",
    assets: [],
  },
  {
    name: "Something with a really really long name oh no oh no",
    color: "#0FF",
    assets: [],
  },
];
// NodeSprite
// NodeSprite normal hover focus disabled
// NodeCategory
</script>
