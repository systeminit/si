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
    class="feather feather-circle"
    version="1.1"
    id="svg58"
    sodipodi:docname="circle-application.svg"
    inkscape:version="1.0.2 (e86c870879, 2021-01-15)"
  >
    <metadata id="metadata64">
      <rdf:RDF>
        <cc:Work rdf:about="">
          <dc:format>image/svg+xml</dc:format>
          <dc:type rdf:resource="http://purl.org/dc/dcmitype/StillImage" />
        </cc:Work>
      </rdf:RDF>
    </metadata>
    <defs id="defs62" />
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
      id="namedview60"
      showgrid="false"
      inkscape:zoom="98.708333"
      inkscape:cx="11.602073"
      inkscape:cy="10.79567"
      inkscape:window-x="5563"
      inkscape:window-y="120"
      inkscape:window-maximized="0"
      inkscape:current-layer="svg58"
    />
    <circle cx="12" cy="12" r="10" id="circle56" />
    <text
      xml:space="preserve"
      style="font-style:normal;font-weight:normal;font-size:18.6667px;line-height:1.25;font-family:sans-serif;fill:#000000;fill-opacity:1;stroke:none"
      x="5.6472545"
      y="17.542866"
      id="text68"
    >
      <tspan sodipodi:role="line" id="tspan66" x="5.6472545" y="17.542866" />
    </text>
    <text
      xml:space="preserve"
      style="font-size:16px;line-height:1.25;font-family:sans-serif;letter-spacing:0px;stroke-width:2"
      x="6.4323931"
      y="17.034256"
      id="text228"
    >
      <tspan
        sodipodi:role="line"
        id="tspan226"
        x="6.4323931"
        y="17.034256"
        style="font-size:16px;stroke-width:2"
      >
        A
      </tspan>
    </text>
  </svg>
</template>

<script lang="ts">
import Vue from "vue";

export default Vue.extend({
  name: "CircleApplicationIcon",
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
