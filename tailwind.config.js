/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        "background": "#212121",
        "darker": "#1a1a1a",
        "bright": "#4a4a4a"
      },
      borderWidth: {
        "1": "1px",
      }
    },
  },
  plugins: [],
}