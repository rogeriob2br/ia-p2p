/**
 * useDocumentation Hook
 * Feature: landing-page-documentation
 * 
 * Central hook for managing documentation state including
 * current section, search, sidebar visibility, and navigation.
 */

import { useState, useCallback, useMemo } from 'react';
import { 
  navigationStructure, 
  getBreadcrumbPath, 
  getPrevNextSections,
  findSectionById,
  flattenNavigation
} from '@/app/content/navigation';
import { getSectionContent } from '@/app/content/docs';
import { extractHeadingsFromMarkdown } from '@/app/components/docs/TableOfContents';
import type { BreadcrumbItem, TOCHeading, NavigationSection } from '@/app/types';

interface UseDocumentationOptions {
  initialSection?: string;
}

interface UseDocumentationReturn {
  // Current state
  currentSection: string;
  sectionData: NavigationSection | null;
  content: string;
  headings: TOCHeading[];
  breadcrumbs: BreadcrumbItem[];
  previousSection: { id: string; title: string; path: string } | undefined;
  nextSection: { id: string; title: string; path: string } | undefined;
  
  // Sidebar state
  isSidebarOpen: boolean;
  toggleSidebar: () => void;
  closeSidebar: () => void;
  
  // Search state
  searchQuery: string;
  setSearchQuery: (query: string) => void;
  
  // Active heading (for TOC)
  activeHeading: string;
  setActiveHeading: (id: string) => void;
  
  // Navigation
  navigateToSection: (sectionId: string) => void;
  
  // Data
  navigation: NavigationSection[];
  allSectionIds: string[];
}

export function useDocumentation({
  initialSection = 'getting-started'
}: UseDocumentationOptions = {}): UseDocumentationReturn {
  // State
  const [currentSection, setCurrentSection] = useState(initialSection);
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [activeHeading, setActiveHeading] = useState('');

  // Computed values
  const sectionData = useMemo(() => {
    return findSectionById(currentSection);
  }, [currentSection]);

  const content = useMemo(() => {
    return getSectionContent(currentSection);
  }, [currentSection]);

  const headings = useMemo<TOCHeading[]>(() => {
    return extractHeadingsFromMarkdown(content);
  }, [content]);

  const breadcrumbs = useMemo<BreadcrumbItem[]>(() => {
    const path = getBreadcrumbPath(currentSection);
    if (!path) return [];
    return path.map(section => ({
      id: section.id,
      title: section.title,
      path: section.path
    }));
  }, [currentSection]);

  const { previousSection, nextSection } = useMemo(() => {
    const { previous, next } = getPrevNextSections(currentSection);
    return {
      previousSection: previous 
        ? { id: previous.id, title: previous.title, path: previous.path } 
        : undefined,
      nextSection: next 
        ? { id: next.id, title: next.title, path: next.path } 
        : undefined
    };
  }, [currentSection]);

  const allSectionIds = useMemo(() => {
    return flattenNavigation(navigationStructure).map(s => s.id);
  }, []);

  // Actions
  const toggleSidebar = useCallback(() => {
    setIsSidebarOpen(prev => !prev);
  }, []);

  const closeSidebar = useCallback(() => {
    setIsSidebarOpen(false);
  }, []);

  const navigateToSection = useCallback((sectionId: string) => {
    setCurrentSection(sectionId);
    setActiveHeading('');
    setIsSidebarOpen(false); // Close sidebar on mobile after navigation
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }, []);

  return {
    // Current state
    currentSection,
    sectionData,
    content,
    headings,
    breadcrumbs,
    previousSection,
    nextSection,
    
    // Sidebar state
    isSidebarOpen,
    toggleSidebar,
    closeSidebar,
    
    // Search state
    searchQuery,
    setSearchQuery,
    
    // Active heading
    activeHeading,
    setActiveHeading,
    
    // Navigation
    navigateToSection,
    
    // Data
    navigation: navigationStructure,
    allSectionIds
  };
}
