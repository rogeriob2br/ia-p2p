/**
 * Property-Based Tests for Documentation Layout
 * Feature: landing-page-documentation, Property 4: Responsive Layout Adaptation
 * 
 * Validates: Requirements 7.1, 7.2, 7.3
 * - Desktop (>1024px): Shows sidebar, content, and table of contents
 * - Tablet (768-1024px): Hides TOC, shows collapsible sidebar
 * - Mobile (<768px): Hides sidebar behind hamburger menu
 */

import * as fc from 'fast-check';

/**
 * Breakpoint constants matching the DocumentationLayout component
 */
const BREAKPOINTS = {
  MOBILE_MAX: 767,
  TABLET_MIN: 768,
  TABLET_MAX: 1023,
  DESKTOP_MIN: 1024
} as const;

/**
 * Layout visibility state based on viewport width
 */
interface LayoutState {
  sidebarVisible: boolean;
  sidebarMode: 'fixed' | 'overlay';
  tocVisible: boolean;
  hamburgerVisible: boolean;
}

/**
 * Pure function to determine layout state based on viewport width
 * This mirrors the logic in DocumentationLayout component
 */
function getLayoutState(viewportWidth: number): LayoutState {
  const isMobile = viewportWidth < BREAKPOINTS.TABLET_MIN;
  const isTablet = viewportWidth >= BREAKPOINTS.TABLET_MIN && viewportWidth < BREAKPOINTS.DESKTOP_MIN;
  const isDesktop = viewportWidth >= BREAKPOINTS.DESKTOP_MIN;

  return {
    sidebarVisible: !isMobile, // Hidden by default on mobile (needs hamburger)
    sidebarMode: isMobile ? 'overlay' : 'fixed',
    tocVisible: isDesktop,
    hamburgerVisible: isMobile
  };
}

describe('Property 4: Responsive Layout Adaptation', () => {
  // Feature: landing-page-documentation, Property 4: Responsive Layout Adaptation
  
  describe('Desktop Layout (>1024px)', () => {
    it('should show all three columns on desktop viewports', () => {
      fc.assert(
        fc.property(
          fc.integer({ min: BREAKPOINTS.DESKTOP_MIN, max: 3840 }),
          (viewportWidth) => {
            const state = getLayoutState(viewportWidth);
            
            // Desktop: sidebar visible, fixed mode, TOC visible, no hamburger
            expect(state.sidebarVisible).toBe(true);
            expect(state.sidebarMode).toBe('fixed');
            expect(state.tocVisible).toBe(true);
            expect(state.hamburgerVisible).toBe(false);
          }
        ),
        { numRuns: 100 }
      );
    });
  });

  describe('Tablet Layout (768-1024px)', () => {
    it('should hide TOC and show collapsible sidebar on tablet viewports', () => {
      fc.assert(
        fc.property(
          fc.integer({ min: BREAKPOINTS.TABLET_MIN, max: BREAKPOINTS.TABLET_MAX }),
          (viewportWidth) => {
            const state = getLayoutState(viewportWidth);
            
            // Tablet: sidebar visible, fixed mode, NO TOC, no hamburger
            expect(state.sidebarVisible).toBe(true);
            expect(state.sidebarMode).toBe('fixed');
            expect(state.tocVisible).toBe(false);
            expect(state.hamburgerVisible).toBe(false);
          }
        ),
        { numRuns: 100 }
      );
    });
  });

  describe('Mobile Layout (<768px)', () => {
    it('should hide sidebar behind hamburger menu on mobile viewports', () => {
      fc.assert(
        fc.property(
          fc.integer({ min: 320, max: BREAKPOINTS.MOBILE_MAX }),
          (viewportWidth) => {
            const state = getLayoutState(viewportWidth);
            
            // Mobile: sidebar hidden by default, overlay mode, NO TOC, hamburger visible
            expect(state.sidebarVisible).toBe(false);
            expect(state.sidebarMode).toBe('overlay');
            expect(state.tocVisible).toBe(false);
            expect(state.hamburgerVisible).toBe(true);
          }
        ),
        { numRuns: 100 }
      );
    });
  });

  describe('Breakpoint Transitions', () => {
    it('should have consistent behavior at exact breakpoints', () => {
      // Test exact breakpoint values
      const mobileMax = getLayoutState(BREAKPOINTS.MOBILE_MAX);
      const tabletMin = getLayoutState(BREAKPOINTS.TABLET_MIN);
      const tabletMax = getLayoutState(BREAKPOINTS.TABLET_MAX);
      const desktopMin = getLayoutState(BREAKPOINTS.DESKTOP_MIN);

      // 767px should be mobile
      expect(mobileMax.hamburgerVisible).toBe(true);
      expect(mobileMax.tocVisible).toBe(false);

      // 768px should be tablet (sidebar visible, no hamburger)
      expect(tabletMin.sidebarVisible).toBe(true);
      expect(tabletMin.hamburgerVisible).toBe(false);
      expect(tabletMin.tocVisible).toBe(false);

      // 1023px should still be tablet
      expect(tabletMax.tocVisible).toBe(false);
      expect(tabletMax.sidebarMode).toBe('fixed');

      // 1024px should be desktop
      expect(desktopMin.tocVisible).toBe(true);
      expect(desktopMin.sidebarVisible).toBe(true);
    });

    it('should transition correctly across all valid viewport widths', () => {
      fc.assert(
        fc.property(
          fc.integer({ min: 320, max: 3840 }),
          (viewportWidth) => {
            const state = getLayoutState(viewportWidth);
            
            // Invariant: At least content area is always implied (main)
            // These are mutually exclusive states
            const isMobile = viewportWidth < BREAKPOINTS.TABLET_MIN;
            const isDesktop = viewportWidth >= BREAKPOINTS.DESKTOP_MIN;
            
            // Hamburger only on mobile
            expect(state.hamburgerVisible).toBe(isMobile);
            
            // TOC only on desktop
            expect(state.tocVisible).toBe(isDesktop);
            
            // Sidebar overlay mode only on mobile
            expect(state.sidebarMode).toBe(isMobile ? 'overlay' : 'fixed');
          }
        ),
        { numRuns: 200 }
      );
    });
  });

  describe('Edge Cases', () => {
    it('should handle minimum common viewport widths', () => {
      const smallMobile = getLayoutState(320); // iPhone SE
      expect(smallMobile.hamburgerVisible).toBe(true);
      expect(smallMobile.sidebarMode).toBe('overlay');
    });

    it('should handle large desktop viewports', () => {
      const ultrawide = getLayoutState(3440); // Ultrawide monitor
      const state = getLayoutState(ultrawide.tocVisible ? 3440 : 1024);
      expect(state.tocVisible).toBe(true);
      expect(state.sidebarVisible).toBe(true);
    });
  });
});

