/**
 * TableOfContents Component
 * Feature: landing-page-documentation
 * 
 * Right sidebar showing section headings for the current page.
 * Highlights the currently visible heading based on scroll position.
 */

import type { TOCHeading } from '@/app/types';
import { cn } from '@/app/components/ui/utils';

interface TableOfContentsProps {
  headings: TOCHeading[];
  activeHeading: string;
  onHeadingClick: (headingId: string) => void;
}

export function TableOfContents({
  headings,
  activeHeading,
  onHeadingClick
}: TableOfContentsProps) {
  if (headings.length === 0) {
    return null;
  }

  const handleClick = (headingId: string) => {
    onHeadingClick(headingId);
    
    // Scroll to the heading
    const element = document.getElementById(headingId);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  };

  return (
    <nav aria-label="Table of contents">
      <h4 className="font-semibold text-sm text-gray-900 mb-4">
        Nesta página
      </h4>
      <ul className="space-y-2">
        {headings.map((heading) => {
          const isActive = activeHeading === heading.id;
          const indentClass = heading.level === 3 
            ? 'pl-4' 
            : heading.level === 4 
              ? 'pl-6' 
              : '';

          return (
            <li key={heading.id}>
              <button
                onClick={() => handleClick(heading.id)}
                className={cn(
                  'text-left text-sm w-full py-1 transition-colors border-l-2',
                  indentClass,
                  isActive
                    ? 'border-[#3C35F2] text-[#3C35F2] font-medium pl-3'
                    : 'border-transparent text-gray-600 hover:text-gray-900 hover:border-gray-300 pl-3'
                )}
                aria-current={isActive ? 'true' : undefined}
              >
                {heading.text}
              </button>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}

/**
 * Extract headings from HTML content
 * Used to generate TOC from rendered markdown
 */
export function extractHeadings(content: string): TOCHeading[] {
  const headings: TOCHeading[] = [];
  const regex = /<h([2-4])\s+id="([^"]+)"[^>]*>([^<]+)<\/h[2-4]>/gi;
  let match;

  while ((match = regex.exec(content)) !== null) {
    headings.push({
      level: parseInt(match[1], 10),
      id: match[2],
      text: match[3].trim()
    });
  }

  return headings;
}

/**
 * Extract headings from markdown content (before rendering)
 */
export function extractHeadingsFromMarkdown(markdown: string): TOCHeading[] {
  const headings: TOCHeading[] = [];
  const lines = markdown.split('\n');

  for (const line of lines) {
    const match = line.match(/^(#{2,4})\s+(.+)$/);
    if (match) {
      const level = match[1].length;
      const text = match[2].trim();
      const id = text
        .toLowerCase()
        .replace(/[^\w\s-]/g, '')
        .replace(/\s+/g, '-');

      headings.push({ level, id, text });
    }
  }

  return headings;
}
