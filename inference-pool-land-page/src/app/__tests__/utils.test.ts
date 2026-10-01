/**
 * Unit Tests for Utility Functions
 * Feature: landing-page-documentation
 */

import { 
  getFromCache, 
  setInCache, 
  removeFromCache, 
  clearCache 
} from '@/app/utils/cache';

// Mock sessionStorage
const mockSessionStorage = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => { store[key] = value; },
    removeItem: (key: string) => { delete store[key]; },
    clear: () => { store = {}; },
    get length() { return Object.keys(store).length; },
    key: (i: number) => Object.keys(store)[i] || null
  };
})();

// @ts-expect-error - Mocking sessionStorage
global.sessionStorage = mockSessionStorage;

describe('Cache Utilities', () => {
  beforeEach(() => {
    mockSessionStorage.clear();
  });

  describe('setInCache and getFromCache', () => {
    it('should store and retrieve data', () => {
      const testData = { foo: 'bar', count: 42 };
      
      setInCache('test-key', testData);
      const retrieved = getFromCache<typeof testData>('test-key');
      
      expect(retrieved).toEqual(testData);
    });

    it('should return null for non-existent keys', () => {
      const result = getFromCache('non-existent');
      expect(result).toBeNull();
    });

    it('should handle string data', () => {
      setInCache('string-key', 'hello world');
      const result = getFromCache<string>('string-key');
      expect(result).toBe('hello world');
    });

    it('should handle array data', () => {
      const arr = [1, 2, 3, 4, 5];
      setInCache('array-key', arr);
      const result = getFromCache<number[]>('array-key');
      expect(result).toEqual(arr);
    });

    it('should handle nested objects', () => {
      const nested = {
        level1: {
          level2: {
            level3: 'deep'
          }
        }
      };
      setInCache('nested-key', nested);
      const result = getFromCache<typeof nested>('nested-key');
      expect(result).toEqual(nested);
    });
  });

  describe('removeFromCache', () => {
    it('should remove cached data', () => {
      setInCache('to-remove', { data: 'test' });
      expect(getFromCache('to-remove')).not.toBeNull();
      
      removeFromCache('to-remove');
      expect(getFromCache('to-remove')).toBeNull();
    });

    it('should not throw for non-existent keys', () => {
      expect(() => removeFromCache('non-existent')).not.toThrow();
    });
  });

  describe('clearCache', () => {
    it('should clear all cached data with prefix', () => {
      setInCache('key1', 'value1');
      setInCache('key2', 'value2');
      setInCache('key3', 'value3');
      
      clearCache();
      
      expect(getFromCache('key1')).toBeNull();
      expect(getFromCache('key2')).toBeNull();
      expect(getFromCache('key3')).toBeNull();
    });
  });
});

describe('Performance Properties', () => {
  describe('Property 8: Content Loading Performance', () => {
    it('should cache content for fast retrieval', () => {
      const content = 'x'.repeat(10000); // 10KB of content
      
      const startWrite = performance.now();
      setInCache('large-content', content);
      const writeTime = performance.now() - startWrite;
      
      const startRead = performance.now();
      const retrieved = getFromCache<string>('large-content');
      const readTime = performance.now() - startRead;
      
      expect(retrieved).toBe(content);
      expect(writeTime).toBeLessThan(100); // Should be fast
      expect(readTime).toBeLessThan(50); // Read should be faster
    });

    it('should handle multiple concurrent cache operations', () => {
      const operations = Array.from({ length: 100 }, (_, i) => ({
        key: `key-${i}`,
        value: { id: i, data: `value-${i}` }
      }));

      // Write all
      for (const op of operations) {
        setInCache(op.key, op.value);
      }

      // Read all and verify
      for (const op of operations) {
        const result = getFromCache(op.key);
        expect(result).toEqual(op.value);
      }
    });
  });
});
