<script setup lang="ts">
import { ref, watch, computed, nextTick, onBeforeUnmount } from 'vue'
import { useRoute } from 'vue-router'
import { useMarketplace } from '@/composables/useMarketplace'
import { getDefaultProvider } from '@/mcp-providers/registry'
import { useAuthStore } from '@/stores/auth'
import { useServersStore } from '@/stores/servers'
import { openUrl } from '@tauri-apps/plugin-opener'
import { config } from '@/config/app'
import type { ListingDetail, MarketplaceItem } from '@/mcp-providers/types'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Loader2, Share2 } from 'lucide-vue-next'
import AppDetailHero from '@/components/marketplace/AppDetailHero.vue'
import ShareListingModal from '@/components/marketplace/ShareListingModal.vue'
import ListingCtaButton from '@/components/marketplace/ListingCtaButton.vue'
import AppDetailOverview from '@/components/marketplace/AppDetailOverview.vue'
import PromptExamplesShowcase from '@/components/marketplace/PromptExamplesShowcase.vue'
import HorizontalCardCarousel from '@/components/marketplace/HorizontalCardCarousel.vue'

const route = useRoute()
const { fetchListingDetail } = useMarketplace()
const provider = getDefaultProvider()
const auth = useAuthStore()
const serversStore = useServersStore()

const loading = ref(true)
const listing = ref<ListingDetail | null>(null)
const error = ref<string | null>(null)
const showShareModal = ref(false)
const selectedToolName = ref<string | null>(null)
const isStuck = ref(false)
const stickySentinelRef = ref<HTMLElement | null>(null)
const scrollContainerRef = ref<HTMLElement | null>(null)

let stickyObserver: IntersectionObserver | null = null

function setupStickyObserver() {
  if (!stickySentinelRef.value || !scrollContainerRef.value) return
  stickyObserver?.disconnect()
  stickyObserver = new IntersectionObserver(
    ([entry]) => {
      isStuck.value = !entry.isIntersecting
    },
    { root: scrollContainerRef.value, threshold: 0 },
  )
  stickyObserver.observe(stickySentinelRef.value)
}

watch(listing, async (val) => {
  if (val) {
    await nextTick()
    setupStickyObserver()
  }
})

onBeforeUnmount(() => {
  stickyObserver?.disconnect()
})

function displayUrl(url: string): string {
  return url.replace(/^https?:\/\/(www\.)?/, '').replace(/\/$/, '')
}

// ── CTA State ────────────────────────────────────────────────────────

const isInstalled = computed(() => {
  if (!listing.value) return false
  return serversStore.servers.some(s => s.name === listing.value!.slug)
})

const priceLabel = computed(() => {
  if (!listing.value) return ''
  return listing.value.listing_type === 'free'
    ? 'Free'
    : `$${(listing.value.price_cents / 100).toFixed(0)}/mo`
})

/** Open the Cloud frontend listing page in the user's default browser. */
async function handleCtaClick() {
  if (!listing.value) return

  if (!auth.authenticated) {
    auth.showLoginModal = true
    return
  }

  if (isInstalled.value) return

  const cloudUrl = `${config.appBaseUrl}/app-catalog/${listing.value.id}`
  await openUrl(cloudUrl)
}

// ── URLs ─────────────────────────────────────────────────────────────

function getShareUrl(): string {
  if (!listing.value) return ''
  return `${provider.siteUrl}/apps/${listing.value.slug}-mcp`
}

const askAiUrls = computed(() => {
  if (!listing.value) return { chatgpt: '', claude: '', perplexity: '' }
  const serverUrl = `${provider.siteUrl}/apps/${listing.value.slug}-mcp`
  const prompt = `Tell me about the ${listing.value.title} MCP Server: ${serverUrl}`
  const encoded = encodeURIComponent(prompt)
  return {
    chatgpt: `https://chatgpt.com/?prompt=${encoded}`,
    claude: `https://claude.ai/new?q=${encoded}`,
    perplexity: `https://www.perplexity.ai/?q=${encoded}`,
  }
})

// ── Data fetching ────────────────────────────────────────────────────

watch(
  () => route.params.slug as string,
  async (slug) => {
    if (slug) {
      loading.value = true
      error.value = null
      const result = await fetchListingDetail(slug)
      if (result) {
        listing.value = result
      } else {
        error.value = 'Failed to load app details'
      }
      loading.value = false
    }
  },
  { immediate: true },
)

// ── Markdown renderer ────────────────────────────────────────────────

