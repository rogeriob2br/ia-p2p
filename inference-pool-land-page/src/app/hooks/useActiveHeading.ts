/**
 * useActiveHeading Hook
 * Feature: landing-page-documentation
 * 
 * Tracks which heading is currently visible in the viewport
 * and updates the active heading for Table of Contents highlighting.
 * 
 * Requirements: 1.3
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import type { TOCHeading } from '@/app/types';

interface UseActiveHeadingOptions {
  /** Headings to track */
  headings: TOCHeading[];
  /** Offset from top of viewport (default: 100) */
  offset?: number;
  /** Root margin for intersection observer */
  rootMargin?: string;
}

interface UseActiveHeadingReturn {
  /** Currently active heading ID */
  activeHeading: string;
  /** Manually set the active heading */
  setActiveHeading: (id: string) => void;
  /** Scroll to a specific heading */
  scrollToHeading: (id: string) => void;
}

/**
 * Hook to track which heading is currently visible in the viewport
 */
export function useActiveHeading({
  headings,
  offset = 100,
  rootMargin = '-80px 0px -80% 0px'
}: UseActiveHeadingOptions): UseActiveHeadingReturn {
  const [activeHeading, setActiveHeading] = useState<string>('');
  const isScrollingRef = useRef(false);
  const scrollTimeoutRef = useRef<NodeJS.Timeout>();

  // Use Intersection Observer for better performance
  useEffect(() => {
    if (headings.length === 0) return;

    const headingElements = headings
      .map(h => document.getElementById(h.id))
      .filter((el): el is HTMLElement => el !== null);

    if (headingElements.length === 0) return;

    const observer = new IntersectionObserver(
      (entries) => {
        // Don't update while manually scrolling
        if (isScrollingRef.current) return;

        // Find the first visible heading
        const visibleEntries = entries.filter(entry => entry.isIntersecting);
        
        if (visibleEntries.length > 0) {
          // Get the topmost visible heading
          const topEntry = visibleEntries.reduce((prev, curr) => {
            const prevRect = prev.boundingClientRect;
            const currRect = curr.boundingClientRect;
            return prevRect.top < currRect.top ? prev : curr;
          });
          
          setActiveHeading(topEntry.target.id);
        }
      },
      {
        rootMargin,
        threshold: [0, 0.5, 1]
      }
    );

    headingElements.forEach(el => observer.observe(el));

    return () => {
      headingElements.forEach(el => observer.unobserve(el));
      observer.disconnect();
    };
  }, [headings, rootMargin]);

  // Fallback scroll listener for browsers without good IO support
  useEffect(() => {
    if (headings.length === 0) return;

    const handleScroll = () => {
      if (isScrollingRef.current) return;

      const scrollPosition = window.scrollY + offset;

      // Find the heading that is currently in view
      for (let i = headings.length - 1; i >= 0; i--) {
        const heading = headings[i];
        const element = document.getElementById(heading.id);
        
        if (element && element.offsetTop <= scrollPosition) {
          setActiveHeading(heading.id);
          return;
        }
      }

      // Default to first heading if none found
      if (headings.length > 0) {
        setActiveHeading(headings[0].id);
      }
    };

    // Throttle scroll handler
    let ticking = false;
    const throttledScroll = () => {
      if (!ticking) {
        requestAnimationFrame(() => {
          handleScroll();
          ticking = false;
        });
        ticking = true;
      }
    };

    window.addEventListener('scroll', throttledScroll, { passive: true });
    
    // Initial check
    handleScroll();

    return () => {
      window.removeEventListener('scroll', throttledScroll);
    };
  }, [headings, offset]);

  // Scroll to a specific heading
  const scrollToHeading = useCallback((id: string) => {
    const element = document.getElementById(id);
    if (!element) return;

    // Set flag to prevent observer from overriding
    isScrollingRef.current = true;
    setActiveHeading(id);

    // Clear any existing timeout
    if (scrollTimeoutRef.current) {
      clearTimeout(scrollTimeoutRef.current);
    }

    // Smooth scroll to element
    element.scrollIntoView({ 
      behavior: 'smooth', 
      block: 'start' 
    });

    // Reset flag after scroll animation completes
    scrollTimeoutRef.current = setTimeout(() => {
      isScrollingRef.current = false;
    }, 1000);
  }, []);

  // Cleanup
  useEffect(() => {
    return () => {
      if (scrollTimeoutRef.current) {
        clearTimeout(scrollTimeoutRef.current);
      }
    };
  }, []);

  return {
    activeHeading,
    setActiveHeading,
    scrollToHeading
  };
}

/**
 * Pure function to determine which heading should be active
 * based on scroll position. Used for testing.
 */
export function calculateActiveHeading(
  headings: { id: string; offsetTop: number }[],
  scrollPosition: number,
  offset: number = 100
): string {
  if (headings.length === 0) return '';

  const adjustedPosition = scrollPosition + offset;

  for (let i = headings.length - 1; i >= 0; i--) {
    if (headings[i].offsetTop <= adjustedPosition) {
      return headings[i].id;
    }
  }

  return headings[0].id;
}
