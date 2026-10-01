/**
 * DocumentationLayout Component
 * Feature: landing-page-documentation
 * 
 * Main layout component for documentation pages with three-column responsive design:
 * - Desktop (>1024px): Sidebar | Content | Table of Contents
 * - Tablet (768-1024px): Collapsible Sidebar | Content
 * - Mobile (<768px): Hidden Sidebar (hamburger menu) | Content
 */

import { useState, useEffect, type ReactNode } from 'react';
import { Menu, X } from 'lucide-react';
import { Button } from '@/app/components/ui/button';

interface DocumentationLayoutProps {
  /** Left sidebar content (navigation) */
  sidebar: ReactNode;
  /** Main documentation content */
  children: ReactNode;
  /** Right sidebar content (table of contents) - hidden on tablet/mobile */
  tableOfContents?: ReactNode;
  /** Header component */
  header: ReactNode;
  /** Footer component */
  footer: ReactNode;
}

export function DocumentationLayout({
  sidebar,
  children,
  tableOfContents,
  header,
  footer
}: DocumentationLayoutProps) {
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [isMobile, setIsMobile] = useState(false);
  const [isTablet, setIsTablet] = useState(false);

  // Handle responsive breakpoints
  useEffect(() => {
    const checkBreakpoint = () => {
      const width = window.innerWidth;
      setIsMobile(width < 768);
      setIsTablet(width >= 768 && width < 1024);
    };

    checkBreakpoint();
    window.addEventListener('resize', checkBreakpoint);
    return () => window.removeEventListener('resize', checkBreakpoint);
  }, []);

  // Close sidebar on navigation (mobile)
  useEffect(() => {
    if (!isMobile) {
      setIsSidebarOpen(false);
    }
  }, [isMobile]);

  // Prevent body scroll when mobile sidebar is open
  useEffect(() => {
    if (isSidebarOpen && isMobile) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => {
      document.body.style.overflow = '';
    };
  }, [isSidebarOpen, isMobile]);

  const toggleSidebar = () => setIsSidebarOpen(!isSidebarOpen);
  const closeSidebar = () => setIsSidebarOpen(false);

  return (
    <div className="min-h-screen flex flex-col bg-gray-50">
      {/* Header */}
      {header}

      {/* Mobile Menu Button */}
      {isMobile && (
        <div className="fixed top-20 left-4 z-40">
          <Button
            variant="outline"
            size="icon"
            onClick={toggleSidebar}
            className="bg-white shadow-md border-gray-200"
            aria-label={isSidebarOpen ? 'Close menu' : 'Open menu'}
          >
            {isSidebarOpen ? (
              <X className="h-5 w-5" />
            ) : (
              <Menu className="h-5 w-5" />
            )}
          </Button>
        </div>
      )}

      {/* Mobile Overlay */}
      {isMobile && isSidebarOpen && (
        <div
          className="fixed inset-0 bg-black/50 z-30 transition-opacity"
          onClick={closeSidebar}
          aria-hidden="true"
        />
      )}

      {/* Main Layout Container */}
      <div className="flex-1 flex">
        {/* Sidebar */}
        <aside
          className={`
            ${isMobile 
              ? `fixed inset-y-0 left-0 z-40 w-72 transform transition-transform duration-300 ease-in-out
                 ${isSidebarOpen ? 'translate-x-0' : '-translate-x-full'}`
              : 'sticky top-0 h-screen w-64 lg:w-72 flex-shrink-0'
            }
            bg-white border-r border-gray-200 overflow-y-auto
          `}
          aria-label="Documentation navigation"
        >
          <div className={`${isMobile ? 'pt-20' : 'pt-4'} pb-8`}>
            {sidebar}
          </div>
        </aside>

        {/* Main Content Area */}
        <main 
          className="flex-1 min-w-0"
          role="main"
        >
          <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
            {children}
          </div>
        </main>

        {/* Table of Contents - Desktop only */}
        {!isMobile && !isTablet && tableOfContents && (
          <aside
            className="hidden lg:block sticky top-0 h-screen w-56 xl:w-64 flex-shrink-0 border-l border-gray-200 bg-white overflow-y-auto"
            aria-label="Table of contents"
          >
            <div className="pt-4 pb-8 px-4">
              {tableOfContents}
            </div>
          </aside>
        )}
      </div>

      {/* Footer */}
      {footer}
    </div>
  );
}
