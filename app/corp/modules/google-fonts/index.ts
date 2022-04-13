import { resolve } from "path";
import { GoogleFontsHelper } from "google-fonts-helper";
import { defineNuxtModule } from "@nuxt/kit-edge";
import { CONFIG_KEY, ModuleOptions } from "../../@types/config.shim";

export default defineNuxtModule({
  meta: {
    name: "@nuxtjs/google-fonts",
    configKey: CONFIG_KEY,
    compatibility: {
      nuxt: "^3.0.0",
    },
  },
  async setup(moduleOptions, nuxt) {
    const DEFAULTS: ModuleOptions = {
      families: {},
      display: null,
      subsets: [],
      text: null,
      prefetch: true,
      preconnect: true,
      preload: true,
      useStylesheet: false,
      download: false,
      base64: false,
      inject: true,
      overwriting: false,
      outputDir: nuxt.options.dir.assets,
      stylePath: "css/fonts.css",
      fontsDir: "fonts",
      fontsPath: "~assets/fonts",
    };

    const options: ModuleOptions = {
      ...DEFAULTS,
      ...moduleOptions,
      ...nuxt.options["google-fonts"],
      ...nuxt.options[CONFIG_KEY],
    };

    const googleFontsHelper = new GoogleFontsHelper({
      families: options.families,
      display: options.display,
      subsets: options.subsets,
      text: options.text,
    });

    // merge fonts from valid head link
    // @ts-ignore
    const fontsParsed = (nuxt.options.meta.link || [])
      .filter((link) => GoogleFontsHelper.isValidURL(link.href))
      .map((link) => GoogleFontsHelper.parse(link.href));

    if (fontsParsed.length) {
      googleFontsHelper.merge(...fontsParsed);
    }

    // construct google fonts url
    const url = googleFontsHelper.constructURL();

    if (!url) {
      console.warn("No provided fonts.");

      return;
    }

    // remove fonts
    // @ts-ignore
    nuxt.options.meta.link = (nuxt.options.meta.link || []).filter(
      (link) => !GoogleFontsHelper.isValidURL(link.href),
    );

    // download
    if (options.download) {
      const outputDir =
        nuxt.options.alias[options.outputDir] || options.outputDir;

      try {
        await GoogleFontsHelper.download(url, {
          base64: options.base64,
          overwriting: options.overwriting,
          outputDir,
          stylePath: options.stylePath,
          fontsDir: options.fontsDir,
          fontsPath: options.fontsPath,
        });

        if (options.inject) {
          nuxt.options.css.push(resolve(outputDir, options.stylePath));
        }
      } catch (e) {
        /* istanbul ignore next */
        console.error(e);
      }

      return;
    }

    // https://developer.mozilla.org/en-US/docs/Web/Performance/dns-prefetch
    if (options.prefetch) {
      // @ts-ignore
      nuxt.options.meta.link.push({
        hid: "gf-prefetch",
        rel: "dns-prefetch",
        href: "https://fonts.gstatic.com/",
      });
    }

    // https://developer.mozilla.org/en-US/docs/Web/Performance/dns-prefetch#Best_practices
    // connect to domain of font files
    if (options.preconnect) {
      // @ts-ignore
      nuxt.options.meta.link.push({
        hid: "gf-preconnect",
        rel: "preconnect",
        href: "https://fonts.gstatic.com/",
        crossorigin: "",
      });
    }

    // https://developer.mozilla.org/pt-BR/docs/Web/HTML/Preloading_content
    // optionally increase loading priority
    if (options.preload) {
      // @ts-ignore
      nuxt.options.meta.link.push({
        hid: "gf-preload",
        rel: "preload",
        as: "style",
        href: url,
      });
    }

    // append CSS
    if (options.useStylesheet) {
      // @ts-ignore
      nuxt.options.meta.link.push({
        hid: "gf-style",
        rel: "stylesheet",
        href: url,
      });

      return;
    }

    // JS to inject CSS
    // @ts-ignore
    nuxt.options.meta.script = nuxt.options.meta.script || [];
    // @ts-ignore
    nuxt.options.meta.script.push({
      hid: "gf-script",
      innerHTML: `(function(){var l=document.createElement('link');l.rel="stylesheet";l.href="${url}";document.querySelector("head").appendChild(l);})();`,
    });

    // no-JS fallback
    // @ts-ignore
    nuxt.options.meta.noscript = nuxt.options.meta.noscript || [];
    // @ts-ignore
    nuxt.options.meta.noscript.push({
      hid: "gf-noscript",
      innerHTML: `<link rel="stylesheet" href="${url}">`,
    });

    // Disable sanitazions
    // @ts-ignore
    nuxt.options.meta.__dangerouslyDisableSanitizersByTagID =
      nuxt.options.meta.__dangerouslyDisableSanitizersByTagID || {};
    // @ts-ignore
    nuxt.options.meta.__dangerouslyDisableSanitizersByTagID["gf-script"] = [
      "innerHTML",
    ];
    // @ts-ignore
    nuxt.options.meta.__dangerouslyDisableSanitizersByTagID["gf-noscript"] = [
      "innerHTML",
    ];
  },
});
