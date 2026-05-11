import type { Config } from "tailwindcss";

const config = {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        voicebox: {
          black: "#0A0A0A",
          red: "#EF4444",
          background: "#FAFAFA",
          surface: "#F5F5F5",
          raised: "#E5E5E5",
          secondary: "#525252",
          tertiary: "#A3A3A3",
          border: "#D4D4D4",
          success: "#16A34A",
          warning: "#CA8A04",
        },
      },
      fontFamily: {
        display: ["Archivo Black", "Impact", "Arial Black", "sans-serif"],
        sans: ["Work Sans", "-apple-system", "BlinkMacSystemFont", "Segoe UI", "sans-serif"],
        mono: ["Space Mono", "Courier New", "Consolas", "monospace"],
      },
      borderRadius: {
        none: "0",
      },
      boxShadow: {
        none: "none",
      },
    },
  },
  plugins: [],
} satisfies Config;

export default config;
