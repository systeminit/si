<template>
  <svg
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmlns:cc="http://creativecommons.org/ns#"
    xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    xmlns:svg="http://www.w3.org/2000/svg"
    xmlns="http://www.w3.org/2000/svg"
    xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"
    xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape"
    :width="width"
    :height="height"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    class="feather feather-square"
    version="1.1"
    id="svg337"
    sodipodi:docname="square-system.svg"
    inkscape:version="1.0.2 (e86c870879, 2021-01-15)"
  >
    <metadata id="metadata343">
      <rdf:RDF>
        <cc:Work rdf:about="">
          <dc:format>image/svg+xml</dc:format>
          <dc:type rdf:resource="http://purl.org/dc/dcmitype/StillImage" />
          <dc:title></dc:title>
        </cc:Work>
      </rdf:RDF>
    </metadata>
    <defs id="defs341" />
    <sodipodi:namedview
      pagecolor="#ffffff"
      bordercolor="#666666"
      borderopacity="1"
      objecttolerance="10"
      gridtolerance="10"
      guidetolerance="10"
      inkscape:pageopacity="0"
      inkscape:pageshadow="2"
      inkscape:window-width="2270"
      inkscape:window-height="2747"
      id="namedview339"
      showgrid="false"
      inkscape:zoom="30.625"
      inkscape:cx="12"
      inkscape:cy="12"
      inkscape:window-x="8430"
      inkscape:window-y="120"
      inkscape:window-maximized="0"
      inkscape:current-layer="svg337"
    />
    <rect x="3" y="3" width="18" height="18" rx="2" ry="2" id="rect335" />
    <text
      xml:space="preserve"
      style="font-size:17.3333px;line-height:1.25;font-family:sans-serif;letter-spacing:0px;"
      x="6.609056"
      y="18.300079"
      id="text347"
    >
      <tspan sodipodi:role="line" id="tspan345" x="6.609056" y="18.300079">
        S
      </tspan>
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