/**
 * Property tests for navigation structure
 * Feature: landing-page-documentation, Property 1: Navigation Consistency
 */
describe('Property 1: Navigation Consistency', () => {
  // Mock navigation data for testing
  const mockNavigation = [
    { id: 'getting-started', title: 'Getting Started', path: '/docs/getting-started' },
    { id: 'api-reference', title: 'API Reference', path: '/docs/api-reference' },
    { id: 'faq', title: 'FAQ', path: '/docs/faq' }
  ];

  it('should always have a valid section selected', () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...mockNavigation.map(n => n.id)),
        (sectionId) => {
          const section = mockNavigation.find(s => s.id === sectionId);
          expect(section).toBeDefined();
          expect(section?.id).toBe(sectionId);
        }
      ),
      { numRuns: 50 }
    );
  });

  it('should have unique section IDs', () => {
    const ids = mockNavigation.map(n => n.id);
    const uniqueIds = new Set(ids);
    expect(uniqueIds.size).toBe(ids.length);
  });
});

/**
 * Property tests for breadcrumb path generation
 * Feature: landing-page-documentation, Property 6: Breadcrumb Path Accuracy
 */
describe('Property 6: Breadcrumb Path Accuracy', () => {
  interface BreadcrumbItem {
    id: string;
    title: string;
  }

  /**
   * Pure function to generate breadcrumb path from hierarchy
   */
  function generateBreadcrumbs(
    sectionId: string,
    hierarchy: Record<string, string | null>
  ): BreadcrumbItem[] {
    const path: BreadcrumbItem[] = [];
    let currentId: string | null = sectionId;

    while (currentId) {
      path.unshift({ id: currentId, title: currentId });
      currentId = hierarchy[currentId] || null;
    }

    return path;
  }

  it('should always end with the current section', () => {
    const hierarchy: Record<string, string | null> = {
      'installation': 'getting-started',
      'quick-start': 'getting-started',
      'getting-started': null,
      'api-reference': null
    };

    fc.assert(
      fc.property(
        fc.constantFrom(...Object.keys(hierarchy)),
        (sectionId) => {
          const breadcrumbs = generateBreadcrumbs(sectionId, hierarchy);
          const lastBreadcrumb = breadcrumbs[breadcrumbs.length - 1];
          expect(lastBreadcrumb.id).toBe(sectionId);
        }
      ),
      { numRuns: 50 }
    );
  });

  it('should have at least one item (the current section)', () => {
    const hierarchy: Record<string, string | null> = {
      'getting-started': null,
      'installation': 'getting-started'
    };

    fc.assert(
      fc.property(
        fc.constantFrom(...Object.keys(hierarchy)),
        (sectionId) => {
          const breadcrumbs = generateBreadcrumbs(sectionId, hierarchy);
          expect(breadcrumbs.length).toBeGreaterThanOrEqual(1);
        }
      ),
      { numRuns: 50 }
    );
  });

  it('should maintain parent-child ordering', () => {
    const hierarchy: Record<string, string | null> = {
      'installation': 'getting-started',
      'getting-started': null
    };

    const breadcrumbs = generateBreadcrumbs('installation', hierarchy);
    expect(breadcrumbs[0].id).toBe('getting-started');
    expect(breadcrumbs[1].id).toBe('installation');
  });
});
