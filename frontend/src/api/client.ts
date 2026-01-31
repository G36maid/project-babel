/**
 * API Client Configuration
 * 
 * Uses VITE_BACKEND_URL environment variable with fallback to '/api'
 * This allows the frontend to connect to different backend URLs
 * based on the deployment environment.
 */

// API base URL configuration
// VITE_BACKEND_URL is set at build time via .env files
// Falls back to '/api' for same-origin deployments (e.g., Docker, production)
export const API_BASE_URL = import.meta.env.VITE_BACKEND_URL || '/api'

/**
 * Get the full API URL for a given endpoint
 * @param endpoint - API endpoint path (e.g., '/login', '/rooms')
 * @returns Full URL string
 */
export function getApiUrl(endpoint: string): string {
  const base = API_BASE_URL.endsWith('/') ? API_BASE_URL.slice(0, -1) : API_BASE_URL
  const path = endpoint.startsWith('/') ? endpoint : `/${endpoint}`
  return `${base}${path}`
}

/**
 * Get WebSocket URL for a given room and token
 * @param roomId - Room ID to connect to
 * @param token - Authentication token
 * @returns WebSocket URL string
 */
export function getWebSocketUrl(roomId: string, token: string): string {
  // Convert HTTP(S) URL to WS(S) URL
  let wsBase: string
  
  if (API_BASE_URL.startsWith('http://')) {
    wsBase = API_BASE_URL.replace('http://', 'ws://')
  } else if (API_BASE_URL.startsWith('https://')) {
    wsBase = API_BASE_URL.replace('https://', 'wss://')
  } else {
    // Relative path (e.g., '/api') - use same host with ws://
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    wsBase = `${protocol}//${window.location.host}${API_BASE_URL}`
  }
  
  const base = wsBase.endsWith('/') ? wsBase.slice(0, -1) : wsBase
  return `${base}/rooms/${roomId}/connect?token=${token}`
}

/**
 * Wrapper around fetch API with error handling
 * @param endpoint - API endpoint
 * @param options - Fetch options
 * @returns Promise with parsed JSON response
 */
export async function apiFetch<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const url = getApiUrl(endpoint)
  
  const defaultOptions: RequestInit = {
    headers: {
      'Content-Type': 'application/json',
    },
  }
  
  const response = await fetch(url, { ...defaultOptions, ...options })
  
  if (!response.ok) {
    const error = new Error(`API Error: ${response.status} ${response.statusText}`)
    ;(error as any).status = response.status
    throw error
  }
  
  // Handle empty responses
  const contentType = response.headers.get('content-type')
  if (contentType && contentType.includes('application/json')) {
    return response.json() as Promise<T>
  }
  
  // Return text for non-JSON responses
  return response.text() as unknown as Promise<T>
}

/**
 * API Client object with common HTTP methods
 */
export const apiClient = {
  get: <T>(endpoint: string, headers?: HeadersInit) => 
    apiFetch<T>(endpoint, { method: 'GET', headers }),
    
  post: <T>(endpoint: string, body?: unknown, headers?: HeadersInit) =>
    apiFetch<T>(endpoint, { 
      method: 'POST', 
      body: body ? JSON.stringify(body) : undefined,
      headers 
    }),
    
  put: <T>(endpoint: string, body?: unknown, headers?: HeadersInit) =>
    apiFetch<T>(endpoint, { 
      method: 'PUT', 
      body: body ? JSON.stringify(body) : undefined,
      headers 
    }),
    
  delete: <T>(endpoint: string, headers?: HeadersInit) =>
    apiFetch<T>(endpoint, { method: 'DELETE', headers }),
}
