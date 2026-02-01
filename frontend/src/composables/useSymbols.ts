import { ref, readonly } from 'vue'
import symbolsData from '@/assets/symbols.json'

export interface SymbolConfig {
  svg: string
  emoji: string
}

const symbols = ref<Record<string, SymbolConfig>>(symbolsData)

/**
 * Composable to manage symbol lookups
 * Provides access to the symbol configuration for rendering
 */
export function useSymbols() {
  /**
   * Get symbol configuration for a specific word
   * @param word - The word to look up
   * @returns Symbol config or undefined if not found
   */
  function getSymbol(word: string): SymbolConfig | undefined {
    return symbols.value[word]
  }

  /**
   * Get all available symbols
   * @returns Record of all symbol configurations
   */
  function getAllSymbols(): Record<string, SymbolConfig> {
    return symbols.value
  }

  return {
    getSymbol,
    getAllSymbols,
    symbols: readonly(symbols),
  }
}
