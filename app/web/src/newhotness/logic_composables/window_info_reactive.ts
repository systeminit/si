import { ref } from "vue";

export const windowWidthReactive = ref<number>(window.innerWidth);
export const windowHeightReactive = ref<number>(window.innerHeight);

const onResize = () => {
  windowWidthReactive.value = window.innerWidth;
  windowHeightReactive.value = window.innerHeight;
};

window.addEventListener("resize", onResize);
