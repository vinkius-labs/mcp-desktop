<script setup lang="ts">
import { ref, computed } from 'vue'
import { useDeepLink } from '@/composables/useDeepLink'
import { useServersStore } from '@/stores/servers'
import { useClientsStore } from '@/stores/clients'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog'

const { showInstallModal, pendingInstall, dismissInstall } = useDeepLink()
const serversStore = useServersStore()
const clientsStore = useClientsStore()

type InstallState = 'confirm' | 'installing' | 'success' | 'error'
const state = ref<InstallState>('confirm')
const errorMessage = ref('')
const installResults = ref<Array<{ clientId: string; success: boolean; error?: string }>>([])

const detectedClients = computed(() => clientsStore.detectedClients)

const alreadyInstalled = computed(() => {
  if (!pendingInstall.value) return false
  return serversStore.servers.some(s => s.name === pendingInstall.value!.name)
})

async function handleInstall() {
  if (!pendingInstall.value || detectedClients.value.length === 0) return

  state.value = 'installing'
  errorMessage.value = ''
  installResults.value = []

  try {
    const clientIds = detectedClients.value.map(c => c.id)
    const results = await serversStore.installServer(
      {
        name: pendingInstall.value.name,
        transport: pendingInstall.value.transport,
        command: pendingInstall.value.command,
        args: pendingInstall.value.args,
        url: pendingInstall.value.url,
        env: pendingInstall.value.env,
      },
      clientIds
    )

    installResults.value = results.map(r => ({
      clientId: r.client_id,
      success: r.success,
      error: r.error ?? undefined,
    }))

    const allSuccess = results.every(r => r.success)
    state.value = allSuccess ? 'success' : 'error'

    if (!allSuccess) {
      const failures = results.filter(r => !r.success)
      errorMessage.value = failures.map(f => `${f.client_id}: ${f.error}`).join('\n')
    }
  } catch (e) {
    state.value = 'error'
    errorMessage.value = String(e)
  }
}

function handleOpenChange(open: boolean) {
  if (!open) {
    setTimeout(() => {
      state.value = 'confirm'
      errorMessage.value = ''
      installResults.value = []
    }, 200)
    dismissInstall()
  }
}

function getClientName(id: string): string {
  return clientsStore.getClient(id)?.name ?? id
}
</script>

