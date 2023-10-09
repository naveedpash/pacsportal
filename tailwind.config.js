module.exports = {
  content: [
    "index.html",
    "./src/**/*.rs",
    "./src/**/*.html",
    "./src/**/*.css"
  ],
  theme: {
    extend: {
      colors: {
        'black': '#040404',
        'red': '#D41C24',
        'grey': '#8A8887',
        'yellow': '#F5CE04'
      }
    },
  },
  plugins: [
    require('@tailwindcss/forms'),
  ],
}
