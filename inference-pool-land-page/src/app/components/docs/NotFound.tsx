/**
 * NotFound Component
 * Feature: landing-page-documentation
 * 
 * 404 page for missing documentation content with suggestions.
 */

import { FileQuestion, Home, Search, ArrowRight } from 'lucide-react';
import { Button } from '@/app/components/ui/button';
import { Card } from '@/app/components/ui/card';

interface NotFoundProps {
  sectionId?: string;
  onNavigate: (sectionId: string) => void;
  onSearch?: () => void;
}

const suggestedSections = [
  { id: 'getting-started', title: 'Getting Started', description: 'Comece aqui' },
  { id: 'quick-start', title: 'Quick Start', description: 'Guia rápido' },
  { id: 'api-reference', title: 'API Reference', description: 'Referência completa' },
];

export function NotFound({ sectionId, onNavigate, onSearch }: NotFoundProps) {
  return (
    <div className="min-h-[500px] flex items-center justify-center p-8">
      <div className="max-w-lg w-full text-center">
        {/* Icon */}
        <div 
          className="w-20 h-20 rounded-full mx-auto mb-6 flex items-center justify-center"
          style={{ backgroundColor: 'rgba(60, 53, 242, 0.1)' }}
        >
          <FileQuestion className="w-10 h-10" style={{ color: '#3C35F2' }} />
        </div>

        {/* Title */}
        <h1 className="text-3xl font-bold text-gray-900 mb-3">
          Página não encontrada
        </h1>

        {/* Description */}
        <p className="text-gray-600 mb-2">
          {sectionId ? (
            <>
              A seção <code className="px-2 py-1 bg-gray-100 rounded text-sm">{sectionId}</code> não existe.
            </>
          ) : (
            'A página que você está procurando não existe ou foi movida.'
          )}
        </p>
        <p className="text-gray-500 text-sm mb-8">
          Verifique o endereço ou navegue para uma das seções abaixo.
        </p>

        {/* Actions */}
        <div className="flex gap-3 justify-center mb-8">
          <Button
            variant="outline"
            onClick={() => onNavigate('getting-started')}
            className="flex items-center gap-2"
          >
            <Home className="w-4 h-4" />
            Início
          </Button>
          {onSearch && (
            <Button
              onClick={onSearch}
              style={{ backgroundColor: '#3C35F2', color: 'white' }}
              className="flex items-center gap-2"
            >
              <Search className="w-4 h-4" />
              Buscar
            </Button>
          )}
        </div>

        {/* Suggested sections */}
        <div className="text-left">
          <h3 className="text-sm font-medium text-gray-500 mb-3">
            Seções populares
          </h3>
          <div className="space-y-2">
            {suggestedSections.map(section => (
              <Card
                key={section.id}
                className="p-4 hover:border-[#3C35F2] transition-colors cursor-pointer group"
                onClick={() => onNavigate(section.id)}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <h4 className="font-medium text-gray-900 group-hover:text-[#3C35F2]">
                      {section.title}
                    </h4>
                    <p className="text-sm text-gray-500">{section.description}</p>
                  </div>
                  <ArrowRight className="w-5 h-5 text-gray-400 group-hover:text-[#3C35F2] transition-colors" />
                </div>
              </Card>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
