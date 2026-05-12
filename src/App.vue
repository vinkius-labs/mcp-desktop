<script setup lang="ts">
import { onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import TitleBar from '@/components/layout/TitleBar.vue'
import Sidebar from '@/components/layout/Sidebar.vue'
import LoginModal from '@/components/auth/LoginModal.vue'
import DeepLinkInstallModal from '@/components/shared/DeepLinkInstallModal.vue'
import { useAuthStore } from '@/stores/auth'
import { useClientsStore } from '@/stores/clients'
import { useSettings } from '@/composables/useSettings'
import { useDeepLink } from '@/composables/useDeepLink'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()
const clientsStore = useClientsStore()
const { init: initSettings } = useSettings()
const { setupDeepLinkListeners } = useDeepLink()

/** Routes that should NOT display the app shell (sidebar, statusbar) */
const shelllessRoutes = ['welcome']

onMounted(async () => {
  auth.setupListeners()
  clientsStore.setupListeners()
  setupDeepLinkListeners()

  // Load settings from tauri-plugin-store and apply theme + tray + autostart
  await initSettings()

  // Check existing session (reads stored tokens)
  await auth.checkSession()

  // Kick off client discovery regardless of auth state map
  clientsStore.discover()

  // If we are artificially on welcome, redirect to dashboard in unauth flow (or keep welcome if strictly needed later)
  if (route.name === 'welcome') {
    router.replace('/dashboard')
  }
})
</script>

<template>
  <div data-theme-root class="flex h-screen w-screen flex-col overflow-hidden dark">
    <!-- Drag region + window controls (always visible, full width) -->
    <div data-tauri-drag-region class="relative h-16 shrink-0">
      <TitleBar />
    </div>

    <!-- Main content area -->
    <template v-if="shelllessRoutes.includes(String(route.name))">
      <!-- Full-screen views (welcome, login) -->
      <main class="flex-1 overflow-hidden">
        <router-view />
      </main>
    </template>

    <template v-else>
      <!-- App shell: Sidebar + Content (Spotify layout) -->
      <div class="flex flex-1 overflow-hidden gap-2 px-2 pb-2">
        <Sidebar />
        <main class="flex-1 overflow-y-auto rounded-xl bg-background">
          <router-view />
        </main>
      </div>
    </template>

    <!-- Global Application Modals -->
    <LoginModal />
    <DeepLinkInstallModal />
  </div>
</template>