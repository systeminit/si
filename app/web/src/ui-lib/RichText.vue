<template>
  <div class="richtext">
    <slot />
  </div>
</template>

<style lang="less">
.richtext {
  a {
    color: @colors-action-300;
    text-decoration: underline;
    text-decoration-thickness: 0.05em;
    text-underline-offset: 0.15em;

    &:hover {
      color: @colors-action-400;
    }
  }
}
</style>
