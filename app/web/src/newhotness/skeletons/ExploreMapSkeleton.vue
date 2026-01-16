<template>
  <DelayedComponent>
    <section :class="clsx('h-full w-full', themeClasses('bg-white', 'bg-neutral-950'))">
      <div
        :class="
          clsx(
            'main pt-xs flex flex-col gap-xs items-stretch [&>div]:mx-[12px]',
            themeClasses('bg-white', 'bg-neutral-950'),
          )
        "
      >
        <div class="flex flex-col items-center mt-xl">
          <div class="flex flex-row items-center justify-center">
            <div
              :class="
                clsx(
                  'rounded skeleton-shimmer',
                  themeClasses('bg-neutral-200 border border-neutral-400', 'bg-neutral-700 border border-neutral-600'),
                )
              "
              style="width: 270px; height: 75px"
            ></div>

            <div class="relative" style="width: 300px; height: 75px">
              <svg width="300" height="175" style="position: absolute; top: 0; left: 0">
                <line x1="0" y1="37" x2="300" y2="37" :stroke="lineColor" stroke-width="2" />
              </svg>
            </div>

            <div
              :class="
                clsx(
                  'rounded skeleton-shimmer',
                  themeClasses('bg-neutral-200 border border-neutral-400', 'bg-neutral-700 border border-neutral-600'),
                )
              "
              style="width: 270px; height: 75px"
            ></div>
          </div>
        </div>

        <div class="flex flex-col items-center mt-xl">
          <div class="flex flex-row items-center justify-center">
            <div
              :class="
                clsx(
                  'rounded skeleton-shimmer',
                  themeClasses('bg-neutral-200 border border-neutral-400', 'bg-neutral-700 border border-neutral-600'),
                )
              "
              style="width: 270px; height: 75px"
            ></div>

            <div class="relative" style="width: 300px; height: 75px">
              <svg width="300" height="175" style="position: absolute; top: 0; left: 0">
                <line x1="0" y1="37" x2="300" y2="37" :stroke="lineColor" stroke-width="2" />

                <line x1="150" y1="37" x2="150" y2="137" :stroke="lineColor" stroke-width="2" />

                <line x1="150" y1="137" x2="300" y2="137" :stroke="lineColor" stroke-width="2" />
              </svg>
            </div>

            <div
              :class="
                clsx(
                  'rounded skeleton-shimmer',
                  themeClasses('bg-neutral-200 border border-neutral-400', 'bg-neutral-700 border border-neutral-600'),
                )
              "
              style="width: 270px; height: 75px"
            ></div>
          </div>

          <div class="flex flex-row items-center justify-center" style="margin-top: 25px">
            <div
              :class="clsx('rounded', themeClasses('bg-white', 'bg-neutral-950'))"
              style="width: 270px; height: 75px"
            ></div>
            <div style="width: 300px; height: 75px"></div>
            <div
              :class="
                clsx(
                  'rounded skeleton-shimmer',
                  themeClasses('bg-neutral-200 border border-neutral-400', 'bg-neutral-700 border border-neutral-600'),
                )
              "
              style="width: 270px; height: 75px"
            ></div>
          </div>
        </div>

        <div class="flex flex-col items-center mt-xl">
          <div class="flex flex-row items-center justify-center">
            <div
              :class="
                clsx(
                  'rounded skeleton-shimmer',
                  themeClasses('bg-neutral-200 border border-neutral-400', 'bg-neutral-700 border border-neutral-600'),
                )
              "
              style="width: 270px; height: 75px"
            ></div>

            <div class="relative" style="width: 300px; height: 75px">
              <svg width="300" height="275" style="position: absolute; top: 0; left: 0">
                <line x1="0" y1="37" x2="300" y2="37" :stroke="lineColor" stroke-width="2" />

                <line x1="150" y1="37" x2="150" y2="237" :stroke="lineColor" stroke-width="2" />

                <line x1="150" y1="137" x2="300" y2="137" :stroke="lineColor" stroke-width="2" />

                <line x1="150" y1="237" x2="300" y2="237" :stroke="lineColor" stroke-width="2" />
              </svg>
            </div>

            <div
              :class="
                clsx(
                  'rounded skeleton-shimmer',
                  themeClasses('bg-neutral-200 border border-neutral-400', 'bg-neutral-700 border border-neutral-600'),
                )
              "
              style="width: 270px; height: 75px"
            ></div>
          </div>

          <div class="flex flex-row items-center justify-center" style="margin-top: 25px">
            <div class="transparent" style="width: 270px; height: 75px"></div>
            <div style="width: 300px; height: 75px"></div>
            <div
              :class="
                clsx(
                  'rounded skeleton-shimmer',
                  themeClasses('bg-neutral-200 border border-neutral-400', 'bg-neutral-700 border border-neutral-600'),
                )
              "
              style="width: 270px; height: 75px"
            ></div>
          </div>

          <div class="flex flex-row items-center justify-center" style="margin-top: 25px">
            <div style="width: 550px; height: 75px"></div>
            <div
              :class="
                clsx(
                  'rounded skeleton-shimmer',
                  themeClasses('bg-neutral-200 border border-neutral-400', 'bg-neutral-700 border border-neutral-600'),
                )
              "
              style="width: 270px; height: 75px"
            ></div>
          </div>
        </div>
      </div>
    </section>
  </DelayedComponent>
</template>

<script lang="ts" setup>
import { computed, inject } from "vue";
import clsx from "clsx";
import { themeClasses, useTheme } from "@si/vue-lib/design-system";
import { assertIsDefined, ExploreContext } from "@/newhotness/types";
import DelayedComponent from "@/newhotness/layout_components/DelayedComponent.vue";

const explore = inject<ExploreContext>("EXPLORE_CONTEXT");
assertIsDefined<ExploreContext>(explore);

const { theme } = useTheme();

const lineColor = computed(() => {
  if (theme.value === "dark") {
    return "#404040";
  } else {
    return "#e5e5e5";
  }
});
</script>
