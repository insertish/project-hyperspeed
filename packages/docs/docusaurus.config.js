const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');
const remarkFootnotes = require('remark-footnotes');

/** @type {import('@docusaurus/types').DocusaurusConfig} */
module.exports = {
  title: 'Project Hyperspeed',
  tagline: 'Rust FTL + WebRTC live streaming software. Faster Than Light protocol documentation.',
  url: 'https://project-hyperspeed.vercel.app',
  baseUrl: '/',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'throw',
  favicon: 'img/favicon.ico',
  organizationName: 'insertish',
  projectName: 'project-hyperspeed',
  themeConfig: {
    navbar: {
      title: 'Project Hyperspeed',
      logo: {
        alt: 'Project Hyperspeed Logo',
        src: 'img/logo.png',
      },
      items: [
        {
          type: 'doc',
          docId: 'intro',
          position: 'left',
          label: 'Introduction',
        },
        {
          type: 'doc',
          docId: 'ftl/overview',
          position: 'left',
          label: 'FTL',
        },
        {
          href: 'https://gitlab.insrt.uk/insert/project-hyperspeed',
          label: 'GitLab',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Overview',
              to: '/',
            },
            {
              label: 'Faster Than Light',
              to: '/ftl',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'Revolt',
              href: 'https://app.revolt.chat/invite/fqqT4MWM',
            },
            {
              label: 'Discord',
              href: 'https://discord.gg/XJPXxdYUxn',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'GitLab',
              href: 'https://gitlab.insrt.uk/insert/project-hyperspeed',
            },
            {
              label: 'GitHub',
              href: 'https://github.com/insertish/project-hyperspeed',
            },
          ],
        },
      ],
      copyright: `Made by <a href="https://insrt.uk">Paul Makles</a>. Built with Docusaurus.`,
    },
    prism: {
      theme: lightCodeTheme,
      darkTheme: darkCodeTheme,
      additionalLanguages: ["rust"]
    },
  },
  presets: [
    [
      '@docusaurus/preset-classic',
      {
        docs: {
          routeBasePath: '/',
          sidebarPath: require.resolve('./sidebars.js'),
          // Please change this to your repo.
          editUrl:
            'https://gitlab.insrt.uk/insert/project-hyperspeed/-/edit/master/packages/docs/',
          remarkPlugins: [
            [remarkFootnotes, { inlineNotes: true }]
          ],
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],
};
