/* small helper to load SVG images and show them in konva - with the ability to
change the fill color. NOTE - this only works if the SVG has the fill set to
"currentColor" in it's source */

<template>
  <v-image ref="konvaImageRef" :config="konvaConfig" />
</template>

<script lang="ts" setup>
import { computed, nextTick, onMounted, ref, watchEffect } from "vue";
import _ from "lodash";
import TextWidgetVue from "../EditForm/TextWidget.vue";

const props = defineProps({
  rawSvg: { type: String },
  color: { type: String },
  config: { type: Object, required: true },
  spin: { type: Boolean },
});

const konvaImageRef = ref();

const imageEl = ref<HTMLImageElement | null>(null);

const konvaConfig = computed(() => {
  // we'll adjust offset and x/y position to make origin in the center of the icon
  // which will allow us to spin easily
  const offsetX = props.config.width / 2;
  const offsetY = props.config.height / 2;
  return {
    ...props.config,
    image: imageEl.value,
    offset: { x: offsetX, y: offsetY },
    x: props.config.x + offsetX,
    y: props.config.y + offsetY,
  };
});

watchEffect(() => {
  if (!props.rawSvg) {
    imageEl.value = null;
    return;
  }

  let svgString = props.rawSvg;

  // the way the raw icons are imported from unplugin-icons may not include the xmlns tag
  // and the way the we're loading them as images is a little picky, so this fixes the issue...
  if (!svgString.includes("xmlns=")) {
    svgString = svgString.replace(
      "<svg ",
      '<svg xmlns="http://www.w3.org/2000/svg" ',
    );
  }
  // replace color with fill
  if (props.color) {
    svgString = svgString.replace("currentColor", props.color);
  }
  const imgBase64Src = `data:image/svg+xml;base64, ${btoa(svgString)}`;

  // have to initialize and actual image DOM element and pass it into the konva element when it loads
  const img = new window.Image();
  img.src = imgBase64Src;
  img.onload = () => {
    imageEl.value = img;
  };
});

function initSpin() {
  const node = konvaImageRef.value.getNode();
  node.to({
    // for some reason, resetting rotation to 0 and then animating to 360 does not work
    // although 360 -> 0 repeatedly works... and I could not recreate the problem in a jsbin using konva directly to report the issue...
    // we can however can keep rotating to multiples of 360, so not worth deeper investigation
    rotation: node.rotation() + 360,
    duration: 1,
    // note the arrow fn here - without it, initSpin gets called in a different context and doesn't work
    onFinish: () => initSpin(),
  });
}

onMounted(() => {
  // calling without nextTick shows an error in the console that the node is not added to the canvas yet
  // although it does still work without any issues
  if (props.spin) nextTick(initSpin);
});
</script>
