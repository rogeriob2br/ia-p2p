/**
 * Breadcrumbs Component
 * Feature: landing-page-documentation
 * 
 * Displays the navigation path to the current documentation section.
 */

import { ChevronRight, Home } from 'lucide-react';
import type { BreadcrumbItem } from '@/app/types';

interface BreadcrumbsProps {
  items: BreadcrumbItem[];
  onNavigate: (sectionId: string) => void;
}

export function Breadcrumbs({ items, onNavigate }: BreadcrumbsProps) {
  if (items.length === 0) {
    return null;
  }

  return (
    <nav 
      aria-label="Breadcrumb" 
      className="flex items-center text-sm text-gray-500 mb-6"
    >
      <ol className="flex items-center flex-wrap gap-1">
        {/* Home link */}
        <li className="flex items-center">
          <button
            onClick={() => onNavigate('getting-started')}
            className="flex items-center hover:text-gray-700 transition-colors"
            aria-label="Documentation home"
          >
            <Home className="w-4 h-4" />
          </button>
        </li>

        {/* Separator after home */}
        <li aria-hidden="true">
          <ChevronRight className="w-4 h-4 text-gray-400" />
        </li>

        {/* Breadcrumb items */}
        {items.map((item, index) => {
          const isLast = index === items.length - 1;

          return (
            <li key={item.id} className="flex items-center">
              {isLast ? (
                <span 
                  className="font-medium text-gray-900"
                  aria-current="page"
                >
                  {item.title}
                </span>
              ) : (
                <>
                  <button
                    onClick={() => onNavigate(item.id)}
                    className="hover:text-gray-700 transition-colors"
                  >
                    {item.title}
                  </button>
                  <ChevronRight 
                    className="w-4 h-4 text-gray-400 mx-1" 
                    aria-hidden="true"
                  />
                </>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
