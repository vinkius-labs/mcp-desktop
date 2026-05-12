import { ref } from 'vue'
import { tauriListen } from '@/composables/useTauri'

/**
 * Deep link install payload emitted by the Rust backend when the app
 * receives a `vinkius://install/{slug}?...` URL from the website.
 */
export interface DeepLinkInstallPayload {
  slug: string
  name: string
  command: string
  args: string[]
  transport: string // 'stdio' | 'http'
  url: string
  env: Record<string, string>
}

/** Reactive state shared between the composable and the modal component */
const showInstallModal = ref(false)
const pendingInstall = ref<DeepLinkInstallPayload | null>(null)

/**
 * Composable that listens for `deep-link:install` events from the Rust backend.
 * Call `setupDeepLinkListeners()` once in App.vue on mount.
 */
export function useDeepLink() {
  function setupDeepLinkListeners() {
    tauriListen('deep-link:install', (payload) => {
      const data = payload as DeepLinkInstallPayload
      pendingInstall.value = data
      showInstallModal.value = true
    })
  }

  function dismissInstall() {
    showInstallModal.value = false
    pendingInstall.value = null
  }

  return {
    showInstallModal,
    pendingInstall,
    setupDeepLinkListeners,
    dismissInstall,
  }
}
