import { Tiles } from "@si/vue-lib/design-system";
import { ComponentOptions } from "vue";
import { TosVersion } from "@si/ts-lib/src/terms-of-service";

type DocContents = Record<
  string,
  {
    title: string;
    slug: string;
    fileName: string;
    component: ComponentOptions;
  }
>;

export const LEGAL_DOCS_CONTENT = {} as Record<TosVersion, DocContents>;

const docImports = import.meta.glob(`@/content/legal/**/*.md`, {
  eager: true,
});
for (const fileName in docImports) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const importedDoc = docImports[fileName] as any;
  const { v: version, s: slug } =
    fileName.match(/.*\/(?<v>.*)\/(?<s>.*)\.md/)?.groups ?? {};

  const title = importedDoc.attributes?.title;

  if (!version || !slug || !title) {
    // eslint-disable-next-line no-console
    console.error(`Error loading doc file ${fileName}`);
    continue;
  }

  if (LEGAL_DOCS_CONTENT[version as TosVersion] === undefined) {
    LEGAL_DOCS_CONTENT[version as TosVersion] = {};
  }

  LEGAL_DOCS_CONTENT[version as TosVersion][slug] = {
    title: importedDoc.attributes.title,
    slug,
    fileName,
    component: importedDoc.VueComponentWith({
      Tiles,
    }),
  };
}
