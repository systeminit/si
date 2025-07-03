<template>
  <section
    :class="clsx('h-full w-full', themeClasses('bg-white', 'bg-neutral-950'))"
  >
    <div
      :class="
        clsx(
          'main pt-xs flex flex-col gap-xs items-stretch [&>div]:mx-[12px]',
          themeClasses('bg-white', 'bg-neutral-950'),
        )
      "
    >
      <!-- view dropdown / search bar -->
      <div class="flex-none flex flex-row items-center gap-xs">
        <!-- View dropdown skeleton -->
        <div
          :class="
            clsx(
              'rounded min-w-[128px] h-8 skeleton-shimmer',
              themeClasses(
                'bg-neutral-200 border border-neutral-400',
                'bg-neutral-700 border border-neutral-600',
              ),
            )
          "
        ></div>

        <!-- search bar -->
        <div
          :class="
            clsx(
              'rounded grow h-8 skeleton-shimmer flex items-center px-2 gap-2',
              themeClasses(
                'bg-neutral-200 border border-neutral-400',
                'bg-neutral-700 border border-neutral-600',
              ),
            )
          "
        ></div>
      </div>

      <div class="flex-none flex flex-row items-center gap-xs justify-between">
        <!-- grid/map toggle -->
        <div class="flex flex-row gap-1">
          <div
            :class="
              clsx(
                'px-3 py-1 rounded h-8 skeleton-shimmer min-w-[128px]',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
          ></div>
        </div>

        <div class="flex flex-row gap-xs">
          <!-- group by dropdown skeleton -->
          <div
            :class="
              clsx(
                'rounded min-w-[100px] h-8 skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
          ></div>

          <!-- sort by dropdown skeleton -->
          <div
            :class="
              clsx(
                'rounded min-w-[100px] h-8 skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
          ></div>
        </div>
      </div>

      <div class="flex flex-col items-center mt-xl">
        <div class="flex flex-row items-center justify-center">
          <div
            :class="
              clsx(
                'rounded skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
            style="width: 250px; height: 75px"
          ></div>

          <div class="relative" style="width: 300px; height: 75px">
            <svg
              width="300"
              height="175"
              style="position: absolute; top: 0; left: 0"
            >
              <line
                x1="0"
                y1="37"
                x2="300"
                y2="37"
                :stroke="lineColor"
                stroke-width="2"
              />
            </svg>
          </div>

          <div
            :class="
              clsx(
                'rounded skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
            style="width: 250px; height: 75px"
          ></div>
        </div>
      </div>

      <div class="flex flex-col items-center mt-xl">
        <div class="flex flex-row items-center justify-center">
          <div
            :class="
              clsx(
                'rounded skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
            style="width: 250px; height: 75px"
          ></div>

          <div class="relative" style="width: 300px; height: 75px">
            <svg
              width="300"
              height="175"
              style="position: absolute; top: 0; left: 0"
            >
              <line
                x1="0"
                y1="37"
                x2="300"
                y2="37"
                :stroke="lineColor"
                stroke-width="2"
              />

              <line
                x1="150"
                y1="37"
                x2="150"
                y2="137"
                :stroke="lineColor"
                stroke-width="2"
              />

              <line
                x1="150"
                y1="137"
                x2="300"
                y2="137"
                :stroke="lineColor"
                stroke-width="2"
              />
            </svg>
          </div>

          <div
            :class="
              clsx(
                'rounded skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
            style="width: 250px; height: 75px"
          ></div>
        </div>

        <div
          class="flex flex-row items-center justify-center"
          style="margin-top: 25px"
        >
          <div
            :class="clsx('rounded', themeClasses('bg-white', 'bg-neutral-950'))"
            style="width: 250px; height: 75px"
          ></div>
          <div style="width: 300px; height: 75px"></div>
          <div
            :class="
              clsx(
                'rounded skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
            style="width: 250px; height: 75px"
          ></div>
        </div>
      </div>

      <div class="flex flex-col items-center mt-xl">
        <div class="flex flex-row items-center justify-center">
          <div
            :class="
              clsx(
                'rounded skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
            style="width: 250px; height: 75px"
          ></div>

          <div class="relative" style="width: 300px; height: 75px">
            <svg
              width="300"
              height="275"
              style="position: absolute; top: 0; left: 0"
            >
              <line
                x1="0"
                y1="37"
                x2="300"
                y2="37"
                :stroke="lineColor"
                stroke-width="2"
              />

              <line
                x1="150"
                y1="37"
                x2="150"
                y2="237"
                :stroke="lineColor"
                stroke-width="2"
              />

              <line
                x1="150"
                y1="137"
                x2="300"
                y2="137"
                :stroke="lineColor"
                stroke-width="2"
              />

              <line
                x1="150"
                y1="237"
                x2="300"
                y2="237"
                :stroke="lineColor"
                stroke-width="2"
              />
            </svg>
          </div>

          <div
            :class="
              clsx(
                'rounded skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
            style="width: 250px; height: 75px"
          ></div>
        </div>

        <div
          class="flex flex-row items-center justify-center"
          style="margin-top: 25px"
        >
          <div class="transparent" style="width: 250px; height: 75px"></div>
          <div style="width: 300px; height: 75px"></div>
          <div
            :class="
              clsx(
                'rounded skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
            style="width: 250px; height: 75px"
          ></div>
        </div>

        <div
          class="flex flex-row items-center justify-center"
          style="margin-top: 25px"
        >
          <div style="width: 550px; height: 75px"></div>
          <div
            :class="
              clsx(
                'rounded skeleton-shimmer',
                themeClasses(
                  'bg-neutral-200 border border-neutral-400',
                  'bg-neutral-700 border border-neutral-600',
                ),
              )
            "
            style="width: 250px; height: 75px"
          ></div>
        </div>
      </div>
    </div>
  </section>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import clsx from "clsx";
import { themeClasses, useTheme } from "@si/vue-lib/design-system";

const { theme } = useTheme();

const lineColor = computed(() => {
  if (theme.value === "dark") {
    return "#404040";
  } else {
    return "#e5e5e5";
  }
});
</script>
