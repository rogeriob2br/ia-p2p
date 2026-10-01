/**
 * useSearch Hook
 * Feature: landing-page-documentation
 * 
 * Provides fuzzy search functionality using Fuse.js
 * for searching through documentation content.
 * 
 * Requirements: 6.1, 6.2, 6.3, 6.4, 6.5
 */

import { useState, useMemo, useCallback, useRef, useEffect } from 'react';
import Fuse from 'fuse.js';
import type { SearchResult } from '@/app/types';
import { flattenNavigation, navigationStructure } from '@/app/content/navigation';
import { getSectionContent, documentationMetadata } from '@/app/content/docs';

interface SearchableItem {
  id: string;
  title: string;
  content: string;
  path: string;
  keywords: string[];
}

interface UseSearchOptions {
  /** Minimum characters to trigger search */
  minChars?: number;
  /** Maximum results to return */
  maxResults?: number;
  /** Debounce delay in ms */
  debounceMs?: number;
}

interface UseSearchReturn {
  /** Current search query */
  query: string;
  /** Set search query */
  setQuery: (query: string) => void;
  /** Search results */
  results: SearchResult[];
  /** Whether search is active (has query) */
  isSearching: boolean;
  /** Currently selected result index */
  selectedIndex: number;
  /** Move selection up */
  selectPrevious: () => void;
  /** Move selection down */
  selectNext: () => void;
  /** Get the currently selected result */
  selectedResult: SearchResult | null;
  /** Reset search state */
  reset: () => void;
  /** Whether the search index is ready */
  isReady: boolean;
}

/**
 * Build searchable index from navigation and content
 */
function buildSearchIndex(): SearchableItem[] {
  const sections = flattenNavigation(navigationStructure);
  
  return sections.map(section => {
    const content = getSectionContent(section.id);
    const metadata = documentationMetadata.sections.find(m => m.id === section.id);
    
    return {
      id: section.id,
      title: section.title,
      content: content,
      path: section.path,
      keywords: metadata?.keywords || []
    };
  });
}

/**
 * Extract a text excerpt around a match
 */
function extractExcerpt(content: string, query: string, maxLength: number = 150): string {
  const lowerContent = content.toLowerCase();
  const lowerQuery = query.toLowerCase();
  const matchIndex = lowerContent.indexOf(lowerQuery);
  
  if (matchIndex === -1) {
    // No direct match, return start of content
    return content.slice(0, maxLength).trim() + (content.length > maxLength ? '...' : '');
  }
  
  // Calculate start position to center the match
  const start = Math.max(0, matchIndex - Math.floor(maxLength / 2));
  const end = Math.min(content.length, start + maxLength);
  
  let excerpt = content.slice(start, end).trim();
  
  // Add ellipsis if needed
  if (start > 0) excerpt = '...' + excerpt;
  if (end < content.length) excerpt = excerpt + '...';
  
  return excerpt;
}

/**
 * Find the matched text in the result
 */
function findMatchedText(item: SearchableItem, query: string): string {
  const lowerQuery = query.toLowerCase();
  
  // Check title first
  if (item.title.toLowerCase().includes(lowerQuery)) {
    return item.title;
  }
  
  // Check keywords
  for (const keyword of item.keywords) {
    if (keyword.toLowerCase().includes(lowerQuery)) {
      return keyword;
    }
  }
  
  // Check content
  const lowerContent = item.content.toLowerCase();
  const matchIndex = lowerContent.indexOf(lowerQuery);
  if (matchIndex !== -1) {
    // Extract the matched word/phrase
    const start = Math.max(0, matchIndex);
    const end = Math.min(item.content.length, matchIndex + query.length + 20);
    return item.content.slice(start, end).split(/\s+/).slice(0, 3).join(' ');
  }
  
  return query;
}

export function useSearch({
  minChars = 2,
  maxResults = 10,
  debounceMs = 150
}: UseSearchOptions = {}): UseSearchReturn {
  const [query, setQueryState] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isReady, setIsReady] = useState(false);
  
  const debounceRef = useRef<NodeJS.Timeout>();
  const fuseRef = useRef<Fuse<SearchableItem>>();

  // Initialize Fuse.js search index
  useEffect(() => {
    const searchIndex = buildSearchIndex();
    
    fuseRef.current = new Fuse(searchIndex, {
      keys: [
        { name: 'title', weight: 0.4 },
        { name: 'keywords', weight: 0.3 },
        { name: 'content', weight: 0.3 }
      ],
      threshold: 0.4,
      includeScore: true,
      includeMatches: true,
      minMatchCharLength: minChars,
      ignoreLocation: true,
      findAllMatches: true
    });
    
    setIsReady(true);
  }, [minChars]);

  // Perform search with debouncing
  const setQuery = useCallback((newQuery: string) => {
    setQueryState(newQuery);
    
    // Clear previous debounce
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    
    // Reset selection
    setSelectedIndex(0);
    
    // Don't search if query is too short
    if (newQuery.length < minChars) {
      setResults([]);
      return;
    }
    
    // Debounce search
    debounceRef.current = setTimeout(() => {
      if (!fuseRef.current) return;
      
      const fuseResults = fuseRef.current.search(newQuery, { limit: maxResults });
      
      const searchResults: SearchResult[] = fuseResults.map(result => ({
        sectionId: result.item.id,
        title: result.item.title,
        excerpt: extractExcerpt(result.item.content, newQuery),
        matchedText: findMatchedText(result.item, newQuery),
        path: result.item.path
      }));
      
      setResults(searchResults);
    }, debounceMs);
  }, [minChars, maxResults, debounceMs]);

  // Keyboard navigation
  const selectPrevious = useCallback(() => {
    setSelectedIndex(prev => (prev > 0 ? prev - 1 : results.length - 1));
  }, [results.length]);

  const selectNext = useCallback(() => {
    setSelectedIndex(prev => (prev < results.length - 1 ? prev + 1 : 0));
  }, [results.length]);

  // Get selected result
  const selectedResult = useMemo(() => {
    return results[selectedIndex] || null;
  }, [results, selectedIndex]);

  // Reset search state
  const reset = useCallback(() => {
    setQueryState('');
    setResults([]);
    setSelectedIndex(0);
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

  return {
    query,
    setQuery,
    results,
    isSearching: query.length >= minChars,
    selectedIndex,
    selectPrevious,
    selectNext,
    selectedResult,
    reset,
    isReady
  };
}

/**
 * Pure function for testing: perform search on items
 */
export function performSearch(
  items: SearchableItem[],
  query: string,
  options: { threshold?: number; maxResults?: number } = {}
): SearchResult[] {
  const { threshold = 0.4, maxResults = 10 } = options;
  
  if (query.length < 2) return [];
  
  const fuse = new Fuse(items, {
    keys: ['title', 'keywords', 'content'],
    threshold,
    includeScore: true
  });
  
  const results = fuse.search(query, { limit: maxResults });
  
  return results.map(result => ({
    sectionId: result.item.id,
    title: result.item.title,
    excerpt: extractExcerpt(result.item.content, query),
    matchedText: findMatchedText(result.item, query),
    path: result.item.path
  }));
}
