import { defineConfig } from "vitepress";

export default defineConfig({
  title: "tardigrada",
  description: "age-identity lifecycle tool — dormant identity that persists anywhere",
  base: "/tardigrada/",
  srcExclude: ["README.md"],
  themeConfig: {
    nav: [
      { text: "9aia", link: "https://9aia.com", target: "_blank" },
      { text: "Getting Started", link: "/tutorials/getting-started" },
      { text: "How-to", link: "/how-to/move-your-identity" },
      { text: "Explanation", link: "/explanation/architecture" },
      { text: "Reference", link: "/reference/cli" },
    ],
    sidebar: [
      {
        text: "Tutorials",
        items: [{ text: "Getting started", link: "/tutorials/getting-started" }],
      },
      {
        text: "How-to",
        items: [
          { text: "Move your identity", link: "/how-to/move-your-identity" },
          { text: "Share your recipient", link: "/how-to/share-your-recipient" },
          { text: "Point cuora at tardigrada's key", link: "/how-to/cuora-interop" },
        ],
      },
      {
        text: "Explanation",
        items: [
          { text: "Architecture", link: "/explanation/architecture" },
          { text: "Concepts", link: "/explanation/concepts" },
          { text: "Boundaries", link: "/explanation/boundaries" },
          { text: "Decisions", link: "/explanation/decisions" },
        ],
      },
      {
        text: "Reference",
        items: [
          { text: "CLI", link: "/reference/cli" },
          { text: "Output contract", link: "/reference/output" },
          { text: "Environment variables", link: "/reference/env-vars" },
        ],
      },
    ],
    socialLinks: [{ icon: "github", link: "https://github.com/9aia/tardigrada" }],
  },
});
