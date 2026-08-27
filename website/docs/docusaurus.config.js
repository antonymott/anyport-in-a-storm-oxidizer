// @ts-check
/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Anyport in a Storm (Oxidizer)',
  tagline: 'Oxidizing country-level packets, one subnet at a time.',
  favicon: 'img/favicon.ico',

  url: 'https://antonymott.github.io',
  baseUrl: '/anyport-in-a-storm-oxidizer/', // Match your GitHub repository name!

  organizationName: 'antonymott', 
  projectName: 'anyport-in-a-storm-oxidizer',

  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'warn',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: require.resolve('./sidebars.js'),
          routeBasePath: '/', // Serve docs at root URL of the site
          editUrl: 'https://github.com/antonymott/anyport-in-a-storm-oxidizer/tree/main/docs/',
        },
        blog: false, // Disable blog unless needed
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      colorMode: {
        defaultMode: 'dark',
        respectPrefersColorScheme: true,
      },
      navbar: {
        title: '⚡ Oxidizer / RustyKey',
        items: [
          {
            href: 'https://github.com/antonymott/anyport-in-a-storm-oxidizer',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        copyright: `Copyright © ${new Date().getFullYear()} Antony R Mott / RustyKey® a FIDO® Alliance Member.`,
      },
    }),
};

module.exports = config;