import { DownloadOptions, GoogleFonts } from "google-fonts-helper";

export interface ModuleOptions extends Partial<DownloadOptions & GoogleFonts> {
  prefetch?: boolean;
  preconnect?: boolean;
  preload?: boolean;
  useStylesheet?: boolean;
  download?: boolean;
  inject?: boolean;
}

export const CONFIG_KEY = "googleFonts";

declare module "@nuxt/schema" {
  interface NuxtConfig {
    [CONFIG_KEY]?: ModuleOptions;
  } // Nuxt 3
}
