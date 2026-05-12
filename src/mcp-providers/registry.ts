/**
 * MCP Provider Registry
 *
 * The single source of truth for all marketplace providers visible in
 * the application. Providers are listed in sidebar display order.
 *
 * To register a new provider:
 * 1. Create a folder under `mcp-providers/<provider-id>/`
 * 2. Implement the `IntegratedProvider` or `ExternalProvider` interface
 * 3. Add it to the `providers` array below
 *
 * @example
 * ```ts
 * import { getDefaultProvider, getAllProviders } from '@/mcp-providers/registry'
 *
 * // Get the primary integrated provider (Vinkius)
 * const provider = getDefaultProvider()
 * const listings = await provider.marketplace.fetchListings()
 *
 * // Iterate all providers for sidebar rendering
 * for (const p of getAllProviders()) { ... }
 * ```
 */

import type { McpProvider, IntegratedProvider, ExternalProvider } from './types'
import { vinkiusProvider } from './vinkius'

// ── Registry ────────────────────────────────────────────────────────

const providers: readonly McpProvider[] = [
  // Integrated providers (full API support)
  vinkiusProvider,

  // Community-sourced servers (aggregated from open registries)
  external('community', 'Community', 'https://vinkius.com/discover'),
]

/** Shorthand factory for external (link-only) providers. */
function external(id: string, name: string, siteUrl: string): ExternalProvider {
  return { kind: 'external', id, name, siteUrl }
}

// ── Public API ──────────────────────────────────────────────────────

/** All registered providers in sidebar display order. */
export function getAllProviders(): readonly McpProvider[] {
  return providers
}

/** The default integrated provider (Vinkius Cloud). */
export function getDefaultProvider(): IntegratedProvider {
  return vinkiusProvider
}

/** Lookup a provider by its unique ID. */
export function getProvider(id: string): McpProvider | undefined {
  return providers.find((p) => p.id === id)
}

/** All integrated providers (those with full marketplace API support). */
export function getIntegratedProviders(): IntegratedProvider[] {
  return providers.filter((p): p is IntegratedProvider => p.kind === 'integrated')
}
