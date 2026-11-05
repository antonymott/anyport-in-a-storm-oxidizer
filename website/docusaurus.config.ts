import { themes as prismThemes } from "prism-react-renderer"
import type { Config } from "@docusaurus/types"
import type * as Preset from "@docusaurus/preset-classic"

const repoUrl = "https://github.com/antonymott/anyport-in-a-storm-oxidizer"

const config: Config = {
  title: "Anyport in a Storm (Oxidizer)",
  tagline: "Oxidizing country-level packets, one subnet at a time.",
  favicon: "img/favicon.png",

  future: {
    v4: true,
  },

  url: "https://antonymott.github.io",
  baseUrl: "/anyport-in-a-storm-oxidizer/",

  organizationName: "antonymott",
  projectName: "anyport-in-a-storm-oxidizer",
  trailingSlash: false,

  onBrokenLinks: "warn",
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: "warn",
    },
  },

  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  presets: [
    [
      "classic",
      {
        docs: {
          sidebarPath: "./sidebars.ts",
          routeBasePath: "/",
          editUrl: `${repoUrl}/tree/main/website/`,
          sidebarCollapsed: false,
        },
        blog: false,
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: "img/emblem.png",
    colorMode: {
      defaultMode: "dark",
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: "⚡ Oxidizer / RustyKey",
      logo: {
        alt: "oxidizer",
        src: "img/emblem.png",
      },
      items: [
        {
          type: "docSidebar",
          sidebarId: "docsSidebar",
          position: "left",
          label: "Docs",
        },
        {
          href: repoUrl,
          label: "GitHub",
          position: "right",
        },
      ],
    },
    footer: {
      style: "dark",
      copyright: `Copyright © ${new Date().getFullYear()} Antony R Mott / RustyKey® a FIDO® Alliance Member.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.vsDark,
      additionalLanguages: ["bash", "rust", "toml"],
    },
  } satisfies Preset.ThemeConfig,
}

export default config
