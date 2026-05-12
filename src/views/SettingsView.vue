<script setup lang="ts">
import { onMounted } from 'vue'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import {
  Sun, Moon, Monitor, ExternalLink,
  Globe, FileText, Shield, BookOpen,
} from 'lucide-vue-next'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useSettings } from '@/composables/useSettings'

// ── Shared settings (persisted via tauri-plugin-store) ───────────────
const { settings, init, setTheme, setStartAtLogin, setShowInTray } = useSettings()

type ThemeOption = 'system' | 'light' | 'dark'

const themeOptions: { value: ThemeOption; label: string; icon: typeof Sun }[] = [
  { value: 'light', label: 'Light', icon: Sun },
  { value: 'dark', label: 'Dark', icon: Moon },
  { value: 'system', label: 'System', icon: Monitor },
]


const aboutLinks = [
  { label: 'Website', url: 'https://vinkius.com', icon: Globe },
  { label: 'Documentation', url: 'https://docs.vinkius.com/getting-started', icon: BookOpen },
  { label: 'Terms of Service', url: 'https://vinkius.com/legal/terms-and-conditions', icon: FileText },
  { label: 'Privacy Policy', url: 'https://vinkius.com/legal/privacy-policy', icon: Shield },
]

function openExternal(url: string) {
  openUrl(url)
}

onMounted(() => {
  init()
})
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">

    <!-- Sticky header -->
    <div class="flex-none px-6 py-3 shrink-0 bg-background flex items-baseline justify-between gap-4">
      <h1 class="text-xl font-bold tracking-tight text-foreground leading-normal">
        Settings
      </h1>
      <p class="text-muted-foreground text-sm shrink-0">
        Preferences &amp; appearance
      </p>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto px-6 py-2">
      <div class="space-y-0">

        <!-- APPEARANCE -->
        <div class="py-3">
          <div class="flex items-baseline justify-between mb-2.5">
            <Label class="text-muted-foreground text-xs uppercase tracking-wider font-semibold">Appearance</Label>
            <span class="text-muted-foreground/50 text-xs">Choose how Vinkius looks on your machine.</span>
          </div>

          <div class="grid grid-cols-3 gap-1.5">
            <button
              v-for="opt in themeOptions"
              :key="opt.value"
              type="button"
              class="group flex items-center gap-2.5 px-3 py-2.5 rounded-lg border transition-all select-none"
              :class="settings.theme === opt.value
                ? 'bg-primary/5 border-primary/30 ring-1 ring-primary/20'
                : 'bg-transparent border-border hover:bg-muted/40 hover:border-border'"
              @click="setTheme(opt.value)"
            >
              <div
                class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md transition-colors"
                :class="settings.theme === opt.value ? 'bg-primary/10 text-foreground' : 'bg-muted text-muted-foreground/50'"
              >
                <component :is="opt.icon" class="h-4 w-4" />
              </div>
              <span
                class="text-xs font-medium transition-colors"
                :class="settings.theme === opt.value ? 'text-foreground' : 'text-muted-foreground group-hover:text-foreground'"
              >
                {{ opt.label }}
              </span>
            </button>
          </div>
        </div>

        <div class="h-px bg-border" />

        <!-- GENERAL -->
        <div class="py-3">
          <div class="flex items-baseline justify-between mb-2.5">
            <Label class="text-muted-foreground text-xs uppercase tracking-wider font-semibold">General</Label>
            <span class="text-muted-foreground/50 text-xs">Startup and system tray behavior.</span>
          </div>

          <div class="flex flex-col gap-0">
            <!-- Start at login -->
            <div class="flex items-center justify-between rounded-lg px-3 py-2.5 hover:bg-muted/40 transition-colors -mx-0.5">
              <div>
                <Label for="start-at-login" class="text-sm font-medium text-foreground cursor-pointer">
                  Start at login
                </Label>
                <p class="text-xs text-muted-foreground mt-0.5">Launch Vinkius automatically when you sign in.</p>
              </div>
              <Switch
                id="start-at-login"
                :checked="settings.start_at_login"
                @update:checked="setStartAtLogin"
              />
            </div>

            <!-- Show in tray -->
            <div class="flex items-center justify-between rounded-lg px-3 py-2.5 hover:bg-muted/40 transition-colors -mx-0.5">
              <div>
                <Label for="show-tray" class="text-sm font-medium text-foreground cursor-pointer">
                  Show in system tray
                </Label>
                <p class="text-xs text-muted-foreground mt-0.5">Keep Vinkius accessible from the notification area.</p>
              </div>
              <Switch
                id="show-tray"
                :checked="settings.show_in_tray"
                @update:checked="setShowInTray"
              />
            </div>
          </div>
        </div>

        <div class="h-px bg-border" />

        <!-- ABOUT -->
        <div class="py-3">
          <div class="flex items-baseline justify-between mb-2.5">
            <Label class="text-muted-foreground text-xs uppercase tracking-wider font-semibold">About</Label>
          </div>


          <!-- Brand identity -->
          <div class="mb-4 px-1">
            <span class="text-sm font-semibold text-foreground">Vinkius Desktop</span>
            <p class="text-xs text-muted-foreground mt-0.5">
              The governed infrastructure for the AI era.
            </p>
          </div>

          <!-- Links grid -->
          <div class="grid grid-cols-2 gap-0.5 -mx-0.5">
            <button
              v-for="link in aboutLinks"
              :key="link.url"
              type="button"
              class="group flex items-center gap-2.5 px-3 py-2 rounded-lg hover:bg-muted/40 transition-colors text-left"
              @click="openExternal(link.url)"
            >
              <div class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground/50 group-hover:text-foreground group-hover:bg-muted/80 transition-colors">
                <component :is="link.icon" class="h-3.5 w-3.5" />
              </div>
              <span class="text-xs font-medium text-muted-foreground group-hover:text-foreground transition-colors">
                {{ link.label }}
              </span>
              <ExternalLink class="h-2.5 w-2.5 ml-auto text-muted-foreground/30 opacity-0 group-hover:opacity-100 transition-opacity" />
            </button>
          </div>

          <!-- Footer -->
          <p class="text-[10px] text-muted-foreground/40 mt-3 px-1">
            Built in Europe · © {{ new Date().getFullYear() }} Vinkius
          </p>
        </div>

      </div>
    </div>
  </div>
</template>