function renderMarkdown(raw: string): string {
  if (!raw) return ''
  let text = raw.replace(/<[^>]+>/g, '')
  text = text.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
  text = text.replace(/\*([^*]+)\*/g, '<em>$1</em>')
  text = text.replace(
    /`([^`]+)`/g,
    '<code>$1</code>',
  )
  text = text.replace(/^### (.+)$/gm, '<h3>$1</h3>')
  text = text.replace(/^## (.+)$/gm, '<h2>$1</h2>')
  text = text.replace(/^# (.+)$/gm, '<h1>$1</h1>')
  text = text.replace(/^- (.+)$/gm, '<li>$1</li>')
  text = text.replace(
    /(<li>.*<\/li>\n?)+/g,
    (m) => `<ul>${m}</ul>`,
  )
  const lines = text.split(/\n{2,}/)
  text = lines
    .map((p) => {
      const trimmed = p.trim()
      if (!trimmed || trimmed.startsWith('<h') || trimmed.startsWith('<ul')) return trimmed
      return `<p>${trimmed.replace(/\n/g, '<br/>')}</p>`
    })
    .join('\n')
  return text
}
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">

    <!-- Loading -->
    <div v-if="loading" class="flex-1 flex flex-col items-center justify-center">
      <Loader2 class="h-10 w-10 animate-spin text-muted-foreground" />
    </div>

    <!-- Error -->
    <div v-else-if="error" class="flex-1 flex flex-col items-center justify-center">
      <p class="text-destructive font-medium mb-4">{{ error }}</p>
      <router-link to="/dashboard" class="text-muted-foreground hover:underline text-sm">Go back home</router-link>
    </div>

    <!-- Detail -->
    <template v-else-if="listing">

      <!-- Scrollable content -->
      <div ref="scrollContainerRef" class="flex-1 overflow-y-auto overflow-x-hidden">
        <div class="space-y-0 pb-8">

          <!-- Hero (image only) -->
          <AppDetailHero :listing="listing" />

          <!-- Sentinel (detects when sticky bar becomes stuck) -->
          <div ref="stickySentinelRef" class="h-0" />

          <!-- Sticky title bar (persists across full scroll) -->
          <div class="sticky top-0 z-20 bg-background/80 backdrop-blur-md px-8 py-3 border-b border-transparent" style="border-image: linear-gradient(to right, transparent, hsl(var(--border)), transparent) 1;">
            <div class="flex items-center justify-between gap-4">
              <h1
                class="font-black text-foreground tracking-[-0.03em] m-0 truncate transition-all duration-200"
                :class="isStuck ? 'text-base leading-normal' : 'text-4xl leading-[0.95]'"
              >
                {{ listing.title }}
              </h1>

              <!-- Share (natural) vs CTA (stuck) -->
              <Button v-if="!isStuck" variant="outline" size="sm" class="gap-1.5 shrink-0" @click="showShareModal = true">
                <Share2 class="w-3.5 h-3.5" />
                Share
              </Button>

              <ListingCtaButton
                v-else
                :is-installed="isInstalled"
                @install="handleCtaClick"
              />
            </div>
            <!-- URL + tools (hidden when stuck) -->
            <div v-if="!isStuck" class="flex items-center gap-1.5 text-sm font-medium text-muted-foreground flex-wrap mt-2">
              <template v-if="listing.integration?.url">
                <a
                  :href="listing.integration.url"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="hover:text-foreground transition-colors"
                >{{ displayUrl(listing.integration.url) }}</a>
                <span class="text-muted-foreground/40">•</span>
              </template>
              <span>{{ listing.tools?.length || 0 }} tools</span>
              <template v-if="listing.seller">
                <span class="text-muted-foreground/40">•</span>
                <span>{{ listing.seller.display_name }}</span>
              </template>
            </div>
          </div>

          <!-- Badges + Description -->
          <div class="px-8 space-y-5 pt-5">
            <!-- Trust badges -->
            <div class="flex flex-wrap items-center gap-2">
              <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-[6px] text-xs font-semibold text-muted-foreground border border-border">
                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
                Built by Vinkius
              </span>
              <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-[6px] text-xs font-semibold text-muted-foreground border border-border">
                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                GDPR
              </span>
              <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-[6px] text-xs font-semibold text-muted-foreground border border-border">
                {{ listing.server_type === 'skills' ? 'Skills' : 'Tools' }}
              </span>
              <span class="inline-flex items-center px-2.5 py-1 rounded-[6px] text-xs font-semibold text-muted-foreground border border-border">
                {{ priceLabel }}
              </span>
            </div>

            <!-- Short description -->
            <p class="text-lg text-muted-foreground max-w-lg font-light leading-relaxed">
              {{ listing.short_description }}
            </p>
          </div>

          <Separator class="mx-6 my-6" />

          <!-- Ask AI -->
          <div class="px-6">
            <AppDetailOverview
              :listing="listing"
              :ask-ai-urls="askAiUrls"
            />
          </div>

          <!-- Prompt examples -->
          <template v-if="listing.prompt_examples && listing.prompt_examples.length > 0">
            <Separator class="mx-6 my-6" />
            <div class="px-6">
              <Label class="text-muted-foreground text-xs uppercase tracking-wider font-semibold mb-3 block">
                See it in action
              </Label>
              <PromptExamplesShowcase
                :examples="listing.prompt_examples"
                :integration-name="listing.title"
              />
            </div>
          </template>

          <!-- Built-in capabilities (pill badges) -->
          <template v-if="listing.tools && listing.tools.length > 0">
            <Separator class="mx-6 my-6" />
            <div class="px-6 space-y-4">
              <h2 class="text-2xl font-bold text-foreground leading-snug tracking-tight">
                Built-in capabilities
                <span class="text-muted-foreground/40 text-xl font-medium ml-1.5">({{ listing.tools.length }})</span>
              </h2>
              <div class="flex flex-wrap gap-2">
                <button
                  v-for="tool in listing.tools"
                  :key="tool.name"
                  class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-mono font-semibold border transition-colors duration-150 cursor-pointer"
                  :class="selectedToolName === tool.name
                    ? 'border-primary bg-primary/10 text-primary'
                    : 'border-border bg-muted/30 text-muted-foreground hover:border-primary/40 hover:text-foreground'"
                  @click="selectedToolName = selectedToolName === tool.name ? null : tool.name"
                >
                  {{ tool.name }}
                </button>
              </div>

              <!-- Selected tool description -->
              <div
                v-if="selectedToolName && listing.tools.find(t => t.name === selectedToolName)"
                class="rounded-xl border border-border bg-muted/20 p-4 space-y-1.5"
              >
                <h3 class="text-sm font-bold font-mono text-foreground">{{ selectedToolName }}</h3>
                <p
                  v-if="listing.tools.find(t => t.name === selectedToolName)?.description"
                  class="text-sm text-muted-foreground leading-relaxed"
                >
                  {{ listing.tools.find(t => t.name === selectedToolName)?.description }}
                </p>
                <p v-else class="text-sm text-muted-foreground/50 italic">No description available.</p>
              </div>
            </div>
          </template>

          <template v-if="listing.description && listing.description !== listing.short_description">
            <Separator class="mx-6 my-6" />
            <div class="px-6">
              <Label class="text-muted-foreground text-xs uppercase tracking-wider font-semibold mb-3 block">
                About
              </Label>
              <!-- eslint-disable-next-line vue/no-v-html -->
              <div
                class="text-sm leading-[1.8] text-muted-foreground max-w-[720px] [&_strong]:text-foreground [&_strong]:font-semibold [&_code]:text-primary [&_code]:font-mono [&_code]:text-xs [&_code]:bg-accent [&_code]:px-1.5 [&_code]:py-0.5 [&_code]:rounded [&_h1]:text-lg [&_h1]:font-bold [&_h1]:text-foreground [&_h1]:mt-8 [&_h1]:mb-4 [&_h2]:text-base [&_h2]:font-bold [&_h2]:text-foreground [&_h2]:mt-7 [&_h2]:mb-3 [&_h2]:pb-3 [&_h2]:border-b [&_h2]:border-border [&_h3]:text-sm [&_h3]:font-bold [&_h3]:text-foreground/90 [&_h3]:mt-6 [&_h3]:mb-2 [&_p]:mb-4 [&_p]:leading-[1.85] [&_ul]:my-4 [&_ul]:pl-0 [&_ul]:list-none [&_li]:mb-2 [&_li]:pl-4 [&_li]:relative"
                v-html="renderMarkdown(listing.description)"
              />
            </div>
          </template>

          <template v-if="listing.category_listings && listing.category_listings.length">
            <Separator class="mx-6 my-6" />
            <div class="px-6">
              <HorizontalCardCarousel
                title="More in this category"
                :items="(listing.category_listings as unknown as MarketplaceItem[])"
                :card-width="260"
              />
            </div>
          </template>

          <!-- You might also like -->
          <template v-if="listing.related_listings && listing.related_listings.length">
            <Separator class="mx-6 my-6" />
            <div class="px-6">
              <HorizontalCardCarousel
                title="You might also like"
                :items="(listing.related_listings as unknown as MarketplaceItem[])"
                :card-width="260"
              />
            </div>
          </template>

        </div>
      </div>

      <!-- Sticky footer (hidden when title bar is stuck, since CTA is already visible there) -->
      <div v-if="!isStuck" class="flex-none px-6 py-3 bg-background/80 backdrop-blur-md border-t border-border flex items-center shrink-0">
        <div class="w-full flex items-center justify-end">
          <ListingCtaButton
            :is-installed="isInstalled"
            @install="handleCtaClick"
          />
        </div>
      </div>
    </template>

    <!-- Share Modal -->
    <ShareListingModal
      v-if="listing"
      :open="showShareModal"
      :title="listing.title"
      :short-description="listing.short_description"
      :listing-url="getShareUrl()"
      @update:open="showShareModal = $event"
    />
  </div>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
