/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        "background": "rgba(33, 33, 33, 0.5)",
        "darker": "rgba(26, 26, 26, 0.5)",
      },
      borderWidth: {
        "1": "1px",
      }
    },
  },
  plugins: [],
}