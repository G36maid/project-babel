import { computed, onMounted, ref } from "vue";

const THEME_KEY = "telegram_theme";
type Theme = "light" | "dark";

export function useTheme() {
  const theme = ref<Theme>("dark");

  function setTheme(newTheme: Theme) {
    theme.value = newTheme;
    document.documentElement.setAttribute("data-theme", newTheme);
    localStorage.setItem(THEME_KEY, newTheme);
  }

  function toggleTheme() {
    setTheme(theme.value === "dark" ? "light" : "dark");
  }

  onMounted(() => {
    const savedTheme = localStorage.getItem(THEME_KEY) as Theme;
    if (savedTheme) {
      setTheme(savedTheme);
    } else {
      // System preference or default to dark
      const systemDark = window.matchMedia(
        "(prefers-color-scheme: dark)",
      ).matches;
      setTheme(systemDark ? "dark" : "light");
    }
  });

  const isDark = computed(() => theme.value === "dark");

  return {
    theme,
    isDark,
    toggleTheme,
    setTheme,
  };
}
