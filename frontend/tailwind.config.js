/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        telegram: {
          // Dark mode colors
          'bg': '#17212b',
          'bg-secondary': '#232e3c',
          'bg-chat': '#0e1621',
          'message-out': '#2b5278',
          'message-in': '#182533',
          'text': '#f5f5f5',
          'text-secondary': '#708499',
          'accent': '#5288c1',
          'link': '#6ab3f3',
          'destructive': '#ec3942',
          // Light mode colors
          'bg-light': '#ffffff',
          'bg-secondary-light': '#efeff3',
          'bg-chat-light': '#ffffff',
          'message-out-light': '#dcf8c6',
          'message-in-light': '#ffffff',
          'text-light': '#000000',
          'text-secondary-light': '#999999',
          'accent-light': '#2481cc',
          'link-light': '#2481cc',
        }
      }
    },
  },
  plugins: [],
}

