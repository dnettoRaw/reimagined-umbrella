import packageJson from "../../package.json";

const currentYear = new Date().getFullYear();

export const APP_CONFIG = {
  name: "PROEXEL",
  version: packageJson.version,
  copyright: `Copyright ${currentYear}, PROEXEL.`,
  meta: {
    title: "PROEXEL",
    description: "Industrial asset, guided maintenance, service order, stock, purchasing and audit management.",
  },
};
