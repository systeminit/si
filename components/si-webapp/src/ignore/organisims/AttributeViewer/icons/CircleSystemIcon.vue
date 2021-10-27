<template>
  <svg
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmlns:cc="http://creativecommons.org/ns#"
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:svg="http://www.w3.org/2000/svg"
    xmlns="http://www.w3.org/2000/svg"
    :width="width"
    :height="height"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    class="feather feather-circle"
    version="1.1"
    id="svg136"
  >
    <metadata id="metadata142">
      <rdf:RDF>
        <cc:Work rdf:about="">
          <dc:format>image/svg+xml</dc:format>
          <dc:type rdf:resource="http://purl.org/dc/dcmitype/StillImage" />
          <dc:title></dc:title>
        </cc:Work>
      </rdf:RDF>
    </metadata>
    <defs id="defs140" />
    <circle cx="12" cy="12" r="10" id="circle134" />
    <text
      xml:space="preserve"
      style="font-size:18.6667px;line-height:1.25;font-family:sans-serif"
      x="6.1632657"
      y="18.756596"
      id="text146"
    >
      <tspan id="tspan144" x="6.1632657" y="18.756596">S</tspan>
    </text>
  </svg>
</template>

<script lang="ts">
import Vue from "vue";

export default Vue.extend({
  name: "CircleSystemIcon",
  props: {
    size: {
      type: String,
      default: "24",
      validator: function validator(s: any) {
        return (
          !isNaN(s) ||
          (s.length >= 2 &&
            !isNaN(s.slice(0, s.length - 1)) &&
            s.slice(-1) === "x")
        );
      },
    },
  },
  computed: {
    sizeInPixels(): string {
      const size =
        this.size.slice(-1) === "x"
          ? this.size.slice(0, this.size.length - 1) + "em"
          : parseInt(this.size) + "px";
      return size;
    },
    width(): string {
      return this.sizeInPixels;
    },
    height(): string {
      return this.sizeInPixels;
    },
  },
});
</script>
