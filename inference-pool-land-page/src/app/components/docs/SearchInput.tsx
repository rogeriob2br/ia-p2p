/**
 * SearchInput Component
 * Feature: landing-page-documentation
 * 
 * Search input with real-time results dropdown, keyboard navigation,
 * and result highlighting.
 * 
 * Requirements: 6.1, 6.2, 6.3, 6.4, 6.5
 */

import { useState, useRef, useEffect, useCallback } from 'react';
import { Search, X, FileText, ArrowRight } from 'lucide-react';
import { Input } from '@/app/components/ui/input';
import { useSearch } from '@/app/hooks/useSearch';
import { cn } from '@/app/components/ui/utils';

interface SearchInputProps {
  /** Called when a search result is selected */
  onResultSelect: (sectionId: string) => void;
  /** Placeholder text */
  placeholder?: string;
  /** Additional class names */
  className?: string;
  /** Whether to show the keyboard shortcut hint */
  showShortcut?: boolean;
}

export function SearchInput({
  onResultSelect,
  placeholder = 'Buscar na documentação...',
  className,
  showShortcut = true
}: SearchInputProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [isFocused, setIsFocused] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

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

  // Show dropdown when searching
  useEffect(() => {
    setIsOpen(isSearching && results.length > 0);
  }, [isSearching, results.length]);

  // Keyboard shortcut (Cmd/Ctrl + K)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        inputRef.current?.focus();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Close dropdown on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(e.target as Node) &&
        !inputRef.current?.contains(e.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (!isOpen) return;

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
          reset();
          setIsOpen(false);
          inputRef.current?.blur();
        }
        break;
      case 'Escape':
        e.preventDefault();
        setIsOpen(false);
        inputRef.current?.blur();
        break;
    }
  }, [isOpen, selectNext, selectPrevious, selectedResult, onResultSelect, reset]);

  // Handle result click
  const handleResultClick = useCallback((sectionId: string) => {
    onResultSelect(sectionId);
    reset();
    setIsOpen(false);
  }, [onResultSelect, reset]);

  // Handle clear button
  const handleClear = useCallback(() => {
    reset();
    inputRef.current?.focus();
  }, [reset]);

  // Highlight matching text in results
  const highlightMatch = (text: string, query: string) => {
    if (!query) return text;
    
    const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
    const parts = text.split(regex);
    
    return parts.map((part, i) => 
      regex.test(part) ? (
        <mark 
          key={i} 
          className="bg-[#9BF2CA]/40 text-inherit rounded px-0.5"
        >
          {part}
        </mark>
      ) : part
    );
  };

  return (
    <div className={cn('relative', className)}>
      {/* Search Input */}
      <div className="relative">
        <Search 
          className={cn(
            'absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 transition-colors',
            isFocused ? 'text-[#3C35F2]' : 'text-gray-400'
          )} 
        />
        <Input
          ref={inputRef}
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onFocus={() => {
            setIsFocused(true);
            if (results.length > 0) setIsOpen(true);
          }}
          onBlur={() => setIsFocused(false)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className={cn(
            'pl-10 pr-20 w-full bg-gray-50 border-gray-200',
            'focus:bg-white focus:border-[#3C35F2] focus:ring-[#3C35F2]',
            'transition-all'
          )}
          aria-label="Search documentation"
          aria-expanded={isOpen}
          aria-controls="search-results"
          aria-activedescendant={selectedResult ? `result-${selectedResult.sectionId}` : undefined}
          role="combobox"
          autoComplete="off"
        />
        
        {/* Clear button or keyboard shortcut */}
        <div className="absolute right-3 top-1/2 transform -translate-y-1/2 flex items-center gap-2">
          {query && (
            <button
              onClick={handleClear}
              className="p-1 hover:bg-gray-200 rounded transition-colors"
              aria-label="Clear search"
            >
              <X className="w-4 h-4 text-gray-400" />
            </button>
          )}
          {showShortcut && !query && (
            <kbd className="hidden lg:inline-flex items-center gap-1 px-2 py-0.5 text-xs text-gray-400 bg-gray-100 rounded border border-gray-200">
              ⌘K
            </kbd>
          )}
        </div>
      </div>

      {/* Results Dropdown */}
      {isOpen && (
        <div
          ref={dropdownRef}
          id="search-results"
          role="listbox"
          className={cn(
            'absolute top-full left-0 right-0 mt-2 z-50',
            'bg-white rounded-lg shadow-lg border border-gray-200',
            'max-h-96 overflow-y-auto'
          )}
        >
          {/* Results header */}
          <div className="px-4 py-2 border-b border-gray-100">
            <span className="text-xs text-gray-500">
              {results.length} resultado{results.length !== 1 ? 's' : ''} encontrado{results.length !== 1 ? 's' : ''}
            </span>
          </div>

          {/* Results list */}
          <ul className="py-2">
            {results.map((result, index) => (
              <li
                key={result.sectionId}
                id={`result-${result.sectionId}`}
                role="option"
                aria-selected={index === selectedIndex}
              >
                <button
                  onClick={() => handleResultClick(result.sectionId)}
                  onMouseEnter={() => {/* Could update selectedIndex on hover */}}
                  className={cn(
                    'w-full text-left px-4 py-3 flex items-start gap-3',
                    'transition-colors cursor-pointer',
                    index === selectedIndex
                      ? 'bg-[#3C35F2]/5 border-l-2 border-[#3C35F2]'
                      : 'hover:bg-gray-50 border-l-2 border-transparent'
                  )}
                >
                  {/* Icon */}
                  <FileText className={cn(
                    'w-5 h-5 mt-0.5 flex-shrink-0',
                    index === selectedIndex ? 'text-[#3C35F2]' : 'text-gray-400'
                  )} />
                  
                  {/* Content */}
                  <div className="flex-1 min-w-0">
                    <div className="font-medium text-gray-900">
                      {highlightMatch(result.title, query)}
                    </div>
                    <div className="text-sm text-gray-500 mt-0.5 line-clamp-2">
                      {highlightMatch(result.excerpt, query)}
                    </div>
                  </div>

                  {/* Arrow indicator */}
                  {index === selectedIndex && (
                    <ArrowRight className="w-4 h-4 text-[#3C35F2] mt-1 flex-shrink-0" />
                  )}
                </button>
              </li>
            ))}
          </ul>

          {/* Keyboard hints */}
          <div className="px-4 py-2 border-t border-gray-100 bg-gray-50 flex items-center gap-4 text-xs text-gray-500">
            <span className="flex items-center gap-1">
              <kbd className="px-1.5 py-0.5 bg-white rounded border border-gray-200">↑</kbd>
              <kbd className="px-1.5 py-0.5 bg-white rounded border border-gray-200">↓</kbd>
              navegar
            </span>
            <span className="flex items-center gap-1">
              <kbd className="px-1.5 py-0.5 bg-white rounded border border-gray-200">Enter</kbd>
              selecionar
            </span>
            <span className="flex items-center gap-1">
              <kbd className="px-1.5 py-0.5 bg-white rounded border border-gray-200">Esc</kbd>
              fechar
            </span>
          </div>
        </div>
      )}

      {/* No results message */}
      {isSearching && results.length === 0 && isFocused && (
        <div
          className={cn(
            'absolute top-full left-0 right-0 mt-2 z-50',
            'bg-white rounded-lg shadow-lg border border-gray-200',
            'p-6 text-center'
          )}
        >
          <Search className="w-8 h-8 text-gray-300 mx-auto mb-2" />
          <p className="text-gray-500">
            Nenhum resultado para "<span className="font-medium">{query}</span>"
          </p>
          <p className="text-sm text-gray-400 mt-1">
            Tente buscar por outros termos
          </p>
        </div>
      )}
    </div>
  );
}
