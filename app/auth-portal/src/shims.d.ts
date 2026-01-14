// adds TS info for .md files - see https://www.npmjs.com/package/vite-plugin-markdown
declare module "*.md" {
  // "unknown" would be more detailed depends on how you structure frontmatter
  const attributes: Record<string, unknown>;

  // When "Mode.TOC" is requested
  const toc: { level: string; content: string }[];

  // When "Mode.HTML" is requested
  const html: string;

  // When "Mode.Vue" is requested
  import { ComponentOptions, Component } from "vue";

  const VueComponent: ComponentOptions;
  const VueComponentWith: (
    components: Record<string, Component>,
  ) => ComponentOptions;

  // Modify below per your usage
  export { attributes, toc, html, VueComponent, VueComponentWith };
}
