import type { SidebarsConfig } from "@docusaurus/plugin-content-docs"

const sidebars: SidebarsConfig = {
  docsSidebar: [
    "intro",
    {
      type: "category",
      label: "Concepts",
      collapsed: false,
      items: ["concepts/pqc-amplification", "concepts/ingress-hook"],
    },
    {
      type: "category",
      label: "Usage",
      collapsed: false,
      items: ["usage/install", "usage/running", "usage/geoip"],
    },
    "safety",
    {
      type: "category",
      label: "Reference",
      collapsed: false,
      items: [
        "reference/cli",
        "reference/ruleset",
        "reference/hosts",
        "reference/logs",
        "reference/modules",
      ],
    },
    "contributing",
  ],
}

export default sidebars
