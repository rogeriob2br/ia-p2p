/**
 * MobileMenu Component
 * Feature: landing-page-documentation
 * 
 * Slide-in mobile menu with overlay, touch gestures, and smooth animations.
 * Requirements: 1.5, 7.3
 */

import { useEffect, useRef, useCallback } from 'react';
import { X, Search } from 'lucide-react';
import { Button } from '@/app/components/ui/button';
import { Input } from '@/app/components/ui/input';
import type { NavigationSection } from '@/app/types';
import { NavigationTree } from './NavigationTree';
import { cn } from '@/app/components/ui/utils';

interface MobileMenuProps {
  isOpen: boolean;
  onClose: () => void;
  sections: NavigationSection[];
  currentSection: string;
  onSectionChange: (sectionId: string) => void;
  searchValue?: string;
  onSearchChange?: (value: string) => void;
}

export function MobileMenu({
  isOpen,
  onClose,
  sections,
  currentSection,
  onSectionChange,
  searchValue = '',
  onSearchChange
}: MobileMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);
  const startXRef = useRef<number>(0);
  const currentXRef = useRef<number>(0);

  // Handle escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isOpen, onClose]);

  // Prevent body scroll when menu is open
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => {
      document.body.style.overflow = '';
    };
  }, [isOpen]);

  // Touch gesture handlers for swipe-to-close
  const handleTouchStart = useCallback((e: React.TouchEvent) => {
    startXRef.current = e.touches[0].clientX;
    currentXRef.current = e.touches[0].clientX;
  }, []);

  const handleTouchMove = useCallback((e: React.TouchEvent) => {
    currentXRef.current = e.touches[0].clientX;
    const diff = startXRef.current - currentXRef.current;
    
    // Only allow left swipe (closing gesture)
    if (diff > 0 && menuRef.current) {
      const translateX = Math.min(diff, 300);
      menuRef.current.style.transform = `translateX(-${translateX}px)`;
    }
  }, []);

  const handleTouchEnd = useCallback(() => {
    const diff = startXRef.current - currentXRef.current;
    
    // Close if swiped more than 100px
    if (diff > 100) {
      onClose();
    }
    
    // Reset transform
    if (menuRef.current) {
      menuRef.current.style.transform = '';
    }
  }, [onClose]);

  // Focus trap
  useEffect(() => {
    if (!isOpen) return;

    const menu = menuRef.current;
    if (!menu) return;

    const focusableElements = menu.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    const firstElement = focusableElements[0] as HTMLElement;
    const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement;

    const handleTabKey = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return;

      if (e.shiftKey) {
        if (document.activeElement === firstElement) {
          e.preventDefault();
          lastElement.focus();
        }
      } else {
        if (document.activeElement === lastElement) {
          e.preventDefault();
          firstElement.focus();
        }
      }
    };

    document.addEventListener('keydown', handleTabKey);
    firstElement?.focus();

    return () => document.removeEventListener('keydown', handleTabKey);
  }, [isOpen]);

  return (
    <>
      {/* Overlay */}
      <div
        className={cn(
          'fixed inset-0 bg-black/50 z-40 transition-opacity duration-300',
          isOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'
        )}
        onClick={onClose}
        aria-hidden="true"
      />

      {/* Menu panel */}
      <div
        ref={menuRef}
        className={cn(
          'fixed inset-y-0 left-0 z-50 w-80 max-w-[85vw] bg-white shadow-xl',
          'transform transition-transform duration-300 ease-out',
          isOpen ? 'translate-x-0' : '-translate-x-full'
        )}
        role="dialog"
        aria-modal="true"
        aria-label="Navigation menu"
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200">
          <div className="flex items-center gap-2">
            <div 
              className="w-8 h-8 rounded-lg flex items-center justify-center"
              style={{ backgroundColor: '#3C35F2' }}
            >
              <div 
                className="w-5 h-5 rounded"
                style={{ backgroundColor: '#9BF2CA' }}
              />
            </div>
            <span className="font-bold text-gray-900">Docs</span>
          </div>
          
          <Button
            variant="ghost"
            size="icon"
            onClick={onClose}
            aria-label="Close menu"
          >
            <X className="w-5 h-5" />
          </Button>
        </div>

        {/* Search */}
        <div className="p-4 border-b border-gray-200">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
            <Input
              type="search"
              placeholder="Buscar..."
              value={searchValue}
              onChange={(e) => onSearchChange?.(e.target.value)}
              className="pl-10 w-full"
              aria-label="Search documentation"
            />
          </div>
        </div>

        {/* Navigation */}
        <div className="flex-1 overflow-y-auto py-4">
          <NavigationTree
            sections={sections}
            currentSection={currentSection}
            onSectionChange={onSectionChange}
            onCloseSidebar={onClose}
          />
        </div>

        {/* Footer hint */}
        <div className="p-4 border-t border-gray-200">
          <p className="text-xs text-gray-500 text-center">
            Deslize para a esquerda para fechar
          </p>
        </div>
      </div>
    </>
  );
}
