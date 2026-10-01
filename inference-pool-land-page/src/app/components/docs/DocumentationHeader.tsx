/**
 * DocumentationHeader Component
 * Feature: landing-page-documentation
 * 
 * Header component for documentation pages, consistent with landing page branding
 * but adapted for the documentation context.
 */

import { ArrowLeft, Search, ExternalLink } from 'lucide-react';
import { Button } from '@/app/components/ui/button';
import { Input } from '@/app/components/ui/input';

interface DocumentationHeaderProps {
  onNavigate: (page: string) => void;
  searchValue?: string;
  onSearchChange?: (value: string) => void;
  onSearchFocus?: () => void;
}

export function DocumentationHeader({
  onNavigate,
  searchValue = '',
  onSearchChange,
  onSearchFocus
}: DocumentationHeaderProps) {
  return (
    <header className="sticky top-0 z-50 bg-white border-b border-gray-200">
      <div className="max-w-screen-2xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Left section: Logo and back button */}
          <div className="flex items-center gap-4">
            <button
              onClick={() => onNavigate('landing')}
              className="flex items-center gap-2 text-gray-600 hover:text-gray-900 transition-colors"
              aria-label="Back to home"
            >
              <ArrowLeft className="w-4 h-4" />
              <span className="hidden sm:inline">Home</span>
            </button>
            
            <div className="h-6 w-px bg-gray-300" aria-hidden="true" />
            
            {/* Logo */}
            <div 
              className="flex items-center gap-2 cursor-pointer"
              onClick={() => onNavigate('documentation')}
            >
              <div 
                className="w-8 h-8 rounded-lg flex items-center justify-center"
                style={{ backgroundColor: '#3C35F2' }}
              >
                <div 
                  className="w-5 h-5 rounded"
                  style={{ backgroundColor: '#9BF2CA' }}
                />
              </div>
              <span className="font-bold text-lg text-gray-900">
                Docs
              </span>
            </div>
          </div>

          {/* Center section: Search */}
          <div className="flex-1 max-w-md mx-4 hidden md:block">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
              <Input
                type="search"
                placeholder="Buscar na documentação..."
                value={searchValue}
                onChange={(e) => onSearchChange?.(e.target.value)}
                onFocus={onSearchFocus}
                className="pl-10 w-full bg-gray-50 border-gray-200 focus:bg-white"
                aria-label="Search documentation"
              />
              <kbd className="absolute right-3 top-1/2 transform -translate-y-1/2 hidden lg:inline-flex items-center gap-1 px-2 py-0.5 text-xs text-gray-400 bg-gray-100 rounded">
                ⌘K
              </kbd>
            </div>
          </div>

          {/* Right section: Actions */}
          <div className="flex items-center gap-2 sm:gap-4">
            {/* Mobile search button */}
            <Button
              variant="ghost"
              size="icon"
              className="md:hidden"
              aria-label="Search"
              onClick={onSearchFocus}
            >
              <Search className="w-5 h-5" />
            </Button>

            {/* GitHub link */}
            <Button
              variant="ghost"
              size="sm"
              className="hidden sm:flex items-center gap-2 text-gray-600 hover:text-gray-900"
              asChild
            >
              <a 
                href="https://github.com/inferencepool" 
                target="_blank" 
                rel="noopener noreferrer"
              >
                <ExternalLink className="w-4 h-4" />
                <span className="hidden lg:inline">GitHub</span>
              </a>
            </Button>

            {/* CTA Button */}
            <Button
              onClick={() => onNavigate('signup')}
              style={{ backgroundColor: '#3C35F2', color: 'white' }}
              className="hover:opacity-90"
            >
              <span className="hidden sm:inline">Começar Agora</span>
              <span className="sm:hidden">Iniciar</span>
            </Button>
          </div>
        </div>
      </div>
    </header>
  );
}
