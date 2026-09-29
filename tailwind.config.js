/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        brand: {
          50: "#f2f7f4",
          100: "#dcece2",
          400: "#3d8f66",
          500: "#237049",
          600: "#1b5a3a",
          700: "#164a30",
          900: "#0f331f",
        },
      },
    },
  },
  plugins: [],
};
