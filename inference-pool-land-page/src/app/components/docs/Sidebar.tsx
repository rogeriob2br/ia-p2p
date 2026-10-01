/**
 * Sidebar Component
 * Feature: landing-page-documentation
 * 
 * Documentation sidebar with hierarchical navigation tree.
 * Supports expandable sections and active section highlighting.
 */

import { useState, useCallback } from 'react';
import { ChevronRight, ChevronDown } from 'lucide-react';
import type { NavigationSection } from '@/app/types';
import { cn } from '@/app/components/ui/utils';

interface SidebarProps {
  sections: NavigationSection[];
  currentSection: string;
  onSectionChange: (sectionId: string) => void;
}

export function Sidebar({
  sections,
  currentSection,
  onSectionChange
}: SidebarProps) {
  return (
    <nav className="px-4" aria-label="Documentation sections">
      <div className="space-y-1">
        {sections.map((section) => (
          <NavigationItem
            key={section.id}
            section={section}
            currentSection={currentSection}
            onSelect={onSectionChange}
            level={0}
          />
        ))}
      </div>
    </nav>
  );
}

interface NavigationItemProps {
  section: NavigationSection;
  currentSection: string;
  onSelect: (sectionId: string) => void;
  level: number;
}

function NavigationItem({
  section,
  currentSection,
  onSelect,
  level
}: NavigationItemProps) {
  const hasChildren = section.children && section.children.length > 0;
  const isActive = currentSection === section.id;
  const isParentOfActive = hasChildren && 
    section.children?.some(child => 
      child.id === currentSection || 
      child.children?.some(grandchild => grandchild.id === currentSection)
    );
  
  const [isExpanded, setIsExpanded] = useState(isActive || isParentOfActive);

  const handleClick = useCallback(() => {
    if (hasChildren) {
      setIsExpanded(!isExpanded);
    }
    onSelect(section.id);
  }, [hasChildren, isExpanded, onSelect, section.id]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleClick();
    }
    if (hasChildren) {
      if (e.key === 'ArrowRight' && !isExpanded) {
        e.preventDefault();
        setIsExpanded(true);
      }
      if (e.key === 'ArrowLeft' && isExpanded) {
        e.preventDefault();
        setIsExpanded(false);
      }
    }
  }, [handleClick, hasChildren, isExpanded]);

  const Icon = section.icon;
  const paddingLeft = level * 12 + 12;

  return (
    <div>
      <button
        onClick={handleClick}
        onKeyDown={handleKeyDown}
        className={cn(
          'w-full flex items-center gap-2 py-2 px-3 rounded-lg text-sm font-medium transition-all',
          'focus:outline-none focus:ring-2 focus:ring-offset-1',
          isActive
            ? 'text-white'
            : 'text-gray-700 hover:bg-gray-100 hover:text-gray-900',
          level > 0 && 'text-sm'
        )}
        style={{
          paddingLeft: `${paddingLeft}px`,
          backgroundColor: isActive ? '#3C35F2' : undefined,
          // @ts-expect-error CSS custom property
          '--tw-ring-color': '#3C35F2'
        }}
        aria-expanded={hasChildren ? isExpanded : undefined}
        aria-current={isActive ? 'page' : undefined}
      >
        {/* Expand/collapse indicator for items with children */}
        {hasChildren && (
          <span className="flex-shrink-0 w-4 h-4">
            {isExpanded ? (
              <ChevronDown className="w-4 h-4" />
            ) : (
              <ChevronRight className="w-4 h-4" />
            )}
          </span>
        )}
        
        {/* Icon */}
        {Icon && !hasChildren && (
          <Icon className={cn(
            'w-4 h-4 flex-shrink-0',
            isActive ? 'text-white' : 'text-gray-400'
          )} />
        )}
        
        {/* Title */}
        <span className="truncate">{section.title}</span>
      </button>

      {/* Children */}
      {hasChildren && isExpanded && (
        <div className="mt-1 space-y-1" role="group">
          {section.children!.map((child) => (
            <NavigationItem
              key={child.id}
              section={child}
              currentSection={currentSection}
              onSelect={onSelect}
              level={level + 1}
            />
          ))}
        </div>
      )}
    </div>
  );
}
