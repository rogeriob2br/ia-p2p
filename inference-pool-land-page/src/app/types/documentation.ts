/**
 * Types and interfaces for the Documentation system
 * Feature: landing-page-documentation
 */

import type { ComponentType } from 'react';

// ============================================================================
// Navigation Types
// ============================================================================

/**
 * Represents a single navigation section in the documentation sidebar
 */
export interface NavigationSection {
  id: string;
  title: string;
  icon?: ComponentType<{ className?: string }>;
  children?: NavigationSection[];
  path: string;
}

/**
 * Props for the Sidebar component
 */
export interface SidebarProps {
  sections: NavigationSection[];
  currentSection: string;
  onSectionChange: (sectionId: string) => void;
  isOpen: boolean;
  onToggle: () => void;
}

// ============================================================================
// Search Types
// ============================================================================

/**
 * Represents a search result item
 */
export interface SearchResult {
  sectionId: string;
  title: string;
  excerpt: string;
  matchedText: string;
  path: string;
}

/**
 * Props for the SearchInput component
 */
export interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  onResultSelect: (sectionId: string) => void;
  placeholder?: string;
}

// ============================================================================
// Content Types
// ============================================================================

/**
 * Represents a documentation section's content
 */
export interface Section {
  id: string;
  title: string;
  slug: string;
  content: string; // Markdown content
  order: number;
  parent?: string;
  children?: string[];
  estimatedReadTime: number; // in minutes
  lastUpdated: string;
}

/**
 * Metadata for the documentation
 */
export interface DocumentationMetadata {
  version: string;
  lastUpdated: string;
  sections: SectionMetadata[];
}

/**
 * Metadata for a single section
 */
export interface SectionMetadata {
  id: string;
  title: string;
  description: string;
  keywords: string[];
}

/**
 * Complete documentation content structure
 */
export interface DocumentationContent {
  sections: Section[];
  metadata: DocumentationMetadata;
}

// ============================================================================
// Content Renderer Types
// ============================================================================

/**
 * Props for the ContentRenderer component
 */
export interface ContentRendererProps {
  content: string;
  sectionId: string;
}

/**
 * Props for the CodeBlock component
 */
export interface CodeBlockProps {
  code: string;
  language: string;
  filename?: string;
  showLineNumbers?: boolean;
}

// ============================================================================
// Table of Contents Types
// ============================================================================

/**
 * Represents a heading for the Table of Contents
 */
export interface TOCHeading {
  id: string;
  text: string;
  level: number; // 2, 3, or 4 for h2, h3, h4
}

/**
 * Props for the TableOfContents component
 */
export interface TableOfContentsProps {
  headings: TOCHeading[];
  activeHeading: string;
  onHeadingClick: (headingId: string) => void;
}

// ============================================================================
// Breadcrumb Types
// ============================================================================

/**
 * Represents a breadcrumb item
 */
export interface BreadcrumbItem {
  id: string;
  title: string;
  path: string;
}

/**
 * Props for the Breadcrumbs component
 */
export interface BreadcrumbsProps {
  items: BreadcrumbItem[];
  onNavigate: (path: string) => void;
}

// ============================================================================
// Page Navigation Types
// ============================================================================

/**
 * Represents a navigation link (prev/next)
 */
export interface PageNavLink {
  id: string;
  title: string;
  path: string;
}

/**
 * Props for the PageNavigation component
 */
export interface PageNavigationProps {
  previous?: PageNavLink;
  next?: PageNavLink;
  onNavigate: (path: string) => void;
}

// ============================================================================
// Layout Types
// ============================================================================

/**
 * State for the main Documentation component
 */
export interface DocumentationState {
  currentSection: string;
  searchQuery: string;
  isSidebarOpen: boolean;
  activeHeading: string;
}

/**
 * Props for the main Documentation component
 */
export interface DocumentationPageProps {
  onNavigate: (page: string) => void;
}

// ============================================================================
// Error Types
// ============================================================================

/**
 * Type of error that can occur in the documentation
 */
export type ErrorType = 
  | 'content-not-found' 
  | 'search-error' 
  | 'network-error' 
  | 'clipboard-error';

/**
 * Represents an error state in the documentation
 */
export interface ErrorState {
  type: ErrorType;
  message: string;
  recoverable: boolean;
  retryAction?: () => void;
}

