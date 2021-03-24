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
    id="svg214"
    sodipodi:docname="square-application.svg"
    inkscape:version="1.0.2 (e86c870879, 2021-01-15)"
  >
    <metadata id="metadata220">
      <rdf:RDF>
        <cc:Work rdf:about="">
          <dc:format>image/svg+xml</dc:format>
          <dc:type rdf:resource="http://purl.org/dc/dcmitype/StillImage" />
          <dc:title></dc:title>
        </cc:Work>
      </rdf:RDF>
    </metadata>
    <defs id="defs218" />
    <sodipodi:namedview
      pagecolor="#ffffff"
      bordercolor="#666666"
      borderopacity="1"
      objecttolerance="10"
      gridtolerance="10"
      guidetolerance="10"
      inkscape:pageopacity="0"
      inkscape:pageshadow="2"
      inkscape:window-width="1700"
      inkscape:window-height="2747"
      id="namedview216"
      showgrid="false"
      inkscape:zoom="30.625"
      inkscape:cx="12"
      inkscape:cy="12"
      inkscape:window-x="7283"
      inkscape:window-y="120"
      inkscape:window-maximized="0"
      inkscape:current-layer="svg214"
    />
    <rect x="3" y="3" width="18" height="18" rx="2" ry="2" id="rect212" />
    <text
      xml:space="preserve"
      style="font-size:18.6667px;line-height:1.25;font-family:sans-serif;letter-spacing:0px"
      x="5.2048788"
      y="18.802464"
      id="text224"
    >
      <tspan sodipodi:role="line" id="tspan222" x="5.2048788" y="18.802464">
        A
      </tspan>
    </text>
  </svg>
</template>

<script lang="ts">
import Vue from "vue";

export default Vue.extend({
  name: "SquareApplicationIcon",
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
