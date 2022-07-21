<template>
  <SiSearch />

  <p
    class="border-b-2 dark:border-[#525252] text-xs font-light leading-tight px-3 py-1"
  >
    Get Started by dragging the assets that you wish to include in your
    application into the canvas to the right
  </p>

  <ul class="overflow-y-auto">
    <SiCollapsible
      v-for="(category, category_index) in assetCategories"
      :key="category_index"
      as="li"
      content-as="ul"
      :label="category.name"
    >
      <li v-for="(node, node_index) in category.assets" :key="node_index">
        <SiNodeSprite :color="category.color" :name="node" />
      </li>
    </SiCollapsible>
  </ul>
</template>

<script setup lang="ts">
import SiNodeSprite from "@/molecules/SiNodeSprite.vue";
import SiCollapsible from "@/organisims/SiCollapsible.vue";
import SiSearch from "@/molecules/SiSearch.vue";

const assetCategories = [
  {
    name: "Kubernetes",
    color: "#00F",
    assets: ["Deployment"],
  },
  {
    name: "Cloud",
    color: "#F00",
    assets: ["Secret", "Database"],
  },
  {
    name: "Something with a really really long name oh no oh no",
    color: "#0FF",
    assets: [
      "Oh no this thing with a really really long name has an asset with a really long name inside",
      "Oh no this thing with a really really long name has an asset with a really really  long name inside",
      "Help me I'm stuck in a cloud component factory",
    ],
  },
];
// NodeSprite
// NodeSprite normal hover focus disabled
// NodeCategory
</script>
