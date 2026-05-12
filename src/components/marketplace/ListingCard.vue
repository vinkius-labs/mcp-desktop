<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    listing: {
      id: string
      title: string
      slug: string
      short_description?: string
      cover_image_url?: string | null
      icon_url?: string | null
      categories?: string[]
      category?: string | null
      tools_count?: number
      listing_type?: string
      price_cents?: number
    }
    aspectRatio?: '16/8' | '16/9'
    iconSize?: 'normal' | 'large'
    showPrice?: boolean
  }>(),
  {
    aspectRatio: '16/8',
    iconSize: 'normal',
    showPrice: false,
  }
)

const aspectRatioClass = computed(() =>
  props.aspectRatio === '16/9' ? 'aspect-[16/9]' : 'aspect-[16/8]'
)
const iconSizeClass = computed(() =>
  props.iconSize === 'large' ? 'w-16 h-16' : 'w-14 h-14 lg:w-16 lg:h-16'
)
const textSizeClass = computed(() =>
  props.iconSize === 'large' ? 'text-4xl' : 'text-3xl'
)

const subtitleFallback = computed(() => {
  return humanizeSlug(
    (props.listing.categories && props.listing.categories[0]) ||
      props.listing.category ||
      ''
  )
})

function humanizeSlug(slug: string): string {
  if (!slug) return 'Integration'
  return slug
    .replace(/-/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase())
}

function heroGradient(listing: { id?: string; slug?: string }) {
  const id = listing.id ?? listing.slug ?? ''
  let hash = 0
  for (let i = 0; i < id.length; i++) hash = id.charCodeAt(i) + ((hash << 5) - hash)
  const hue = Math.abs(hash) % 360
  const h2 = (hue + 45) % 360
  return `linear-gradient(145deg, hsl(${hue},30%,20%), hsl(${h2},40%,15%))`
}
</script>

<template>
  <router-link
    :to="`/app-catalog/${listing.slug || listing.id}`"
    class="group flex flex-col rounded-2xl transition-all duration-200 cursor-pointer"
  >
    <!-- Cover -->
    <div
      class="relative flex items-center justify-center overflow-hidden rounded-2xl card-cover-border"
      :class="aspectRatioClass"
      :style="{ background: listing.cover_image_url ? 'transparent' : heroGradient(listing) }"
    >
      <img
        v-if="listing.cover_image_url"
        :src="listing.cover_image_url"
        :alt="listing.title"
        class="w-full h-full object-cover group-hover:scale-[1.04] transition-transform duration-500"
      />
      <img
        v-else-if="listing.icon_url"
        :src="listing.icon_url"
        :alt="listing.title"
        class="rounded-2xl object-contain drop-shadow-lg group-hover:scale-110 transition-transform duration-300"
        :class="iconSizeClass"
      />
      <span
        v-else
        class="font-black text-white/80 select-none"
        :class="textSizeClass"
      >
        {{ (listing.title?.charAt(0) ?? 'M').toUpperCase() }}
      </span>
    </div>

    <!-- Info -->
    <div class="p-3 space-y-0.5">
      <div class="flex items-center justify-between gap-2">
        <h3 class="font-semibold text-sm text-foreground leading-snug line-clamp-1">
          {{ listing.title }}
        </h3>
        <span
          v-if="listing.tools_count"
          class="text-xs text-muted-foreground shrink-0 tabular-nums font-medium"
        >
          {{ listing.tools_count }} tools
        </span>
        <span
          v-else-if="showPrice"
          class="text-xs text-muted-foreground font-medium whitespace-nowrap"
        >
          {{
            listing.listing_type === 'free' || !listing.price_cents
              ? 'Free'
              : `$${(listing.price_cents / 100).toFixed(0)}/mo`
          }}
        </span>
      </div>
      <p class="text-sm text-muted-foreground leading-snug line-clamp-1">
        {{ listing.short_description || subtitleFallback }}
      </p>
    </div>
  </router-link>
</template>

<style scoped>
.card-cover-border {
  border: 1px solid rgba(0, 0, 0, 0.15);
}

.dark .card-cover-border {
  border-color: rgba(255, 255, 255, 0.1);
}
</style>
