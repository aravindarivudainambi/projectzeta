import type { Config } from "tailwindcss";

/**
 * Declares the Tailwind content sources used by the frontend application.
 *
 * Extend this configuration once the design system tokens and plugin list are finalized.
 */
const config: Config = {
  content: ["./app/**/*.{ts,tsx}", "./components/**/*.{ts,tsx}"],
  theme: {
    extend: {},
  },
  plugins: [],
};

export default config;
