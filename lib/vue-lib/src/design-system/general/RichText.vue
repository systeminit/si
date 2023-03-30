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

  line-height: 1.4em;

  > * {
    margin-bottom: 1em;
    &:last-child {
      margin-bottom: 0;
    }
  }

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

  blockquote {
    padding-left: 2em;
  }

  > h1,
  > h2,
  > h3 {
    font-weight: bold;
    padding-top: 0.8em;
    &:first-child {
      padding-top: 0;
    }
  }
  > h1 {
    font-size: 28px;
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
    padding-left: 1em;
    li {
      list-style-type: disc;

      // hide list bullet if icon
      &:has(> .icon:first-child) {
        list-style: none;
        margin-left: -1em;
      }
      > .icon:first-child {
        display: inline-block;
        vertical-align: bottom;
        margin-right: 0.5em;
      }
      li {
        list-style-type: circle;
        li {
          list-style-type: square;
        }
      }
      // display: flex;
      // align-items: center;
      padding-bottom: 0.3em;
    }
    ul {
      padding-left: 1em;
      padding-top: 0.5em;
    }
  }
  > img,
  p > img {
    width: 100%;
    max-width: 700px;
    margin: 0 auto;
  }

  > table {
    width: 100%;

    th {
      background: rgba(0, 0, 0, 0.2);
    }

    tr,
    td,
    th {
      text-align: left;
      border: 1px solid white;
      html.light & {
        border-color: black;
      }
      padding: 0.5em;
      vertical-align: top;
    }
  }
}
</style>
