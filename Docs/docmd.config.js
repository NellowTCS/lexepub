module.exports = {
  siteTitle: "LexePub",
  siteUrl: "https://nellowtcs.me/lexepub/docs",
  logo: { alt: "LexePub", href: "./" },
  favicon: "",
  srcDir: "docs",
  outputDir: "site",
  theme: {
    name: "ruby",
    defaultMode: "system",
    enableModeToggle: true,
    positionMode: "top",
    codeHighlight: true,
    customCss: [],
  },
  search: true,
  minify: true,
  autoTitleFromH1: true,
  copyCode: true,
  pageNavigation: true,
  navigation: [
    { title: "Home", path: "/", icon: "home" },
    {
      title: "Getting Started",
      icon: "rocket",
      collapsible: false,
      children: [
        { title: "Quick Start", path: "/getting-started/quickstart", icon: "play" },
      ],
    },
    {
      title: "Adapters",
      icon: "code",
      path: "/adapters/",
      children: [
        { title: "C/C++", path: "/adapters/c/", icon: "terminal" },
        { title: "Rust", path: "/adapters/rust/", icon: "box" },
        { title: "WASM", path: "/adapters/wasm/", icon: "cpu" },
      ],
    },
    { title: "GitHub", path: "https://github.com/NellowTCS/lexepub", icon: "github", external: true },
  ],
  footer: "Built with docmd.",
};
