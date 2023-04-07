import { Tiles } from "@si/vue-lib/design-system";
import { ComponentOptions } from "vue";

export const LEGAL_DOCS_CONTENT = {} as Record<
  string,
  {
    title: string;
    slug: string;
    fileName: string;
    component: ComponentOptions;
  }
>;
const docImports = import.meta.glob(`@/content/legal/2023-03-30/*.md`, {
  eager: true,
});
for (const fileName in docImports) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const importedDoc = docImports[fileName] as any;
  const slug = fileName.replace(/.md$/, "").replace(/.*\/\d+-/, "");
  LEGAL_DOCS_CONTENT[slug] = {
    title: importedDoc.attributes.title,
    slug,
    fileName,
    component: importedDoc.VueComponentWith({
      Tiles,
    }),
  };
}
