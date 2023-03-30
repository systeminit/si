<template>
  <div class="richtext">
    <slot />
  </div>
</template>

<!-- having this helps with importing / IDE click through -->
<script lang="ts" setup></script>

<style lang="less">
.richtext {
  // this styling can be a bit problematic when nesting actual components inside
  // so likely will need some work if we do that more...
  // for now, I try to not apply styling if something is in an element with class "escape"

  :not(.escape) a:not(.vbutton) {
    html.dark & {
      color: @colors-action-300;
    }
    html.light & {
      color: @colors-action-500;
    }
    text-decoration: underline;
    text-decoration-thickness: 0.05em;
    text-underline-offset: 0.15em;

    &:hover {
      color: @colors-action-400;
    }
  }

  > .icon {
    display: inline-block;
    vertical-align: middle;
    margin-right: 0.4em;
  }
  > p {
    font-size: 16px;
    line-height: 1.4em;
    &:not(:last-child) {
      margin-bottom: 1em;
    }
  }

  > h1,
  > h2,
  > h3,
  > h4 {
    font-weight: bold;
    line-height: 1.4em;

    &:not(:last-child) {
      margin-bottom: 0.5em;
    }
  }
  > h1 {
    font-size: 30px;
  }
  > h2 {
    font-size: 24px;
  }
  > h3 {
    font-size: 18px;
  }
  > h4 {
    font-size: 16px;
  }

  > ul {
    margin-bottom: 1em;
    // doing this to fix icon alignment... we'll see if it holds true as we use this component in more places
    > li {
      display: flex;
      align-items: center;
      padding-bottom: 0.2em;
    }
  }
  > img,
  p > img {
    width: 100%;
    max-width: 700px;
    margin: 0 auto;
  }
}
</style>
