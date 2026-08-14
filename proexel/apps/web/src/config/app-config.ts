import packageJson from "../../package.json";

const currentYear = new Date().getFullYear();

export const APP_CONFIG = {
  name: "PROEXEL",
  version: packageJson.version,
  copyright: `Copyright ${currentYear}, PROEXEL.`,
  meta: {
    title: "PROEXEL",
    description: "Maintenance operations dashboard for valves, service orders, stock, purchasing and audit.",
  },
};
