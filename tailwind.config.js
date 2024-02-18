/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["*.html", "./src/**/*.rs", "./node_modules/preline/dist/*.js"],
  theme: {
    screens: {
      sm: '480px',
      md: '768px',
      lg: '1020px',
      xl: '1440px',
    },
    fontFamily: {
      sans: ['Inter', 'sans-serif'],

    },
    extend: {
      //https://play.tailwindcss.com/VCZwwz1e3R
      animation: {
        text: 'text 5s ease infinite',
      },
      keyframes: {
        text: {
          '0%, 100%': {
            'background-size': '200% 200%',
            'background-position': 'left center',
          },
          '50%': {
            'background-size': '200% 200%',
            'background-position': 'right center',
          },
        },
      },
    },
  },
  darkMode: 'class',
  plugins: [require('@tailwindcss/forms'), require("@tailwindcss/typography"), require("preline/plugin")],
}
