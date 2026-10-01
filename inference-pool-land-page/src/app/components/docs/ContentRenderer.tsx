/**
 * ContentRenderer Component
 * Feature: landing-page-documentation
 * 
 * Renders Markdown content with custom components for headings,
 * code blocks, links, and other elements.
 * 
 * Requirements: 2.5, 5.5
 */

import { useMemo } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import type { Components } from 'react-markdown';
import { CodeBlock, InlineCode } from './CodeBlock';
import { cn } from '@/app/components/ui/utils';

interface ContentRendererProps {
  /** Markdown content to render */
  content: string;
  /** Section ID for heading anchors */
  sectionId?: string;
  /** Additional class names */
  className?: string;
}

/**
 * Generate a slug from heading text for anchor links
 */
function generateSlug(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w\s-]/g, '')
    .replace(/\s+/g, '-')
    .trim();
}

export function ContentRenderer({
  content,
  sectionId,
  className
}: ContentRendererProps) {
  // Custom components for ReactMarkdown
  const components = useMemo<Components>(() => ({
    // Headings with anchor links
    h1: ({ children, ...props }) => {
      const text = String(children);
      const id = generateSlug(text);
      return (
        <h1 
          id={id} 
          className="text-4xl font-bold mt-8 mb-4 text-gray-900 scroll-mt-20"
          {...props}
        >
          <a href={`#${id}`} className="hover:text-[#3C35F2] transition-colors">
            {children}
          </a>
        </h1>
      );
    },
    
    h2: ({ children, ...props }) => {
      const text = String(children);
      const id = generateSlug(text);
      return (
        <h2 
          id={id} 
          className="text-2xl font-bold mt-8 mb-4 text-gray-900 scroll-mt-20"
          {...props}
        >
          <a href={`#${id}`} className="hover:text-[#3C35F2] transition-colors">
            {children}
          </a>
        </h2>
      );
    },
    
    h3: ({ children, ...props }) => {
      const text = String(children);
      const id = generateSlug(text);
      return (
        <h3 
          id={id} 
          className="text-xl font-bold mt-6 mb-3 text-gray-900 scroll-mt-20"
          {...props}
        >
          <a href={`#${id}`} className="hover:text-[#3C35F2] transition-colors">
            {children}
          </a>
        </h3>
      );
    },
    
    h4: ({ children, ...props }) => {
      const text = String(children);
      const id = generateSlug(text);
      return (
        <h4 
          id={id} 
          className="text-lg font-bold mt-4 mb-2 text-gray-900 scroll-mt-20"
          {...props}
        >
          {children}
        </h4>
      );
    },

    // Paragraphs
    p: ({ children }) => (
      <p className="text-gray-600 leading-7 mb-4">
        {children}
      </p>
    ),

    // Links
    a: ({ href, children }) => {
      const isExternal = href?.startsWith('http');
      return (
        <a
          href={href}
          className="text-[#3C35F2] hover:underline font-medium"
          target={isExternal ? '_blank' : undefined}
          rel={isExternal ? 'noopener noreferrer' : undefined}
        >
          {children}
        </a>
      );
    },

    // Code blocks
    code: ({ className, children, ...props }) => {
      const match = /language-(\w+)/.exec(className || '');
      const isCodeBlock = match !== null;
      
      if (isCodeBlock) {
        const language = match[1];
        const code = String(children).replace(/\n$/, '');
        return <CodeBlock code={code} language={language} />;
      }
      
      // Inline code
      return <InlineCode>{children}</InlineCode>;
    },

    // Pre wrapper (handled by code component)
    pre: ({ children }) => (
      <div className="my-4">
        {children}
      </div>
    ),

    // Lists
    ul: ({ children }) => (
      <ul className="list-disc list-inside space-y-2 mb-4 text-gray-600">
        {children}
      </ul>
    ),
    
    ol: ({ children }) => (
      <ol className="list-decimal list-inside space-y-2 mb-4 text-gray-600">
        {children}
      </ol>
    ),
    
    li: ({ children }) => (
      <li className="leading-7">
        {children}
      </li>
    ),

    // Blockquotes
    blockquote: ({ children }) => (
      <blockquote className="border-l-4 border-[#3C35F2] pl-4 py-2 my-4 bg-blue-50 rounded-r-lg">
        <div className="text-gray-700 italic">
          {children}
        </div>
      </blockquote>
    ),

    // Horizontal rule
    hr: () => (
      <hr className="my-8 border-gray-200" />
    ),

    // Tables
    table: ({ children }) => (
      <div className="overflow-x-auto my-4">
        <table className="min-w-full divide-y divide-gray-200 border border-gray-200 rounded-lg">
          {children}
        </table>
      </div>
    ),
    
    thead: ({ children }) => (
      <thead className="bg-gray-50">
        {children}
      </thead>
    ),
    
    tbody: ({ children }) => (
      <tbody className="divide-y divide-gray-200">
        {children}
      </tbody>
    ),
    
    tr: ({ children }) => (
      <tr className="hover:bg-gray-50">
        {children}
      </tr>
    ),
    
    th: ({ children }) => (
      <th className="px-4 py-3 text-left text-sm font-semibold text-gray-900">
        {children}
      </th>
    ),
    
    td: ({ children }) => (
      <td className="px-4 py-3 text-sm text-gray-600">
        {children}
      </td>
    ),

    // Strong/Bold
    strong: ({ children }) => (
      <strong className="font-semibold text-gray-900">
        {children}
      </strong>
    ),

    // Emphasis/Italic
    em: ({ children }) => (
      <em className="italic">
        {children}
      </em>
    ),

    // Images
    img: ({ src, alt }) => (
      <figure className="my-6">
        <img
          src={src}
          alt={alt || ''}
          className="rounded-lg shadow-md max-w-full h-auto"
          loading="lazy"
        />
        {alt && (
          <figcaption className="text-center text-sm text-gray-500 mt-2">
            {alt}
          </figcaption>
        )}
      </figure>
    ),
  }), []);

  return (
    <article 
      className={cn('prose prose-gray max-w-none', className)}
      data-section-id={sectionId}
    >
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={components}
      >
        {content}
      </ReactMarkdown>
    </article>
  );
}

/**
 * Extract all heading IDs from markdown content
 * Useful for generating table of contents
 */
export function extractHeadingIds(content: string): { id: string; text: string; level: number }[] {
  const headings: { id: string; text: string; level: number }[] = [];
  const lines = content.split('\n');

  for (const line of lines) {
    const match = line.match(/^(#{1,4})\s+(.+)$/);
    if (match) {
      const level = match[1].length;
      const text = match[2].trim();
      const id = generateSlug(text);
      headings.push({ id, text, level });
    }
  }

  return headings;
}
