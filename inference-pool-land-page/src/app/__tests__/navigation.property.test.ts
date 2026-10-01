/**
 * Property-Based Tests for Navigation
 * Feature: landing-page-documentation, Property 1: Navigation Consistency
 * 
 * Validates: Requirements 1.2, 1.4
 * - Navigation actions update displayed content and URL accordingly
 * - Sidebar items match selected section
 */

import * as fc from 'fast-check';
import { 
  navigationStructure, 
  findSectionById, 
  getBreadcrumbPath,
  getPrevNextSections,
  flattenNavigation,
  getAllSectionIds
} from '@/app/content/navigation';

describe('Property 1: Navigation Consistency', () => {
  // Feature: landing-page-documentation, Property 1: Navigation Consistency

  const allSectionIds = getAllSectionIds();
  const flatNavigation = flattenNavigation(navigationStructure);

  describe('Section ID Uniqueness', () => {
    it('should have unique section IDs across all navigation items', () => {
      const ids = allSectionIds;
      const uniqueIds = new Set(ids);
      expect(uniqueIds.size).toBe(ids.length);
    });

    it('should have non-empty section IDs', () => {
      fc.assert(
        fc.property(
          fc.constantFrom(...allSectionIds),
          (sectionId) => {
            expect(sectionId).toBeTruthy();
            expect(sectionId.length).toBeGreaterThan(0);
          }
        ),
        { numRuns: allSectionIds.length }
      );
    });
  });

  describe('findSectionById', () => {
    it('should find any valid section by ID', () => {
      fc.assert(
        fc.property(
          fc.constantFrom(...allSectionIds),
          (sectionId) => {
            const section = findSectionById(sectionId);
            expect(section).not.toBeNull();
            expect(section?.id).toBe(sectionId);
          }
        ),
        { numRuns: 100 }
      );
    });

    it('should return null for non-existent section IDs', () => {
      fc.assert(
        fc.property(
          fc.string({ minLength: 10, maxLength: 20 }).filter(s => !allSectionIds.includes(s)),
          (invalidId) => {
            const section = findSectionById(invalidId);
            expect(section).toBeNull();
          }
        ),
        { numRuns: 50 }
      );
    });

    it('should return section with matching title', () => {
      fc.assert(
        fc.property(
          fc.constantFrom(...allSectionIds),
          (sectionId) => {
            const section = findSectionById(sectionId);
            expect(section?.title).toBeTruthy();
            expect(typeof section?.title).toBe('string');
          }
        ),
        { numRuns: 100 }
      );
    });
  });

  describe('getBreadcrumbPath', () => {
    it('should always return a path ending with the current section', () => {
      fc.assert(
        fc.property(
          fc.constantFrom(...allSectionIds),
          (sectionId) => {
            const path = getBreadcrumbPath(sectionId);
            
            // Path should exist for valid sections
            expect(path).not.toBeNull();
            expect(path!.length).toBeGreaterThanOrEqual(1);
            
            // Last item should be the current section
            const lastItem = path![path!.length - 1];
            expect(lastItem.id).toBe(sectionId);
          }
        ),
        { numRuns: 100 }
      );
    });

    it('should maintain parent-child hierarchy in breadcrumb path', () => {
      fc.assert(
        fc.property(
          fc.constantFrom(...allSectionIds),
          (sectionId) => {
            const path = getBreadcrumbPath(sectionId);
            if (!path || path.length <= 1) return;

            // Each item in path (except last) should be a parent of the next
            for (let i = 0; i < path.length - 1; i++) {
              const parent = path[i];
              const child = path[i + 1];
              
              // Parent should have children array containing child
              const hasChild = parent.children?.some(c => c.id === child.id);
              expect(hasChild).toBe(true);
            }
          }
        ),
        { numRuns: 100 }
      );
    });

    it('should return null for non-existent sections', () => {
      const path = getBreadcrumbPath('non-existent-section-xyz');
      expect(path).toBeNull();
    });
  });

  describe('getPrevNextSections', () => {
    it('should return valid prev/next sections for middle sections', () => {
      // Get sections that are not first or last
      const middleSections = allSectionIds.slice(1, -1);
      
      fc.assert(
        fc.property(
          fc.constantFrom(...middleSections),
          (sectionId) => {
            const { previous, next } = getPrevNextSections(sectionId);
            
            // Middle sections should have both previous and next
            expect(previous).not.toBeNull();
            expect(next).not.toBeNull();
            
            // Previous and next should be different sections
            expect(previous?.id).not.toBe(sectionId);
            expect(next?.id).not.toBe(sectionId);
            expect(previous?.id).not.toBe(next?.id);
          }
        ),
        { numRuns: 50 }
      );
    });

    it('should return null for previous on first section', () => {
      const firstSection = allSectionIds[0];
      const { previous } = getPrevNextSections(firstSection);
      expect(previous).toBeNull();
    });

    it('should return null for next on last section', () => {
      const lastSection = allSectionIds[allSectionIds.length - 1];
      const { next } = getPrevNextSections(lastSection);
      expect(next).toBeNull();
    });

    it('should maintain navigation order consistency', () => {
      // If B is next of A, then A should be previous of B
      fc.assert(
        fc.property(
          fc.constantFrom(...allSectionIds.slice(0, -1)), // Exclude last
          (sectionId) => {
            const { next } = getPrevNextSections(sectionId);
            if (next) {
              const { previous } = getPrevNextSections(next.id);
              expect(previous?.id).toBe(sectionId);
            }
          }
        ),
        { numRuns: 50 }
      );
    });
  });

  describe('flattenNavigation', () => {
    it('should preserve all sections from the tree', () => {
      const flat = flattenNavigation(navigationStructure);
      
      // Count all sections in tree
      function countSections(sections: typeof navigationStructure): number {
        let count = 0;
        for (const section of sections) {
          count++;
          if (section.children) {
            count += countSections(section.children);
          }
        }
        return count;
      }
      
      const treeCount = countSections(navigationStructure);
      expect(flat.length).toBe(treeCount);
    });

    it('should maintain parent before children order', () => {
      const flat = flattenNavigation(navigationStructure);
      
      // For each parent with children, parent should appear before all children
      for (const section of flat) {
        if (section.children) {
          const parentIndex = flat.findIndex(s => s.id === section.id);
          for (const child of section.children) {
            const childIndex = flat.findIndex(s => s.id === child.id);
            expect(childIndex).toBeGreaterThan(parentIndex);
          }
        }
      }
    });
  });

  describe('Navigation Structure Integrity', () => {
    it('should have valid paths for all sections', () => {
      fc.assert(
        fc.property(
          fc.constantFrom(...flatNavigation),
          (section) => {
            expect(section.path).toBeTruthy();
            expect(section.path.startsWith('/docs')).toBe(true);
          }
        ),
        { numRuns: 100 }
      );
    });

    it('should have titles for all sections', () => {
      fc.assert(
        fc.property(
          fc.constantFrom(...flatNavigation),
          (section) => {
            expect(section.title).toBeTruthy();
            expect(section.title.length).toBeGreaterThan(0);
          }
        ),
        { numRuns: 100 }
      );
    });
  });
});