<template>
  <Dialog :open="showInstallModal" @update:open="handleOpenChange">
    <DialogContent class="sm:max-w-[520px] !bg-black border-white/[0.06] p-0 overflow-hidden gap-0">

      <!-- Confirm State -->
      <template v-if="state === 'confirm' && pendingInstall">
        <div class="px-8 pt-10 pb-6 flex flex-col items-center text-center">
          <!-- Icon -->
          <div class="relative mb-5">
            <div class="h-14 w-14 rounded-2xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center">
              <svg class="h-7 w-7 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
              </svg>
            </div>
          </div>

          <DialogHeader class="space-y-2">
            <DialogTitle class="text-xl font-bold tracking-tight text-white">
              Install {{ pendingInstall.name }}
            </DialogTitle>
            <DialogDescription class="text-sm text-muted-foreground leading-relaxed max-w-[380px]">
              This MCP server will be added to all your detected AI clients in one click.
            </DialogDescription>
          </DialogHeader>

          <!-- Server info -->
          <div class="mt-6 w-full rounded-xl bg-white/[0.03] border border-white/[0.06] p-4 text-left space-y-3">
            <div class="flex items-center justify-between">
              <span class="text-xs text-muted-foreground uppercase tracking-wider">Transport</span>
              <Badge variant="outline" class="text-xs">{{ pendingInstall.transport }}</Badge>
            </div>
            <div v-if="pendingInstall.command" class="flex items-start justify-between gap-4">
              <span class="text-xs text-muted-foreground uppercase tracking-wider shrink-0">Command</span>
              <code class="text-xs text-white/70 font-mono text-right break-all">
                {{ pendingInstall.command }} {{ pendingInstall.args.join(' ') }}
              </code>
            </div>
            <div v-if="pendingInstall.url" class="flex items-start justify-between gap-4">
              <span class="text-xs text-muted-foreground uppercase tracking-wider shrink-0">URL</span>
              <code class="text-xs text-white/70 font-mono text-right break-all">{{ pendingInstall.url }}</code>
            </div>
          </div>

          <!-- Target clients -->
          <div class="mt-4 w-full">
            <p class="text-xs text-muted-foreground uppercase tracking-wider mb-2 text-left">Install to</p>
            <div class="flex flex-wrap gap-1.5">
              <Badge
                v-for="client in detectedClients"
                :key="client.id"
                variant="secondary"
                class="text-xs"
              >
                {{ client.name }}
              </Badge>
            </div>
            <p v-if="detectedClients.length === 0" class="text-xs text-muted-foreground/50">
              No AI clients detected on this machine.
            </p>
          </div>

          <!-- Already installed warning -->
          <div v-if="alreadyInstalled" class="mt-4 w-full rounded-lg bg-amber-500/10 border border-amber-500/20 px-4 py-3">
            <p class="text-xs text-amber-300">
              A server named "{{ pendingInstall.name }}" is already installed. Installing again will update the configuration.
            </p>
          </div>
        </div>

        <DialogFooter class="px-8 pb-8 flex gap-2">
          <Button variant="outline" class="flex-1" @click="dismissInstall">Cancel</Button>
          <Button
            class="flex-1"
            :disabled="detectedClients.length === 0"
            @click="handleInstall"
          >
            Install to {{ detectedClients.length }} {{ detectedClients.length === 1 ? 'Client' : 'Clients' }}
          </Button>
        </DialogFooter>
      </template>

      <!-- Installing State -->
      <template v-if="state === 'installing'">
        <div class="px-8 py-16 flex flex-col items-center text-center">
          <div class="h-10 w-10 rounded-full border-2 border-white/20 border-t-emerald-400 animate-spin mb-6" />
          <p class="text-sm text-muted-foreground">
            Installing <span class="text-white font-medium">{{ pendingInstall?.name }}</span> to all clients...
          </p>
        </div>
      </template>

      <!-- Success State -->
      <template v-if="state === 'success'">
        <div class="px-8 py-12 flex flex-col items-center text-center">
          <div class="h-14 w-14 rounded-full bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center mb-6">
            <svg class="h-7 w-7 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
            </svg>
          </div>
          <h3 class="text-lg font-bold text-white mb-2">Installed Successfully</h3>
          <p class="text-sm text-muted-foreground mb-6">
            <span class="text-white font-medium">{{ pendingInstall?.name }}</span> is now available in {{ installResults.length }} {{ installResults.length === 1 ? 'client' : 'clients' }}.
          </p>

          <!-- Per-client results -->
          <div class="w-full space-y-1.5 mb-6">
            <div
              v-for="result in installResults"
              :key="result.clientId"
              class="flex items-center justify-between rounded-lg bg-white/[0.03] px-4 py-2.5"
            >
              <span class="text-sm text-white/80">{{ getClientName(result.clientId) }}</span>
              <Badge :variant="result.success ? 'default' : 'destructive'" class="text-xs">
                {{ result.success ? 'Installed' : 'Failed' }}
              </Badge>
            </div>
          </div>

          <Button class="w-full" @click="handleOpenChange(false)">Done</Button>
        </div>
      </template>

      <!-- Error State -->
      <template v-if="state === 'error'">
        <div class="px-8 py-12 flex flex-col items-center text-center">
          <div class="h-14 w-14 rounded-full bg-red-500/10 border border-red-500/20 flex items-center justify-center mb-6">
            <svg class="h-7 w-7 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
            </svg>
          </div>
          <h3 class="text-lg font-bold text-white mb-2">Installation Failed</h3>
          <p class="text-sm text-muted-foreground mb-4 whitespace-pre-line">{{ errorMessage }}</p>

          <!-- Per-client results when partial failure -->
          <div v-if="installResults.length > 0" class="w-full space-y-1.5 mb-6">
            <div
              v-for="result in installResults"
              :key="result.clientId"
              class="flex items-center justify-between rounded-lg bg-white/[0.03] px-4 py-2.5"
            >
              <span class="text-sm text-white/80">{{ getClientName(result.clientId) }}</span>
              <Badge :variant="result.success ? 'default' : 'destructive'" class="text-xs">
                {{ result.success ? 'Installed' : 'Failed' }}
              </Badge>
            </div>
          </div>

          <div class="flex gap-2 w-full">
            <Button variant="outline" class="flex-1" @click="handleOpenChange(false)">Close</Button>
            <Button class="flex-1" @click="state = 'confirm'">Try Again</Button>
          </div>
        </div>
      </template>

    </DialogContent>
  </Dialog>
</template>
