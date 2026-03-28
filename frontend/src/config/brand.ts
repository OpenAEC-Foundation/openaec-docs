export interface BrandConfig {
  namePrefix: string;
  nameAccent: string;
  product: string;
  tagline: string;
  colors: {
    primary: string;
    secondary: string;
    headerBg: string;
    headerText: string;
    accentSoft: string;
    gold: string;
    orange: string;
  };
}

export const brand: BrandConfig = {
  namePrefix: "Open",
  nameAccent: "AEC",
  product: "Docs",
  tagline: "Document management voor BIM-projecten",
  colors: {
    primary: "#D97706",
    secondary: "#EA580C",
    headerBg: "#36363E",
    headerText: "#FAFAF9",
    accentSoft: "rgba(217,119,6,0.08)",
    gold: "#F59E0B",
    orange: "#EA580C",
  },
};
