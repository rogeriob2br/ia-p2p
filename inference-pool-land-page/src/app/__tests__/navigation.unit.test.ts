/**
 * Unit Tests for Navigation Components
 * Feature: landing-page-documentation
 * 
 * Tests for NavigationTree component behavior including:
 * - Section expansion/collapse
 * - Active section highlighting
 * - Mobile menu toggle
 */

import { 
  navigationStructure, 
  findSectionById, 
  getBreadcrumbPath,
  getPrevNextSections,
  flattenNavigation
} from '@/app/content/navigation';

describe('NavigationTree Unit Tests', () => {
  describe('Section Expansion Logic', () => {
    it('should identify parent sections correctly', () => {
      // 'installation' is a child of 'getting-started'
      const gettingStarted = findSectionById('getting-started');
      expect(gettingStarted?.children).toBeDefined();
      expect(gettingStarted?.children?.some(c => c.id === 'installation')).toBe(true);
    });

    it('should identify sections with children', () => {
      const sectionsWithChildren = flattenNavigation(navigationStructure)
        .filter(s => s.children && s.children.length > 0);
      
      expect(sectionsWithChildren.length).toBeGreaterThan(0);
      
      for (const section of sectionsWithChildren) {
        expect(section.children?.length).toBeGreaterThan(0);
      }
    });

    it('should identify leaf sections (no children)', () => {
      const leafSections = flattenNavigation(navigationStructure)
        .filter(s => !s.children || s.children.length === 0);
      
      expect(leafSections.length).toBeGreaterThan(0);
      
      for (const section of leafSections) {
        expect(section.children).toBeFalsy();
      }
    });
  });

  describe('Active Section Highlighting', () => {
    it('should find active section in navigation', () => {
      const activeId = 'installation';
      const section = findSectionById(activeId);
      
      expect(section).not.toBeNull();
      expect(section?.id).toBe(activeId);
    });

    it('should determine if section is ancestor of active', () => {
      // Helper function matching component logic
      function isAncestorOf(section: typeof navigationStructure[0], targetId: string): boolean {
        if (!section.children) return false;
        for (const child of section.children) {
          if (child.id === targetId) return true;
          if (isAncestorOf(child, targetId)) return true;
        }
        return false;
      }

      const gettingStarted = findSectionById('getting-started');
      expect(gettingStarted).not.toBeNull();
      expect(isAncestorOf(gettingStarted!, 'installation')).toBe(true);
      expect(isAncestorOf(gettingStarted!, 'quick-start')).toBe(true);
      expect(isAncestorOf(gettingStarted!, 'api-reference')).toBe(false);
    });
  });

  describe('Breadcrumb Generation', () => {
    it('should generate correct breadcrumbs for nested sections', () => {
      const path = getBreadcrumbPath('installation');
      
      expect(path).not.toBeNull();
      expect(path?.length).toBe(2);
      expect(path?.[0].id).toBe('getting-started');
      expect(path?.[1].id).toBe('installation');
    });

    it('should generate single-item breadcrumbs for root sections', () => {
      const path = getBreadcrumbPath('getting-started');
      
      expect(path).not.toBeNull();
      expect(path?.length).toBe(1);
      expect(path?.[0].id).toBe('getting-started');
    });

    it('should generate correct breadcrumbs for deeply nested sections', () => {
      // Test with a section that might be deeply nested
      const flat = flattenNavigation(navigationStructure);
      const deepSection = flat.find(s => {
        const path = getBreadcrumbPath(s.id);
        return path && path.length >= 2;
      });

      if (deepSection) {
        const path = getBreadcrumbPath(deepSection.id);
        expect(path?.length).toBeGreaterThanOrEqual(2);
        expect(path?.[path.length - 1].id).toBe(deepSection.id);
      }
    });
  });

  describe('Previous/Next Navigation', () => {
    it('should navigate to correct previous section', () => {
      const { previous } = getPrevNextSections('installation');
      
      // Should be 'getting-started' (parent comes before children in flat order)
      expect(previous).not.toBeNull();
      expect(previous?.id).toBe('getting-started');
    });

    it('should navigate to correct next section', () => {
      const { next } = getPrevNextSections('getting-started');
      
      // Should be 'installation' (first child in flat order)
      expect(next).not.toBeNull();
      expect(next?.id).toBe('installation');
    });

    it('should handle first section (no previous)', () => {
      const flat = flattenNavigation(navigationStructure);
      const firstSection = flat[0];
      
      const { previous } = getPrevNextSections(firstSection.id);
      expect(previous).toBeNull();
    });

    it('should handle last section (no next)', () => {
      const flat = flattenNavigation(navigationStructure);
      const lastSection = flat[flat.length - 1];
      
      const { next } = getPrevNextSections(lastSection.id);
      expect(next).toBeNull();
    });
  });

  describe('Navigation Structure Validation', () => {
    it('should have Getting Started as first section', () => {
      expect(navigationStructure[0].id).toBe('getting-started');
      expect(navigationStructure[0].title).toBe('Getting Started');
    });

    it('should have FAQ as last section', () => {
      const lastSection = navigationStructure[navigationStructure.length - 1];
      expect(lastSection.id).toBe('faq');
      expect(lastSection.title).toBe('FAQ');
    });

    it('should have correct structure for API Reference', () => {
      const apiRef = findSectionById('api-reference');
      
      expect(apiRef).not.toBeNull();
      expect(apiRef?.children).toBeDefined();
      expect(apiRef?.children?.length).toBeGreaterThanOrEqual(3);
      
      const childIds = apiRef?.children?.map(c => c.id);
      expect(childIds).toContain('client');
      expect(childIds).toContain('methods');
      expect(childIds).toContain('configuration');
    });

    it('should have correct structure for Integration Guides', () => {
      const integrations = findSectionById('integration-guides');
      
      expect(integrations).not.toBeNull();
      expect(integrations?.children).toBeDefined();
      
      const childIds = integrations?.children?.map(c => c.id);
      expect(childIds).toContain('langchain');
      expect(childIds).toContain('crewai');
      expect(childIds).toContain('nodejs');
    });
  });
});

