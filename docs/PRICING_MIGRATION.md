# Pricing Migration: TCGplayer → Multi-Source (Jan 2026)

## What Changed?

TCGplayer has discontinued its public API. VaultSync has migrated to a **multi-source pricing engine** that is actually *more resilient* than before.

### Before (TCGplayer Only)
- ❌ Single point of failure
- ❌ Dependent on one vendor's data format
- ❌ Limited to Magic, Pokémon, some sports

### Now (Multi-Source)
- ✅ Scryfall for Magic (same data TCGplayer used, but direct)
- ✅ Official PokemonTCG API for Pokémon
- ✅ PriceCharting for sports cards
- ✅ Custom feeds for your own data
- ✅ Automatic fallback if any source is down

## Will My Prices Change?

**For Magic cards:** Scryfall uses the same TCGplayer market data (it aggregates from the same sources). Prices should be **identical or very similar**. Minor differences may appear due to:
- Scryfall updates slower (1-2x per day vs real-time)
- Foil vs non-foil handling
- Regional pricing differences

**For Pokémon:** Official API pricing is used instead of aggregated. May see differences.

**For Sports:** No change (already using PriceCharting).

## Pricing Rules Are Unchanged

Your buylist and markup rules (cash/credit multipliers) remain 100% the same. Only the *market data source* changed, not the *logic*.

## Do I Need to Update My Configuration?

**If using defaults:** No action needed. App automatically routes:
- Magic → Scryfall
- Pokémon → PokemonTCG API
- Sports → PriceCharting

**If using custom feeds:** You can continue using CSV uploads or webhooks. No breaking changes.

## Troubleshooting

**Q: Prices are missing or $0.00**
- A: First sync can take time. Check logs: `[INFO] Price sync progress`
- Wait 5-10 minutes for batch job to complete.

**Q: Prices for Pokémon changed significantly**
- A: API sourcing changed from aggregated (TCGplayer) to official. This is expected.
- Adjust buylist rules if needed (e.g., lower cash multiplier).

**Q: I want to use my own pricing vendor**
- A: Contact support. We now support custom price feeds.

## For Developers

See [PRICING_ARCHITECTURE.md](PRICING_ARCHITECTURE.md) for technical details.
