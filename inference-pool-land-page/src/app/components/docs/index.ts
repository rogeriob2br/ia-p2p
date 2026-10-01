/**
 * Documentation components barrel export
 * Feature: landing-page-documentation
 */

// Layout
export { DocumentationLayout } from './DocumentationLayout';
export { DocumentationHeader } from './DocumentationHeader';

// Navigation
export { Sidebar } from './Sidebar';
export { NavigationTree } from './NavigationTree';
export { MobileMenu } from './MobileMenu';
export { Breadcrumbs } from './Breadcrumbs';
export { PageNavigation } from './PageNavigation';

// Search
export { SearchInput } from './SearchInput';
export { SearchModal } from './SearchModal';

// Content
export { TableOfContents, extractHeadings, extractHeadingsFromMarkdown } from './TableOfContents';
export { ContentRenderer, extractHeadingIds } from './ContentRenderer';
export { CodeBlock, InlineCode } from './CodeBlock';
export type { CodeBlockProps } from './CodeBlock';

// Error Handling
export { ErrorBoundary } from './ErrorBoundary';
export { NotFound } from './NotFound';

// Accessibility
export { SkipToContent } from './SkipToContent';

// SEO
export { SEOHead, generateBreadcrumbSchema } from './SEOHead';
