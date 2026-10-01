/**
 * Content Caching Utilities
 * Feature: landing-page-documentation
 * 
 * Caches documentation content in sessionStorage for performance.
 */

const CACHE_PREFIX = 'inferencepool_docs_';
const CACHE_VERSION = '1.0';
const CACHE_EXPIRY = 30 * 60 * 1000; // 30 minutes

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  version: string;
}

/**
 * Get cached data
 */
export function getFromCache<T>(key: string): T | null {
  try {
    const cached = sessionStorage.getItem(`${CACHE_PREFIX}${key}`);
    if (!cached) return null;

    const entry: CacheEntry<T> = JSON.parse(cached);

    // Check version
    if (entry.version !== CACHE_VERSION) {
      removeFromCache(key);
      return null;
    }

    // Check expiry
    if (Date.now() - entry.timestamp > CACHE_EXPIRY) {
      removeFromCache(key);
      return null;
    }

    return entry.data;
  } catch {
    return null;
  }
}

/**
 * Set cached data
 */
export function setInCache<T>(key: string, data: T): void {
  try {
    const entry: CacheEntry<T> = {
      data,
      timestamp: Date.now(),
      version: CACHE_VERSION
    };

    sessionStorage.setItem(`${CACHE_PREFIX}${key}`, JSON.stringify(entry));
  } catch (e) {
    // Handle quota exceeded by clearing old entries
    if (e instanceof Error && e.name === 'QuotaExceededError') {
      clearExpiredCache();
      try {
        const entry: CacheEntry<T> = {
          data,
          timestamp: Date.now(),
          version: CACHE_VERSION
        };
        sessionStorage.setItem(`${CACHE_PREFIX}${key}`, JSON.stringify(entry));
      } catch {
        // Give up if still failing
      }
    }
  }
}

/**
 * Remove cached data
 */
export function removeFromCache(key: string): void {
  try {
    sessionStorage.removeItem(`${CACHE_PREFIX}${key}`);
  } catch {
    // Ignore errors
  }
}

/**
 * Clear all cached documentation data
 */
export function clearCache(): void {
  try {
    const keysToRemove: string[] = [];
    
    for (let i = 0; i < sessionStorage.length; i++) {
      const key = sessionStorage.key(i);
      if (key?.startsWith(CACHE_PREFIX)) {
        keysToRemove.push(key);
      }
    }
    
    keysToRemove.forEach(key => sessionStorage.removeItem(key));
  } catch {
    // Ignore errors
  }
}

/**
 * Clear expired cache entries
 */
export function clearExpiredCache(): void {
  try {
    const keysToRemove: string[] = [];
    
    for (let i = 0; i < sessionStorage.length; i++) {
      const key = sessionStorage.key(i);
      if (key?.startsWith(CACHE_PREFIX)) {
        const cached = sessionStorage.getItem(key);
        if (cached) {
          try {
            const entry = JSON.parse(cached);
            if (
              entry.version !== CACHE_VERSION ||
              Date.now() - entry.timestamp > CACHE_EXPIRY
            ) {
              keysToRemove.push(key);
            }
          } catch {
            keysToRemove.push(key);
          }
        }
      }
    }
    
    keysToRemove.forEach(key => sessionStorage.removeItem(key));
  } catch {
    // Ignore errors
  }
}

/**
 * Get cache statistics
 */
export function getCacheStats(): { entries: number; size: number } {
  try {
    let entries = 0;
    let size = 0;
    
    for (let i = 0; i < sessionStorage.length; i++) {
      const key = sessionStorage.key(i);
      if (key?.startsWith(CACHE_PREFIX)) {
        entries++;
        const value = sessionStorage.getItem(key);
        if (value) {
          size += value.length * 2; // UTF-16 encoding
        }
      }
    }
    
    return { entries, size };
  } catch {
    return { entries: 0, size: 0 };
  }
}
