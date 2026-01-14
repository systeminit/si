<template>
  <div class="flex flex-col h-full">
    <!-- Name Header -->
    <div
      :class="
        clsx(
          'flex flex-row items-center px-2 py-3 border-b',
          themeClasses('bg-white border-neutral-300', 'bg-neutral-800 border-neutral-600'),
        )
      "
    >
      <ShimmeringTextLine class="h-[19px] w-64" />
    </div>

    <!-- Body Panels -->
    <section :class="clsx('grid flex-row grow px-sm pb-[15px] gap-sm')">
      <div class="attrs">
        <div
          :class="
            clsx(
              'attrs border flex flex-col',
              themeClasses('border-neutral-400 bg-white', 'border-neutral-600 bg-neutral-800'),
            )
          "
        >
          <div
            :class="
              clsx('border-b p-xs', themeClasses('border-neutral-400 bg-white', 'border-neutral-600 bg-neutral-800'))
            "
          >
            <ShimmeringTextLine class="h-[23px] w-full" style="max-width: 240px" />
          </div>

          <div class="flex flex-col gap-xs p-xs">
            <div class="flex flex-row items-center justify-between">
              <ShimmeringTextLine class="h-[22px] grow" style="max-width: 180px" />
              <ShimmeringTextLine class="h-9 w-1/2" />
            </div>
            <ShimmeringTextLine class="h-9 w-full" />
          </div>
          <div
            :class="
              clsx('border m-xs', themeClasses('border-neutral-400 bg-white', 'border-neutral-600 bg-neutral-800'))
            "
          >
            <div
              :class="
                clsx('border-b p-xs', themeClasses('border-neutral-400 bg-white', 'border-neutral-600 bg-neutral-800'))
              "
            >
              <ShimmeringTextLine class="h-[23px] w-full" style="max-width: 240px" />
            </div>

            <div class="flex flex-col gap-3xs p-xs">
              <div
                v-for="(size, i) in [140, 160, 180, 200, 180]"
                :key="i"
                class="flex flex-row items-center justify-between"
              >
                <ShimmeringTextLine class="h-[22px] grow" :style="`max-width: ${size}px`" />
                <ShimmeringTextLine class="h-8 w-1/2" />
              </div>
            </div>
          </div>
          <div
            :class="
              clsx('border m-xs h-52', themeClasses('border-neutral-400 bg-white', 'border-neutral-600 bg-neutral-800'))
            "
          >
            <div
              :class="
                clsx('border-b p-xs', themeClasses('border-neutral-400 bg-white', 'border-neutral-600 bg-neutral-800'))
              "
            >
              <ShimmeringTextLine class="h-[23px] w-full" style="max-width: 240px" />
            </div>
          </div>
        </div>
      </div>
      <div class="docs">
        <div
          :class="
            clsx(
              'flex flex-col gap-sm h-64 border p-sm',
              themeClasses('border-neutral-400 children:bg-white bg-white', 'border-neutral-600 bg-neutral-800'),
            )
          "
        >
          <ShimmeringTextLine class="h-8 w-full" />
        </div>
      </div>
      <div class="right">
        <div
          :class="
            clsx(
              'flex flex-col gap-sm h-64 border p-sm',
              themeClasses('border-neutral-400 children:bg-white bg-white', 'border-neutral-600 bg-neutral-800'),
            )
          "
        >
          <ShimmeringTextLine class="h-8 w-full" />
          <ShimmeringTextLine class="h-8 w-full" />
          <ShimmeringTextLine class="h-8 w-full" />
        </div>
      </div>
    </section>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import ShimmeringTextLine from "@/newhotness/layout_components/ShimmeringTextLine.vue";
</script>

<style lang="less" scoped>
section.grid {
  grid-template-areas:
    "name name name"
    "attrs docs right";
  grid-template-rows: 0 minmax(0, 1fr);
  grid-template-columns: minmax(0, 1fr) minmax(0, 25%) minmax(0, 25%);
}
.docs {
  grid-area: docs;
}
.right {
  grid-area: right;
}
.attrs {
  grid-area: attrs;
}
</style>
