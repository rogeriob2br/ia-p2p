/**
 * NavigationTree Component
 * Feature: landing-page-documentation
 * 
 * Hierarchical navigation tree with expand/collapse functionality,
 * keyboard navigation, and active section highlighting.
 * 
 * Requirements: 1.1, 1.2, 9.3
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { ChevronRight, ChevronDown } from 'lucide-react';
import type { NavigationSection } from '@/app/types';
import { cn } from '@/app/components/ui/utils';

interface NavigationTreeProps {
  sections: NavigationSection[];
  currentSection: string;
  onSectionChange: (sectionId: string) => void;
  /** Called when sidebar should close (e.g., after mobile navigation) */
  onCloseSidebar?: () => void;
}

export function NavigationTree({
  sections,
  currentSection,
  onSectionChange,
  onCloseSidebar
}: NavigationTreeProps) {
  const navRef = useRef<HTMLElement>(null);

  // Handle keyboard navigation at tree level
  const handleTreeKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      const buttons = navRef.current?.querySelectorAll('button[data-nav-item]');
      if (!buttons) return;

      const currentIndex = Array.from(buttons).findIndex(
        btn => btn.getAttribute('aria-current') === 'page'
      );
      
      let nextIndex: number;
      if (e.key === 'ArrowDown') {
        nextIndex = currentIndex < buttons.length - 1 ? currentIndex + 1 : 0;
      } else {
        nextIndex = currentIndex > 0 ? currentIndex - 1 : buttons.length - 1;
      }

      (buttons[nextIndex] as HTMLButtonElement).focus();
    }
  }, []);

  return (
    <nav 
      ref={navRef}
      className="px-3"
      aria-label="Documentation sections"
      onKeyDown={handleTreeKeyDown}
    >
      {/* Version badge */}
      <div className="mb-4 px-3">
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-[#3C35F2]/10 text-[#3C35F2]">
          v0.1.0
        </span>
      </div>

      {/* Navigation items */}
      <ul className="space-y-1" role="tree">
        {sections.map((section) => (
          <NavigationNode
            key={section.id}
            section={section}
            currentSection={currentSection}
            onSelect={(id) => {
              onSectionChange(id);
              onCloseSidebar?.();
            }}
            level={0}
          />
        ))}
      </ul>
    </nav>
  );
}

interface NavigationNodeProps {
  section: NavigationSection;
  currentSection: string;
  onSelect: (sectionId: string) => void;
  level: number;
}

function NavigationNode({
  section,
  currentSection,
  onSelect,
  level
}: NavigationNodeProps) {
  const hasChildren = section.children && section.children.length > 0;
  const isActive = currentSection === section.id;
  
  // Check if any descendant is active
  const isAncestorOfActive = hasChildren && checkIfAncestor(section, currentSection);
  
  // Auto-expand if active or ancestor of active
  const [isExpanded, setIsExpanded] = useState(isActive || isAncestorOfActive);

  // Update expansion state when currentSection changes
  useEffect(() => {
    if (isActive || isAncestorOfActive) {
      setIsExpanded(true);
    }
  }, [isActive, isAncestorOfActive]);

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

  const handleExpandClick = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    setIsExpanded(!isExpanded);
  }, [isExpanded]);

  const Icon = section.icon;
  const indentPadding = level * 12;

  return (
    <li role="treeitem" aria-expanded={hasChildren ? isExpanded : undefined}>
      <div
        className={cn(
          'group flex items-center rounded-lg transition-all duration-150',
          isActive
            ? 'bg-[#3C35F2] text-white shadow-sm'
            : 'text-gray-700 hover:bg-gray-100'
        )}
        style={{ marginLeft: `${indentPadding}px` }}
      >
        {/* Expand/collapse button for items with children */}
        {hasChildren && (
          <button
            onClick={handleExpandClick}
            className={cn(
              'p-2 rounded-l-lg transition-colors',
              isActive 
                ? 'hover:bg-white/10' 
                : 'hover:bg-gray-200'
            )}
            aria-label={isExpanded ? 'Collapse section' : 'Expand section'}
            tabIndex={-1}
          >
            {isExpanded ? (
              <ChevronDown className="w-4 h-4" />
            ) : (
              <ChevronRight className="w-4 h-4" />
            )}
          </button>
        )}

        {/* Main navigation button */}
        <button
          onClick={handleClick}
          onKeyDown={handleKeyDown}
          data-nav-item
          className={cn(
            'flex-1 flex items-center gap-2 py-2 text-sm font-medium',
            'focus:outline-none focus:ring-2 focus:ring-[#3C35F2] focus:ring-offset-1 rounded-lg',
            hasChildren ? 'pr-3' : 'px-3',
            !hasChildren && 'rounded-lg'
          )}
          aria-current={isActive ? 'page' : undefined}
        >
          {/* Icon for leaf nodes */}
          {Icon && !hasChildren && (
            <Icon className={cn(
              'w-4 h-4 flex-shrink-0 transition-colors',
              isActive ? 'text-white' : 'text-gray-400 group-hover:text-gray-600'
            )} />
          )}
          
          {/* Section title */}
          <span className="truncate">{section.title}</span>

          {/* Active indicator dot for parent items */}
          {hasChildren && isAncestorOfActive && !isActive && (
            <span 
              className="w-1.5 h-1.5 rounded-full ml-auto"
              style={{ backgroundColor: '#3C35F2' }}
            />
          )}
        </button>
      </div>

      {/* Children */}
      {hasChildren && isExpanded && (
        <ul 
          className="mt-1 space-y-1" 
          role="group"
        >
          {section.children!.map((child) => (
            <NavigationNode
              key={child.id}
              section={child}
              currentSection={currentSection}
              onSelect={onSelect}
              level={level + 1}
            />
          ))}
        </ul>
      )}
    </li>
  );
}

/**
 * Check if a section is an ancestor of the given section ID
 */
function checkIfAncestor(section: NavigationSection, targetId: string): boolean {
  if (!section.children) return false;
  
  for (const child of section.children) {
    if (child.id === targetId) return true;
    if (checkIfAncestor(child, targetId)) return true;
  }
  
  return false;
}
