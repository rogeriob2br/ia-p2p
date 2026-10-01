/**
 * PageNavigation Component
 * Feature: landing-page-documentation
 * 
 * Previous/Next navigation links at the bottom of documentation pages.
 */

import { ArrowLeft, ArrowRight } from 'lucide-react';
import type { PageNavLink } from '@/app/types';

interface PageNavigationProps {
  previous?: PageNavLink;
  next?: PageNavLink;
  onNavigate: (sectionId: string) => void;
}

export function PageNavigation({
  previous,
  next,
  onNavigate
}: PageNavigationProps) {
  if (!previous && !next) {
    return null;
  }

  return (
    <nav
      aria-label="Page navigation"
      className="flex items-center justify-between mt-12 pt-8 border-t border-gray-200"
    >
      {/* Previous link */}
      <div className="flex-1">
        {previous && (
          <button
            onClick={() => onNavigate(previous.id)}
            className="group flex flex-col items-start p-4 -ml-4 rounded-lg hover:bg-gray-50 transition-colors"
          >
            <span className="flex items-center gap-2 text-sm text-gray-500 group-hover:text-gray-700">
              <ArrowLeft className="w-4 h-4" />
              Anterior
            </span>
            <span 
              className="font-medium text-gray-900 group-hover:text-[#3C35F2] transition-colors"
            >
              {previous.title}
            </span>
          </button>
        )}
      </div>

      {/* Next link */}
      <div className="flex-1 flex justify-end">
        {next && (
          <button
            onClick={() => onNavigate(next.id)}
            className="group flex flex-col items-end p-4 -mr-4 rounded-lg hover:bg-gray-50 transition-colors"
          >
            <span className="flex items-center gap-2 text-sm text-gray-500 group-hover:text-gray-700">
              Próximo
              <ArrowRight className="w-4 h-4" />
            </span>
            <span 
              className="font-medium text-gray-900 group-hover:text-[#3C35F2] transition-colors"
            >
              {next.title}
            </span>
          </button>
        )}
      </div>
    </nav>
  );
}