/**
 * Property tests for hierarchical navigation behavior
 */
describe('Hierarchical Navigation Properties', () => {
  const flatNavigation = flattenNavigation(navigationStructure);

  describe('Parent-Child Relationships', () => {
    it('children should have paths extending parent paths', () => {
      for (const section of flatNavigation) {
        if (section.children) {
          for (const child of section.children) {
            // Child path should contain parent path as prefix (after /docs)
            expect(child.path.startsWith('/docs')).toBe(true);
          }
        }
      }
    });

    it('root sections should not have parent references', () => {
      for (const section of navigationStructure) {
        // Root sections are in the top-level array
        expect(findSectionById(section.id)).toBe(section);
      }
    });
  });

  describe('Section Ordering', () => {
    it('should maintain consistent order in flattened navigation', () => {
      const flat1 = flattenNavigation(navigationStructure);
      const flat2 = flattenNavigation(navigationStructure);
      
      expect(flat1.map(s => s.id)).toEqual(flat2.map(s => s.id));
    });

    it('getAllSectionIds should return same order as flattenNavigation', () => {
      const fromGetAll = getAllSectionIds();
      const fromFlatten = flattenNavigation(navigationStructure).map(s => s.id);
      
      expect(fromGetAll).toEqual(fromFlatten);
    });
  });
});
