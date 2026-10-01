/**
 * SearchModal Component
 * Feature: landing-page-documentation
 * 
 * Full-screen search modal for mobile devices and quick access.
 * Provides enhanced search experience with larger touch targets.
 */

import { useEffect, useRef, useCallback } from 'react';
import { Search, X, FileText, ArrowRight, Command } from 'lucide-react';
import { Input } from '@/app/components/ui/input';
import { Button } from '@/app/components/ui/button';
import { useSearch } from '@/app/hooks/useSearch';
import { cn } from '@/app/components/ui/utils';

interface SearchModalProps {
  isOpen: boolean;
  onClose: () => void;
  onResultSelect: (sectionId: string) => void;
}

export function SearchModal({
  isOpen,
  onClose,
  onResultSelect
}: SearchModalProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  
  const {
    query,
    setQuery,
    results,
    isSearching,
    selectedIndex,
    selectPrevious,
    selectNext,
    selectedResult,
    reset
  } = useSearch();

  // Focus input when modal opens
  useEffect(() => {
    if (isOpen) {
      setTimeout(() => inputRef.current?.focus(), 100);
    } else {
      reset();
    }
  }, [isOpen, reset]);

  // Prevent body scroll when open
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

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        selectNext();
        break;
      case 'ArrowUp':
        e.preventDefault();
        selectPrevious();
        break;
      case 'Enter':
        e.preventDefault();
        if (selectedResult) {
          onResultSelect(selectedResult.sectionId);
          onClose();
        }
        break;
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
    }
  }, [selectNext, selectPrevious, selectedResult, onResultSelect, onClose]);

  // Handle result click
  const handleResultClick = useCallback((sectionId: string) => {
    onResultSelect(sectionId);
    onClose();
  }, [onResultSelect, onClose]);

  // Highlight matching text
  const highlightMatch = (text: string, query: string) => {
    if (!query) return text;
    
    const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
    const parts = text.split(regex);
    
    return parts.map((part, i) => 
      regex.test(part) ? (
        <mark key={i} className="bg-[#9BF2CA]/40 text-inherit rounded px-0.5">
          {part}
        </mark>
      ) : part
    );
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50">
      {/* Backdrop */}
      <div 
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div 
        className={cn(
          'absolute top-4 left-4 right-4 md:left-1/2 md:-translate-x-1/2 md:w-full md:max-w-2xl',
          'bg-white rounded-xl shadow-2xl overflow-hidden',
          'max-h-[80vh] flex flex-col'
        )}
        onKeyDown={handleKeyDown}
      >
        {/* Header with search input */}
        <div className="flex items-center gap-3 p-4 border-b border-gray-200">
          <Search className="w-5 h-5 text-gray-400 flex-shrink-0" />
          <Input
            ref={inputRef}
            type="search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Buscar documentação..."
            className="flex-1 border-0 shadow-none focus:ring-0 text-lg p-0"
            autoComplete="off"
          />
          <Button
            variant="ghost"
            size="sm"
            onClick={onClose}
            className="flex-shrink-0"
          >
            <X className="w-5 h-5" />
          </Button>
        </div>

        {/* Results */}
        <div className="flex-1 overflow-y-auto">
          {/* Initial state */}
          {!isSearching && (
            <div className="p-8 text-center">
              <Command className="w-12 h-12 text-gray-200 mx-auto mb-4" />
              <p className="text-gray-500">
                Digite para buscar na documentação
              </p>
              <div className="flex flex-wrap justify-center gap-2 mt-4">
                <span className="px-2 py-1 bg-gray-100 rounded text-sm text-gray-600">
                  Getting Started
                </span>
                <span className="px-2 py-1 bg-gray-100 rounded text-sm text-gray-600">
                  API Reference
                </span>
                <span className="px-2 py-1 bg-gray-100 rounded text-sm text-gray-600">
                  Authentication
                </span>
              </div>
            </div>
          )}

          {/* No results */}
          {isSearching && results.length === 0 && (
            <div className="p-8 text-center">
              <Search className="w-12 h-12 text-gray-200 mx-auto mb-4" />
              <p className="text-gray-500">
                Nenhum resultado para "<span className="font-medium">{query}</span>"
              </p>
            </div>
          )}

          {/* Results list */}
          {results.length > 0 && (
            <ul className="py-2">
              {results.map((result, index) => (
                <li key={result.sectionId}>
                  <button
                    onClick={() => handleResultClick(result.sectionId)}
                    className={cn(
                      'w-full text-left px-4 py-4 flex items-start gap-4',
                      'transition-colors cursor-pointer',
                      index === selectedIndex
                        ? 'bg-[#3C35F2]/5 border-l-4 border-[#3C35F2]'
                        : 'hover:bg-gray-50 border-l-4 border-transparent'
                    )}
                  >
                    <FileText className={cn(
                      'w-6 h-6 mt-0.5 flex-shrink-0',
                      index === selectedIndex ? 'text-[#3C35F2]' : 'text-gray-400'
                    )} />
                    
                    <div className="flex-1 min-w-0">
                      <div className="font-medium text-gray-900 text-lg">
                        {highlightMatch(result.title, query)}
                      </div>
                      <div className="text-sm text-gray-500 mt-1 line-clamp-2">
                        {highlightMatch(result.excerpt, query)}
                      </div>
                    </div>

                    {index === selectedIndex && (
                      <ArrowRight className="w-5 h-5 text-[#3C35F2] mt-1 flex-shrink-0" />
                    )}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>

        {/* Footer with keyboard hints */}
        <div className="px-4 py-3 border-t border-gray-200 bg-gray-50 flex items-center justify-between text-xs text-gray-500">
          <div className="flex items-center gap-4">
            <span className="flex items-center gap-1">
              <kbd className="px-1.5 py-0.5 bg-white rounded border border-gray-200">↑</kbd>
              <kbd className="px-1.5 py-0.5 bg-white rounded border border-gray-200">↓</kbd>
              navegar
            </span>
            <span className="flex items-center gap-1">
              <kbd className="px-1.5 py-0.5 bg-white rounded border border-gray-200">Enter</kbd>
              ir
            </span>
          </div>
          <span className="flex items-center gap-1">
            <kbd className="px-1.5 py-0.5 bg-white rounded border border-gray-200">Esc</kbd>
            fechar
          </span>
        </div>
      </div>
    </div>
  );
}
