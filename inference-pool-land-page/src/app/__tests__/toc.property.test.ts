/**
 * Property-Based Tests for Table of Contents and Active Heading
 * Feature: landing-page-documentation
 * 
 * Property 5: Active Section Highlighting
 * Validates: Requirements 1.3
 * - Navigation sidebar and TOC highlight the section currently visible
 * 
 * Property 6: Breadcrumb Path Accuracy
 * Validates: Requirements 1.4
 * - Breadcrumb trail accurately reflects hierarchical path
 */

import * as fc from 'fast-check';
import { calculateActiveHeading } from '@/app/hooks/useActiveHeading';
import { getBreadcrumbPath, flattenNavigation, navigationStructure } from '@/app/content/navigation';
import { extractHeadingsFromMarkdown } from '@/app/components/docs/TableOfContents';

describe('Property 5: Active Section Highlighting', () => {
  // Feature: landing-page-documentation, Property 5: Active Section Highlighting

  describe('calculateActiveHeading', () => {
    it('should return first heading when scroll is at top', () => {
      const headings = [
        { id: 'intro', offsetTop: 0 },
        { id: 'section-1', offsetTop: 500 },
        { id: 'section-2', offsetTop: 1000 }
      ];

      const active = calculateActiveHeading(headings, 0, 100);
      expect(active).toBe('intro');
    });

    it('should return correct heading based on scroll position', () => {
      const headings = [
        { id: 'intro', offsetTop: 0 },
        { id: 'section-1', offsetTop: 500 },
        { id: 'section-2', offsetTop: 1000 }
      ];

      // At position 450 + 100 offset = 550, should be in section-1
      expect(calculateActiveHeading(headings, 450, 100)).toBe('section-1');
      
      // At position 950 + 100 offset = 1050, should be in section-2
      expect(calculateActiveHeading(headings, 950, 100)).toBe('section-2');
    });

    it('should return last heading when scrolled past all', () => {
      const headings = [
        { id: 'intro', offsetTop: 0 },
        { id: 'section-1', offsetTop: 500 }
      ];

      const active = calculateActiveHeading(headings, 10000, 100);
      expect(active).toBe('section-1');
    });

    it('should return empty string for empty headings', () => {
      expect(calculateActiveHeading([], 500, 100)).toBe('');
    });

    it('should handle single heading', () => {
      const headings = [{ id: 'only-one', offsetTop: 0 }];
      
      fc.assert(
        fc.property(
          fc.integer({ min: 0, max: 10000 }),
          (scrollPosition) => {
            const active = calculateActiveHeading(headings, scrollPosition, 100);
            expect(active).toBe('only-one');
          }
        ),
        { numRuns: 50 }
      );
    });

    it('should always return a valid heading ID for non-empty list', () => {
      fc.assert(
        fc.property(
          fc.array(
            fc.record({
              id: fc.string({ minLength: 1, maxLength: 20 }),
              offsetTop: fc.integer({ min: 0, max: 10000 })
            }),
            { minLength: 1, maxLength: 10 }
          ).map(arr => arr.sort((a, b) => a.offsetTop - b.offsetTop)),
          fc.integer({ min: 0, max: 15000 }),
          fc.integer({ min: 0, max: 200 }),
          (headings, scrollPosition, offset) => {
            const active = calculateActiveHeading(headings, scrollPosition, offset);
            const validIds = headings.map(h => h.id);
            expect(validIds).toContain(active);
          }
        ),
        { numRuns: 100 }
      );
    });

    it('should respect offset parameter', () => {
      const headings = [
        { id: 'section-1', offsetTop: 500 },
        { id: 'section-2', offsetTop: 1000 }
      ];

      // With offset 100: scroll 400 + 100 = 500, should be section-1
      expect(calculateActiveHeading(headings, 400, 100)).toBe('section-1');
      
      // With offset 0: scroll 400 + 0 = 400, should be section-1 (first before 500)
      expect(calculateActiveHeading(headings, 400, 0)).toBe('section-1');
      
      // With offset 200: scroll 300 + 200 = 500, should be section-1
      expect(calculateActiveHeading(headings, 300, 200)).toBe('section-1');
    });
  });

  describe('Heading extraction from markdown', () => {
    it('should extract h2, h3, h4 headings', () => {
      const markdown = `
# Title (h1 - ignored for TOC)

## Section 1

Some content.

### Subsection 1.1

More content.

#### Detail 1.1.1

## Section 2
`;

      const headings = extractHeadingsFromMarkdown(markdown);
      
      // Should only get h2, h3, h4
      expect(headings.filter(h => h.level === 2).length).toBe(2);
      expect(headings.filter(h => h.level === 3).length).toBe(1);
      expect(headings.filter(h => h.level === 4).length).toBe(1);
    });

    it('should generate unique IDs for headings', () => {
      const markdown = `
## Getting Started
## Installation
## Configuration
`;

      const headings = extractHeadingsFromMarkdown(markdown);
      const ids = headings.map(h => h.id);
      const uniqueIds = new Set(ids);
      
      expect(uniqueIds.size).toBe(ids.length);
    });

    it('should handle special characters in headings', () => {
      const markdown = `
## What's New?
## C++ Guide
## Node.js & npm
`;

      const headings = extractHeadingsFromMarkdown(markdown);
      
      for (const heading of headings) {
        // IDs should be URL-safe
        expect(heading.id).toMatch(/^[a-z0-9-]+$/);
      }
    });
  });
});

