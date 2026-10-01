/**
 * useScrollSpy Hook
 * Feature: landing-page-documentation
 * 
 * Tracks which section is currently visible in the viewport
 * and updates the active section accordingly.
 */

import { useState, useEffect, useCallback, useRef } from 'react';

interface UseScrollSpyOptions {
  /** IDs of sections to observe */
  sectionIds: string[];
  /** Offset from top of viewport (default: 100) */
  offset?: number;
  /** Throttle delay in ms (default: 100) */
  throttleMs?: number;
}

interface UseScrollSpyReturn {
  /** Currently active section ID */
  activeSection: string;
  /** Manually set the active section */
  setActiveSection: (id: string) => void;
  /** Scroll to a specific section */
  scrollToSection: (id: string) => void;
}

/**
 * Hook to track which section is currently in view
 */
export function useScrollSpy({
  sectionIds,
  offset = 100,
  throttleMs = 100
}: UseScrollSpyOptions): UseScrollSpyReturn {
  const [activeSection, setActiveSection] = useState<string>(sectionIds[0] || '');
  const isManualScrolling = useRef(false);
  const manualScrollTimeout = useRef<NodeJS.Timeout>();

  // Throttle function
  const throttle = useCallback(<T extends (...args: unknown[]) => void>(
    func: T,
    limit: number
  ): T => {
    let inThrottle = false;
    return ((...args: unknown[]) => {
      if (!inThrottle) {
        func(...args);
        inThrottle = true;
        setTimeout(() => (inThrottle = false), limit);
      }
    }) as T;
  }, []);

  // Find which section is currently in view
  const updateActiveSection = useCallback(() => {
    if (isManualScrolling.current) return;

    const scrollPosition = window.scrollY + offset;

    // Find the section that is currently in view
    for (let i = sectionIds.length - 1; i >= 0; i--) {
      const sectionId = sectionIds[i];
      const element = document.getElementById(sectionId);
      
      if (element) {
        const { offsetTop } = element;
        if (scrollPosition >= offsetTop) {
          setActiveSection(sectionId);
          return;
        }
      }
    }

    // Default to first section if none found
    if (sectionIds.length > 0) {
      setActiveSection(sectionIds[0]);
    }
  }, [sectionIds, offset]);

  // Scroll to a specific section
  const scrollToSection = useCallback((id: string) => {
    const element = document.getElementById(id);
    if (element) {
      // Set manual scrolling flag to prevent scroll spy from overriding
      isManualScrolling.current = true;
      setActiveSection(id);

      // Clear any existing timeout
      if (manualScrollTimeout.current) {
        clearTimeout(manualScrollTimeout.current);
      }

      // Scroll to element
      element.scrollIntoView({ behavior: 'smooth', block: 'start' });

      // Reset manual scrolling flag after animation completes
      manualScrollTimeout.current = setTimeout(() => {
        isManualScrolling.current = false;
      }, 1000);
    }
  }, []);

  // Set up scroll listener
  useEffect(() => {
    const throttledUpdate = throttle(updateActiveSection, throttleMs);
    
    window.addEventListener('scroll', throttledUpdate, { passive: true });
    
    // Initial check
    updateActiveSection();

    return () => {
      window.removeEventListener('scroll', throttledUpdate);
      if (manualScrollTimeout.current) {
        clearTimeout(manualScrollTimeout.current);
      }
    };
  }, [updateActiveSection, throttle, throttleMs]);

  return {
    activeSection,
    setActiveSection,
    scrollToSection
  };
}
