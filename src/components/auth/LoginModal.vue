<script setup lang="ts">
import { ref, watch } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
} from '@/components/ui/dialog'

const auth = useAuthStore()

type LoginState = 'idle' | 'waiting' | 'error'

const state = ref<LoginState>('idle')
const errorMessage = ref('')

watch(
  () => auth.authenticated,
  (isAuthenticated) => {
    if (isAuthenticated) {
      auth.showLoginModal = false
      state.value = 'idle'
    }
  }
)

async function handleLogin() {
  errorMessage.value = ''
  state.value = 'waiting'

  try {
    await auth.login()
  } catch (e) {
    errorMessage.value = String(e)
    state.value = 'error'
  }
}

function handleOpenChange(open: boolean) {
  auth.showLoginModal = open
  if (!open) {
    setTimeout(() => {
      state.value = 'idle'
      errorMessage.value = ''
    }, 200)
  }
}
</script>

<template>
  <Dialog :open="auth.showLoginModal" @update:open="handleOpenChange">
    <DialogContent class="sm:max-w-[520px] !bg-black border-white/[0.06] p-0 overflow-hidden gap-0" @interact-outside.prevent>

      <!-- Hero area -->
      <div class="relative px-8 pt-12 pb-8 flex flex-col items-center text-center">
        <!-- Ambient glow behind logo -->
        <div class="absolute top-8 left-1/2 -translate-x-1/2 w-32 h-32 bg-white/[0.03] rounded-full blur-3xl pointer-events-none" />

        <!-- Logo -->
        <div class="relative mb-6">
          <img
            src="https://site-assets.vinkius.com/vk/icon-v-black-min.png"
            alt="Vinkius"
            class="h-16 w-16"
          />
        </div>

        <!-- Title -->
        <h2 class="text-2xl font-bold tracking-tight text-white mb-2">
          Vinkius for AI Agents
        </h2>

        <!-- Tagline -->
        <p class="text-sm text-muted-foreground leading-relaxed max-w-[360px]">
          MCPs are the music of AI Agents. We built the player and the premium catalog to power them.
        </p>
      </div>

      <!-- Action area -->
      <div class="px-8 pb-8">

        <!-- Idle -->
        <template v-if="state === 'idle'">
          <Button
            size="lg"
            class="w-full h-11 text-sm font-semibold tracking-wide"
            @click="handleLogin"
            :disabled="auth.loading"
          >
            Sign in securely
          </Button>
          <p class="text-center text-[11px] text-muted-foreground/50 uppercase tracking-widest mt-4">
            No credit card · No commitment
          </p>
        </template>

        <!-- Waiting -->
        <template v-else-if="state === 'waiting'">
          <p class="text-sm text-muted-foreground text-center mb-5">
            Complete the sign-in in your browser.<br />
            This window will update automatically.
          </p>
          <Button variant="outline" class="w-full" @click="handleLogin">Reopen Browser</Button>
        </template>

        <!-- Error -->
        <template v-else-if="state === 'error'">
          <p class="text-sm text-destructive text-center mb-5">{{ errorMessage }}</p>
          <div class="flex gap-2">
            <Button variant="outline" class="flex-1" @click="state = 'idle'">Back</Button>
            <Button class="flex-1" @click="handleLogin">Try Again</Button>
          </div>
        </template>
      </div>

    </DialogContent>
  </Dialog>
</template>
