import { ref, computed, watch, onMounted } from 'vue'

const THEME_KEY = 'telegram-theme'
type Theme = 'dark' | 'light'

export function useTheme() {
  const theme = ref<Theme>('dark')

  const isDark = computed(() => theme.value === 'dark')
  const isLight = computed(() => theme.value === 'light')

  function setTheme(newTheme: Theme) {
    theme.value = newTheme
    document.documentElement.setAttribute('data-theme', newTheme)
    localStorage.setItem(THEME_KEY, newTheme)
  }

  function toggleTheme() {
    const newTheme: Theme = theme.value === 'dark' ? 'light' : 'dark'
    setTheme(newTheme)
  }

  function initTheme() {
    const stored = localStorage.getItem(THEME_KEY) as Theme | null
    if (stored && (stored === 'dark' || stored === 'light')) {
      setTheme(stored)
    } else {
      setTheme('dark')
    }
  }

  onMounted(() => {
    initTheme()
  })

  return {
    theme,
    isDark,
    isLight,
    setTheme,
    toggleTheme,
    initTheme
  }
}
