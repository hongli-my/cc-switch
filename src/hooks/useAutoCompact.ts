import { useEffect, useRef, useState, type RefObject } from "react";

/**
 * Detects whether the container's children overflow the available width
 * and returns a `compact` flag for the AppSwitcher.
 *
 * Uses ResizeObserver on a flex-constrained container. The container
 * must have `flex-1 min-w-0 overflow-hidden` so its width is determined
 * by the parent layout, not its own content — avoiding the oscillation
 * problem when toggling compact mode.
 */
export function useAutoCompact(
  containerRef: RefObject<HTMLDivElement | null>,
  resetKey?: unknown,
): boolean {
  const [compact, setCompact] = useState(false);
  const normalWidthRef = useRef(0);
  const lockUntilRef = useRef(0);
  const firstResetRef = useRef(true);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const ro = new ResizeObserver(() => {
      // During expand animation, ignore resize events to prevent flicker
      if (Date.now() < lockUntilRef.current) return;

      if (!compact) {
        // Overflow detected → switch to compact
        if (el.scrollWidth > el.clientWidth + 1) {
          // Cache only at the overflow edge: when content fits,
          // scrollWidth === clientWidth (DOM spec), so caching unconditionally
          // would pollute normalWidthRef with the container width (e.g. after
          // maximizing), making the expand threshold unreachable.
          normalWidthRef.current = el.scrollWidth;
          setCompact(true);
        }
      } else if (normalWidthRef.current > 0) {
        // In compact mode: only recover to normal if
        // available space >= what normal mode needed
        if (el.clientWidth >= normalWidthRef.current) {
          // Lock out resize events during the expand animation (200ms + 50ms margin)
          lockUntilRef.current = Date.now() + 250;
          setCompact(false);
        }
      }
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, [compact]);

  // When `resetKey` changes (e.g. switching the active app) the toolbar is
  // mid-transition (button group swap via AnimatePresence) and the cached
  // `normalWidthRef` is stale for the new content. Expanding on that stale
  // value is what caused the compact <-> normal oscillation (visible jitter).
  //
  // Lock the ResizeObserver for the transition duration, then run a single
  // safe re-evaluation that only *collapses* to compact when truly
  // overflowing — it never expands on stale data. A legitimate expand still
  // happens later via the ResizeObserver on a real window resize.
  useEffect(() => {
    if (resetKey === undefined || firstResetRef.current) {
      firstResetRef.current = false;
      return;
    }
    // AnimatePresence exit(150ms) + enter(150ms) + margin
    lockUntilRef.current = Date.now() + 400;
    const timer = window.setTimeout(() => {
      const el = containerRef.current;
      if (!el) return;
      if (el.scrollWidth > el.clientWidth + 1) {
        normalWidthRef.current = el.scrollWidth;
        setCompact(true);
      }
    }, 350);
    return () => window.clearTimeout(timer);
  }, [resetKey, containerRef]);

  return compact;
}
