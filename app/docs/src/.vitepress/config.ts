import { defineConfig } from "vitepress";
import dotenv from "dotenv";
import path from "path";
import lightbox from "vitepress-plugin-lightbox";

dotenv.config();

export default defineConfig({
  title: "System Initiative Docs",
  description: "The documentation for System Initiative https://systeminit.com",
  markdown: {
    theme: {
      light: "github-light",
      dark: "github-dark",
    },
    config: (md) => {
      md.use(lightbox, {});
    },
  },
  head: [
    [
      "script",
      {},
      `
    !function(t,e){var o,n,p,r;e.__SV||(window.posthog=e,e._i=[],e.init=function(i,s,a){function g(t,e){var o=e.split(".");2==o.length&&(t=t[o[0]],e=o[1]),t[e]=function(){t.push([e].concat(Array.prototype.slice.call(arguments,0)))}}(p=t.createElement("script")).type="text/javascript",p.async=!0,p.src=s.api_host.replace(".i.posthog.com","-assets.i.posthog.com")+"/static/array.js",(r=t.getElementsByTagName("script")[0]).parentNode.insertBefore(p,r);var u=e;for(void 0!==a?u=e[a]=[]:a="posthog",u.people=u.people||[],u.toString=function(t){var e="posthog";return"posthog"!==a&&(e+="."+a),t||(e+=" (stub)"),e},u.people.toString=function(){return u.toString(1)+".people (stub)"},o="capture identify alias people.set people.set_once set_config register register_once unregister opt_out_capturing has_opted_out_capturing opt_in_capturing reset isFeatureEnabled onFeatureFlags getFeatureFlag getFeatureFlagPayload reloadFeatureFlags group updateEarlyAccessFeatureEnrollment getEarlyAccessFeatures getActiveMatchingSurveys getSurveys getNextSurveyStep onSessionId setPersonProperties".split(" "),n=0;n<o.length;n++)g(u,o[n]);e._i.push([i,s,a])},e.__SV=1)}(document,window.posthog||[]);
    posthog.init('${process.env.VITE_POSTHOG_PUBLIC_KEY}',{api_host:'${process.env.VITE_POSTHOG_API_HOST}',});
`,
    ],
    ["link", { rel: "icon", href: "/favicon.png" }],
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
      { text: "Explanation", link: "/explanation/" },
      { text: "Changelog", link: "/changelog/" },
      {
        text: "Log In",
        link: "https://auth.systeminit.com/login",
        target: "__self",
      },
    ],
    socialLinks: [
      {
        icon: "github",
        link: "https://github.com/systeminit/si",
      },
      {
        icon: "discord",
        link: "https://discord.com/invite/system-init",
      },
      {
        icon: {
          svg: '<svg viewBox="0 0 216 216" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><path d="m275.42 59.42h-216v216h216zm-78.29 206.56h-128.28v-59.42h128.28zm68.85 0h-59.42v-68.85h-137.71v-128.28h128.27v59.42h-59.42v9.44h128.27v128.27zm0-137.71h-59.42v-59.42h59.42z" fill-rule="nonzero" transform="translate(-59.42 -59.42)"/></svg>',
        },
        link: "https://systeminit.com/",
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
            text: "Working with Components",
            link: "/tutorials/working-with-components",
          },
          {
            text: "Actions and Resources",
            link: "/tutorials/actions-and-resources",
          },
          {
            text: "Creating Components",
            link: "/tutorials/creating-components",
          },
          {
            text: "Editing and Contributing Components",
            link: "/tutorials/editing-components-and-contributing",
          },
        ],
      },
      {
        text: "How To Guides",
        link: "/how-tos/",
        collapsed: false,
        items: [
          {
            text: "Use the Public API",
            link: "/how-tos/use-public-api",
          },
          {
            text: "AWS IAM Policy",
            link: "/how-tos/aws-iam",
          },
          {
            text: "AWS VPC",
            link: "/how-tos/aws-vpc",
          },
          {
            text: "AWS HA EC2",
            link: "/how-tos/aws-ha-ec2",
          },
          {
            text: "AWS ECS",
            link: "/how-tos/aws-ecs",
          },
          {
            text: "AWS ECR ECS",
            link: "/how-tos/aws-ecr-ecs",
          },
          {
            text: "AWS Static Cloudfront Website",
            link: "/how-tos/aws-static-cloudfront",
          },
          {
            text: "AWS macOS Instances",
            link: "/how-tos/aws-macos",
          },
          {
            text: "AWS EKS",
            link: "/how-tos/aws-eks",
          },
          {
            text: "AWS Lambda",
            link: "/how-tos/aws-lambda",
          },
        ],
      },
      {
        text: "Reference Guides",
        link: "/reference/",
        collapsed: false,
        items: [
          {
            text: "Asset Schema",
            link: "/reference/asset/schema",
          },
          {
            text: "Asset Functions",
            link: "/reference/asset/function",
          },
          {
            text: "Authorization Roles",
            link: "/reference/authorization-roles",
          },
          {
            text: "GitHub Action",
            link: "/reference/github-action",
          },
          {
            text: "Public API",
            link: "/reference/public-api",
          },
          {
            text: "TypeScript Function API",
            link: "/reference/typescript/README",
          },
          {
            text: "Vocabulary",
            link: "/reference/vocabulary",
          },
        ],
      },
      {
        text: "Explanation",
        link: "/explanation/",
        collapsed: false,
        items: [
          {
            text: "AWS Authentication",
            link: "/explanation/aws-authentication",
          },
          {
            text: "Working on System Initiative",
            link: "/explanation/working-on-si",
          },
          {
            text: "Enable Slack Webhook",
            link: "/explanation/enable-slack-webhook",
          },
          {
            text: "Create Workspace API Tokens",
            link: "/explanation/generate-a-workspace-api-token",
          },
        ],
      },
      {
        text: "Changelog",
        link: "/changelog/",
      },
      {
        text: "Road map",
        link: "/roadmap/",
      },
      {
        text: "System Initiative Website",
        link: "https://systeminit.com/",
      },
    ],
    editLink: {
      pattern: "https://github.com/systeminit/si/edit/main/app/docs/src/:path",
      text: "Edit this page on GitHub",
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
