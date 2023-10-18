/* small helper to load SVG images and show them in konva - with the ability to
change the fill color. NOTE - this only works if the SVG has the fill set to
"currentColor" in its source */

<template>
  <v-group>
    <v-image ref="konvaImageRef" :config="konvaConfig" />
  </v-group>
</template>

<script lang="ts" setup>
import {
  computed,
  nextTick,
  ref,
  watchEffect,
  watch,
  onBeforeUnmount,
} from "vue";
import Konva from "konva";

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
    x: (props.config.x || 0) + offsetX,
    y: (props.config.y || 0) + offsetY,
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
    svgString = svgString.replace(/currentColor/g, props.color);
  }
  const imgBase64Src = `data:image/svg+xml;base64, ${btoa(svgString)}`;

  // have to initialize an actual image DOM element and pass it into the konva element when it loads
  const img = new window.Image();
  img.src = imgBase64Src;
  img.onload = () => {
    imageEl.value = img;
  };
});

// handle rotation with a Konva tween
// NOTE - trying to be very mindful of memory leaks / cleaning up!
let spinTween: Konva.Tween | undefined;

function initSpin() {
  const node = konvaImageRef.value?.getNode();

  // on initial mount, the node may not be added to the canvas yet
  // it does still work, but we get an error in the console
  if (!node) {
    nextTick(initSpin);
    return;
  }

  if (!spinTween) {
    spinTween = new Konva.Tween({
      node,
      duration: 1,
      rotation: 360,
      onFinish: () => {
        spinTween?.reset();
        spinTween?.play();
      },
    });
  } else {
    spinTween.reset();
  }
  spinTween.play();
}

function stopSpin() {
  if (!spinTween) return;
  konvaImageRef.value?.getNode().rotation(0);
  spinTween.destroy();
  spinTween = undefined;
}

onBeforeUnmount(stopSpin);

watch(
  [() => props.spin],
  () => {
    if (props.spin) initSpin();
    else stopSpin();
  },
  { immediate: true },
);
</script>