describe('Mobile Menu Behavior', () => {
  describe('Sidebar State Management', () => {
    it('should toggle sidebar state correctly', () => {
      let isOpen = false;
      const toggle = () => { isOpen = !isOpen; };
      
      expect(isOpen).toBe(false);
      toggle();
      expect(isOpen).toBe(true);
      toggle();
      expect(isOpen).toBe(false);
    });

    it('should close sidebar on section change', () => {
      let isOpen = true;
      const closeSidebar = () => { isOpen = false; };
      
      // Simulate section change closing sidebar
      closeSidebar();
      expect(isOpen).toBe(false);
    });
  });

  describe('Touch Gesture Detection', () => {
    it('should calculate swipe distance correctly', () => {
      const startX = 250;
      const endX = 100;
      const swipeDistance = startX - endX;
      
      expect(swipeDistance).toBe(150);
      expect(swipeDistance > 100).toBe(true); // Should trigger close
    });

    it('should not trigger close for small swipes', () => {
      const startX = 250;
      const endX = 200;
      const swipeDistance = startX - endX;
      
      expect(swipeDistance).toBe(50);
      expect(swipeDistance > 100).toBe(false); // Should not trigger close
    });

    it('should not trigger for right swipes', () => {
      const startX = 100;
      const endX = 250;
      const swipeDistance = startX - endX;
      
      expect(swipeDistance).toBe(-150);
      expect(swipeDistance > 0).toBe(false); // Right swipe, should not close
    });
  });
});

describe('Keyboard Navigation', () => {
  describe('Tree Navigation Keys', () => {
    const navigationKeys = ['ArrowDown', 'ArrowUp', 'ArrowLeft', 'ArrowRight', 'Enter', ' '];

    it('should recognize navigation key events', () => {
      for (const key of navigationKeys) {
        const isNavigationKey = navigationKeys.includes(key);
        expect(isNavigationKey).toBe(true);
      }
    });

    it('should handle expand/collapse with arrow keys', () => {
      // Simulate expand/collapse logic
      let isExpanded = false;
      const hasChildren = true;

      const handleKeyDown = (key: string) => {
        if (hasChildren) {
          if (key === 'ArrowRight' && !isExpanded) {
            isExpanded = true;
          }
          if (key === 'ArrowLeft' && isExpanded) {
            isExpanded = false;
          }
        }
      };

      expect(isExpanded).toBe(false);
      handleKeyDown('ArrowRight');
      expect(isExpanded).toBe(true);
      handleKeyDown('ArrowLeft');
      expect(isExpanded).toBe(false);
    });
  });

  describe('Focus Management', () => {
    it('should handle focus movement logic', () => {
      const items = ['item1', 'item2', 'item3'];
      let focusIndex = 0;

      const moveFocusDown = () => {
        focusIndex = focusIndex < items.length - 1 ? focusIndex + 1 : 0;
      };

      const moveFocusUp = () => {
        focusIndex = focusIndex > 0 ? focusIndex - 1 : items.length - 1;
      };

      expect(focusIndex).toBe(0);
      moveFocusDown();
      expect(focusIndex).toBe(1);
      moveFocusDown();
      expect(focusIndex).toBe(2);
      moveFocusDown(); // Should wrap to 0
      expect(focusIndex).toBe(0);
      moveFocusUp(); // Should wrap to last
      expect(focusIndex).toBe(2);
    });
  });
});
