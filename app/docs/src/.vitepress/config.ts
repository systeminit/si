import { defineConfig } from "vitepress";
import dotenv from "dotenv";
import path from "path";

dotenv.config();

export default defineConfig({
  title: "System Initiative Docs",
  description: "The documentation for System Initiative https://systeminit.com",
  markdown: {
    theme: {
      light: "github-light",
      dark: "github-dark",
    },
  },
  head: [
    ['script', {}, `
    !function(t,e){var o,n,p,r;e.__SV||(window.posthog=e,e._i=[],e.init=function(i,s,a){function g(t,e){var o=e.split(".");2==o.length&&(t=t[o[0]],e=o[1]),t[e]=function(){t.push([e].concat(Array.prototype.slice.call(arguments,0)))}}(p=t.createElement("script")).type="text/javascript",p.async=!0,p.src=s.api_host.replace(".i.posthog.com","-assets.i.posthog.com")+"/static/array.js",(r=t.getElementsByTagName("script")[0]).parentNode.insertBefore(p,r);var u=e;for(void 0!==a?u=e[a]=[]:a="posthog",u.people=u.people||[],u.toString=function(t){var e="posthog";return"posthog"!==a&&(e+="."+a),t||(e+=" (stub)"),e},u.people.toString=function(){return u.toString(1)+".people (stub)"},o="capture identify alias people.set people.set_once set_config register register_once unregister opt_out_capturing has_opted_out_capturing opt_in_capturing reset isFeatureEnabled onFeatureFlags getFeatureFlag getFeatureFlagPayload reloadFeatureFlags group updateEarlyAccessFeatureEnrollment getEarlyAccessFeatures getActiveMatchingSurveys getSurveys getNextSurveyStep onSessionId setPersonProperties".split(" "),n=0;n<o.length;n++)g(u,o[n]);e._i.push([i,s,a])},e.__SV=1)}(document,window.posthog||[]);
    posthog.init('${process.env.VITE_POSTHOG_PUBLIC_KEY}',{api_host:'${process.env.VITE_POSTHOG_API_HOST}',});
`],
    ['link', { rel: "icon", href: "/favicon.png" }],
  ],
  lastUpdated: true,
  outDir: path.join(__dirname, "../../dist"),
  ignoreDeadLinks: false,
  cleanUrls: true,
  themeConfig: {
    logo: {
      dark: "/si-logo-symbol-dark-mode.svg",
      light: "/si-logo-symbol.svg",
      alt: "System Initiative",
    },
    nav: [
      { text: "Home", link: "/" },
      { text: "Tutorials", link: "/tutorials/" },
      { text: "How To", link: "/how-tos/" },
      { text: "Reference Guides", link: "/reference/" },
      { text: "Changelog", link: "/changelog/" },
      { text: "Log In", link: 'https://auth.systeminit.com/login', target: '__self', },
    ],
    socialLinks: [
      {
        icon: 'github', link: "https://github.com/systeminit/si",
      },
      {
        icon: "discord", link: "https://discord.com/invite/system-init",
      },
    ],
    sidebar: [
      {
        text: "Tutorials",
        collapsed: false,
        link: "/tutorials/",
        items: [
          {
            text: "Getting Started",
            link: "/tutorials/getting-started",
          },
          {
            text: "Creating a new Asset",
            link: "/tutorials/creating-new-assets",
          }
        ]
      },
      {
        text: "How To Guides",
        link: "/how-tos/",
        collapsed: false,
        items: [
          {
            text: "AWS IAM Policy",
            link: "/how-tos/aws-iam",
          },
          {
            text: "AWS VPC",
            link: "/how-tos/aws-vpc",
          }
        ]
      },
      {
        text: "Reference Guides",
        link: "/reference/",
        collapsed: false,
        items: [
          {
            text: "Vocabulary",
            link: "/reference/vocabulary",
          },
          {
            text: "TypeScript Function API",
            link: "/reference/typescript/README",
          },
        ]
      },
      {
        text: "Changelog",
        link: "/changelog/",
      }
    ],
    editLink: {
      pattern: 'https://github.com/systeminit/si/edit/main/app/docs/src/:path',
      text: 'Edit this page on GitHub',
    },
    search: {
      provider: "local",
    },
  },
  vite: {
    server: {
      host: process.env.DEV_HOST,
      port: parseInt(process.env.DEV_PORT!, 10),
    },
  },
});