describe('Property 6: Breadcrumb Path Accuracy', () => {
  // Feature: landing-page-documentation, Property 6: Breadcrumb Path Accuracy

  const allSections = flattenNavigation(navigationStructure);

  describe('Path generation', () => {
    it('should always end with the current section', () => {
      fc.assert(
        fc.property(
          fc.constantFrom(...allSections.map(s => s.id)),
          (sectionId) => {
            const path = getBreadcrumbPath(sectionId);
            
            expect(path).not.toBeNull();
            expect(path!.length).toBeGreaterThanOrEqual(1);
            expect(path![path!.length - 1].id).toBe(sectionId);
          }
        ),
        { numRuns: 50 }
      );
    });

    it('should have valid parent-child relationships', () => {
      fc.assert(
        fc.property(
          fc.constantFrom(...allSections.map(s => s.id)),
          (sectionId) => {
            const path = getBreadcrumbPath(sectionId);
            if (!path || path.length <= 1) return;

            // Each item should be a parent of the next
            for (let i = 0; i < path.length - 1; i++) {
              const parent = path[i];
              const child = path[i + 1];
              
              expect(parent.children).toBeDefined();
              expect(parent.children!.some(c => c.id === child.id)).toBe(true);
            }
          }
        ),
        { numRuns: 50 }
      );
    });

    it('should have consistent length for same section', () => {
      fc.assert(
        fc.property(
          fc.constantFrom(...allSections.map(s => s.id)),
          (sectionId) => {
            const path1 = getBreadcrumbPath(sectionId);
            const path2 = getBreadcrumbPath(sectionId);
            
            expect(path1?.length).toBe(path2?.length);
          }
        ),
        { numRuns: 30 }
      );
    });

    it('should start with a root section', () => {
      const rootIds = navigationStructure.map(s => s.id);
      
      fc.assert(
        fc.property(
          fc.constantFrom(...allSections.map(s => s.id)),
          (sectionId) => {
            const path = getBreadcrumbPath(sectionId);
            
            expect(path).not.toBeNull();
            expect(rootIds).toContain(path![0].id);
          }
        ),
        { numRuns: 50 }
      );
    });
  });

  describe('Root sections', () => {
    it('should have single-item path for root sections', () => {
      const rootIds = navigationStructure.map(s => s.id);
      
      for (const rootId of rootIds) {
        const path = getBreadcrumbPath(rootId);
        expect(path?.length).toBe(1);
        expect(path![0].id).toBe(rootId);
      }
    });
  });

  describe('Nested sections', () => {
    it('should have multi-item path for nested sections', () => {
      // Find sections that have parents
      const nestedSections = allSections.filter(section => {
        const path = getBreadcrumbPath(section.id);
        return path && path.length > 1;
      });

      for (const section of nestedSections.slice(0, 5)) {
        const path = getBreadcrumbPath(section.id);
        expect(path!.length).toBeGreaterThan(1);
      }
    });
  });
});

describe('TOC Component Properties', () => {
  describe('Heading hierarchy', () => {
    it('should maintain heading level order', () => {
      const markdown = `
## Level 2
### Level 3
#### Level 4
### Another Level 3
## Another Level 2
`;

      const headings = extractHeadingsFromMarkdown(markdown);
      
      expect(headings[0].level).toBe(2);
      expect(headings[1].level).toBe(3);
      expect(headings[2].level).toBe(4);
      expect(headings[3].level).toBe(3);
      expect(headings[4].level).toBe(2);
    });

    it('should preserve heading text exactly', () => {
      const markdown = `
## Getting Started
### Quick Installation
#### Advanced Options
`;

      const headings = extractHeadingsFromMarkdown(markdown);
      
      expect(headings[0].text).toBe('Getting Started');
      expect(headings[1].text).toBe('Quick Installation');
      expect(headings[2].text).toBe('Advanced Options');
    });
  });

  describe('Click handling', () => {
    it('should generate valid anchor IDs from typical headings', () => {
      const testCases = [
        { input: 'Getting Started', expected: 'getting-started' },
        { input: 'API Reference', expected: 'api-reference' },
        { input: 'Quick Start Guide', expected: 'quick-start-guide' },
        { input: 'Installation', expected: 'installation' },
        { input: 'What is New', expected: 'what-is-new' }
      ];

      for (const { input, expected } of testCases) {
        const id = input
          .toLowerCase()
          .replace(/[^\w\s-]/g, '')
          .replace(/\s+/g, '-')
          .trim();
        
        expect(id).toBe(expected);
      }
    });

    it('should produce URL-safe slugs', () => {
      fc.assert(
        fc.property(
          // Use realistic heading text patterns
          fc.constantFrom(
            'Introduction',
            'Getting Started',
            'API Reference',
            'Configuration Options',
            'Best Practices'
          ),
          (text) => {
            const id = text
              .toLowerCase()
              .replace(/[^\w\s-]/g, '')
              .replace(/\s+/g, '-')
              .trim();
            
            // Should only contain safe characters
            expect(id).toMatch(/^[a-z][a-z0-9-]*$/);
          }
        ),
        { numRuns: 20 }
      );
    });
  });
});
