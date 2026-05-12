<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Zap, CheckCircle2 } from 'lucide-vue-next'

defineProps<{
  isInstalled: boolean
}>()

const emit = defineEmits<{
  install: []
}>()
</script>

<template>
  <Button
    v-if="isInstalled"
    variant="outline"
    size="sm"
    class="gap-2 shrink-0"
    disabled
  >
    <CheckCircle2 class="w-4 h-4" />
    Installed
  </Button>

  <Button
    v-else
    size="sm"
    class="gap-2 shrink-0"
    @click="emit('install')"
  >
    <Zap class="w-4 h-4" />
    Unlock for AI Agents
  </Button>
</template>
