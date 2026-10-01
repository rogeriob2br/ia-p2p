/**
 * Documentation Page Component
 * Feature: landing-page-documentation
 * 
 * Main documentation page that integrates all documentation components.
 * Uses the three-column responsive layout with sidebar navigation,
 * main content area, and table of contents.
 */

import { useState, useCallback } from 'react';
import {
  DocumentationLayout,
  DocumentationHeader,
  NavigationTree,
  SearchModal,
  TableOfContents,
  Breadcrumbs,
  PageNavigation,
} from '@/app/components/docs';
import { Footer } from '@/app/components/Footer';
import { Card } from '@/app/components/ui/card';
import { useDocumentation } from '@/app/hooks';
import { 
  Zap, 
  Copy, 
  Check,
  Terminal,
  Code
} from 'lucide-react';

interface DocumentationProps {
  onNavigate: (page: string) => void;
}

export function Documentation({ onNavigate }: DocumentationProps) {
  const [copiedCode, setCopiedCode] = useState<string | null>(null);
  const [isSearchModalOpen, setIsSearchModalOpen] = useState(false);

  // Use the documentation hook for state management
  const {
    currentSection,
    sectionData,
    headings,
    breadcrumbs,
    previousSection,
    nextSection,
    searchQuery,
    setSearchQuery,
    activeHeading,
    setActiveHeading,
    navigateToSection,
    navigation,
  } = useDocumentation();

  // Copy to clipboard function
  const copyToClipboard = useCallback((text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedCode(id);
    setTimeout(() => setCopiedCode(null), 2000);
  }, []);

  // Handle search result selection
  const handleSearchResultSelect = useCallback((sectionId: string) => {
    navigateToSection(sectionId);
    setIsSearchModalOpen(false);
  }, [navigateToSection]);

  // Get section title
  const sectionTitle = sectionData?.title || 'Documentation';

  // Code examples
  const codeExample1 = `npm install @inference-pool/sdk`;
  const codeExample2 = `import { InferencePool } from '@inference-pool/sdk';

const pool = new InferencePool({
  apiKey: process.env.INFERENCE_POOL_API_KEY,
  network: 'mainnet'
});

// Make an inference request
const response = await pool.complete({
  model: 'gpt-4',
  messages: [
    { role: 'user', content: 'Hello, world!' }
  ]
});

console.log(response.choices[0].message.content);`;

  const codeExample3 = `const pool = new InferencePool({
  apiKey: 'sk-pool-your-api-key-here',
  network: 'mainnet',
  routing: 'weighted_score', // or 'round_robin', 'least_loaded'
  timeout: 120000 // 2 minutes
});`;

  return (
    <>
      <DocumentationLayout
        header={
          <DocumentationHeader
            onNavigate={onNavigate}
            searchValue={searchQuery}
            onSearchChange={setSearchQuery}
            onSearchFocus={() => setIsSearchModalOpen(true)}
          />
        }
        sidebar={
          <NavigationTree
            sections={navigation}
            currentSection={currentSection}
            onSectionChange={navigateToSection}
          />
        }
        tableOfContents={
          <TableOfContents
            headings={headings}
            activeHeading={activeHeading}
            onHeadingClick={setActiveHeading}
          />
        }
        footer={<Footer />}
      >
        {/* Breadcrumbs */}
        <Breadcrumbs 
          items={breadcrumbs} 
          onNavigate={navigateToSection} 
        />

        {/* Main Content */}
        <article className="prose prose-gray max-w-none">
          {currentSection === 'getting-started' && (
            <div className="space-y-8">
              <div>
                <h1 
                  id="getting-started" 
                  className="text-4xl font-bold mb-4"
                  style={{ color: '#0D0D0D' }}
                >
                  Getting Started
                </h1>
                <p className="text-lg text-gray-600">
                  Bem-vindo à documentação do InferencePool. Este guia vai te ajudar a começar em minutos.
                </p>
              </div>

              <Card className="p-6 border-l-4" style={{ borderLeftColor: '#3C35F2' }}>
                <div className="flex items-start gap-3">
                  <Zap className="w-6 h-6 flex-shrink-0 mt-1" style={{ color: '#9BF2CA' }} />
                  <div>
                    <h3 className="font-bold mb-2">Quick Start</h3>
                    <p className="text-gray-600">
                      Comece com 1M tokens grátis. Sem cartão de crédito necessário.
                    </p>
                  </div>
                </div>
              </Card>

              <div>
                <h2 id="instalação" className="text-2xl font-bold mb-4">1. Instalação</h2>
                <p className="text-gray-600 mb-4">
                  Instale o SDK usando npm ou yarn:
                </p>
                <div className="relative">
                  <Card className="p-4 bg-gray-900 rounded-lg">
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <Terminal className="w-4 h-4 text-gray-400" />
                        <span className="text-sm text-gray-400">Terminal</span>
                      </div>
                      <button
                        onClick={() => copyToClipboard(codeExample1, 'code1')}
                        className="p-2 hover:bg-gray-800 rounded transition-colors"
                        aria-label="Copy code"
                      >
                        {copiedCode === 'code1' ? (
                          <Check className="w-4 h-4 text-green-400" />
                        ) : (
                          <Copy className="w-4 h-4 text-gray-400" />
                        )}
                      </button>
                    </div>
                    <pre className="text-sm overflow-x-auto">
                      <code style={{ color: '#9BF2CA' }}>{codeExample1}</code>
                    </pre>
                  </Card>
                </div>
              </div>

              <div>
                <h2 id="configuração-básica" className="text-2xl font-bold mb-4">2. Configuração Básica</h2>
                <p className="text-gray-600 mb-4">
                  Configure sua API key e faça sua primeira requisição:
                </p>
                <div className="relative">
                  <Card className="p-4 bg-gray-900 rounded-lg">
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <Code className="w-4 h-4 text-gray-400" />
                        <span className="text-sm text-gray-400">JavaScript</span>
                      </div>
                      <button
                        onClick={() => copyToClipboard(codeExample2, 'code2')}
                        className="p-2 hover:bg-gray-800 rounded transition-colors"
                        aria-label="Copy code"
                      >
                        {copiedCode === 'code2' ? (
                          <Check className="w-4 h-4 text-green-400" />
                        ) : (
                          <Copy className="w-4 h-4 text-gray-400" />
                        )}
                      </button>
                    </div>
                    <pre className="text-sm overflow-x-auto">
                      <code className="text-white">{codeExample2}</code>
                    </pre>
                  </Card>
                </div>
              </div>

              <div>
                <h2 id="opções-avançadas" className="text-2xl font-bold mb-4">3. Opções Avançadas</h2>
                <p className="text-gray-600 mb-4">
                  Configure estratégias de roteamento e timeouts:
                </p>
                <div className="relative">
                  <Card className="p-4 bg-gray-900 rounded-lg">
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center gap-2">
                        <Code className="w-4 h-4 text-gray-400" />
                        <span className="text-sm text-gray-400">JavaScript</span>
                      </div>
                      <button
                        onClick={() => copyToClipboard(codeExample3, 'code3')}
                        className="p-2 hover:bg-gray-800 rounded transition-colors"
                        aria-label="Copy code"
                      >
                        {copiedCode === 'code3' ? (
                          <Check className="w-4 h-4 text-green-400" />
                        ) : (
                          <Copy className="w-4 h-4 text-gray-400" />
                        )}
                      </button>
                    </div>
                    <pre className="text-sm overflow-x-auto">
                      <code className="text-white">{codeExample3}</code>
                    </pre>
                  </Card>
                </div>
              </div>

              <Card className="p-6 bg-blue-50 border-blue-200">
                <h3 className="font-bold mb-2" style={{ color: '#3C35F2' }}>Próximos Passos</h3>
                <ul className="space-y-2 text-gray-700">
                  <li className="flex items-start gap-2">
                    <span style={{ color: '#3C35F2' }}>→</span>
                    <button 
                      onClick={() => navigateToSection('authentication')}
                      className="text-left hover:text-[#3C35F2] transition-colors"
                    >
                      Explore a seção de Authentication para proteger suas requisições
                    </button>
                  </li>
                  <li className="flex items-start gap-2">
                    <span style={{ color: '#3C35F2' }}>→</span>
                    <button 
                      onClick={() => navigateToSection('api-reference')}
                      className="text-left hover:text-[#3C35F2] transition-colors"
                    >
                      Veja a API Reference completa para todas as opções disponíveis
                    </button>
                  </li>
                  <li className="flex items-start gap-2">
                    <span style={{ color: '#3C35F2' }}>→</span>
                    <button 
                      onClick={() => navigateToSection('lightning-payments')}
                      className="text-left hover:text-[#3C35F2] transition-colors"
                    >
                      Configure Lightning Network para pagamentos instantâneos
                    </button>
                  </li>
                </ul>
              </Card>
            </div>
          )}

          {currentSection === 'installation' && (
            <div className="space-y-6">
              <h1 id="installation" className="text-4xl font-bold" style={{ color: '#0D0D0D' }}>
                Installation
              </h1>
              <p className="text-lg text-gray-600">
                Instale o SDK do InferencePool usando seu gerenciador de pacotes preferido.
              </p>
              <Card className="p-6">
                <h3 className="text-xl font-bold mb-4">npm</h3>
                <Card className="p-4 bg-gray-900 rounded-lg">
                  <code style={{ color: '#9BF2CA' }}>npm install @inference-pool/sdk</code>
                </Card>
              </Card>
              <Card className="p-6">
                <h3 className="text-xl font-bold mb-4">yarn</h3>
                <Card className="p-4 bg-gray-900 rounded-lg">
                  <code style={{ color: '#9BF2CA' }}>yarn add @inference-pool/sdk</code>
                </Card>
              </Card>
              <Card className="p-6">
                <h3 className="text-xl font-bold mb-4">pnpm</h3>
                <Card className="p-4 bg-gray-900 rounded-lg">
                  <code style={{ color: '#9BF2CA' }}>pnpm add @inference-pool/sdk</code>
                </Card>
              </Card>
            </div>
          )}

          {currentSection === 'quick-start' && (
            <div className="space-y-6">
              <h1 id="quick-start" className="text-4xl font-bold" style={{ color: '#0D0D0D' }}>
                Quick Start
              </h1>
              <p className="text-lg text-gray-600">
                Comece a usar o InferencePool em menos de 5 minutos.
              </p>
              
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <Card className="p-6 text-center">
                  <div className="text-4xl font-bold mb-2" style={{ color: '#3C35F2' }}>1</div>
                  <h3 className="font-bold mb-2">Criar conta</h3>
                  <p className="text-sm text-gray-600">Cadastre-se gratuitamente</p>
                </Card>
                <Card className="p-6 text-center">
                  <div className="text-4xl font-bold mb-2" style={{ color: '#3C35F2' }}>2</div>
                  <h3 className="font-bold mb-2">Obter API Key</h3>
                  <p className="text-sm text-gray-600">Gere sua chave no dashboard</p>
                </Card>
                <Card className="p-6 text-center">
                  <div className="text-4xl font-bold mb-2" style={{ color: '#3C35F2' }}>3</div>
                  <h3 className="font-bold mb-2">Fazer requisição</h3>
                  <p className="text-sm text-gray-600">Use o SDK ou REST API</p>
                </Card>
              </div>

              <Card className="p-4 bg-gray-900 rounded-lg">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <Code className="w-4 h-4 text-gray-400" />
                    <span className="text-sm text-gray-400">TypeScript</span>
                  </div>
                  <button
                    onClick={() => copyToClipboard(codeExample2, 'quick-code')}
                    className="p-2 hover:bg-gray-800 rounded transition-colors"
                    aria-label="Copy code"
                  >
                    {copiedCode === 'quick-code' ? (
                      <Check className="w-4 h-4 text-green-400" />
                    ) : (
                      <Copy className="w-4 h-4 text-gray-400" />
                    )}
                  </button>
                </div>
                <pre className="text-sm overflow-x-auto">
                  <code className="text-white">{codeExample2}</code>
                </pre>
              </Card>
            </div>
          )}

          {currentSection === 'authentication' && (
            <div className="space-y-6">
              <h1 id="authentication" className="text-4xl font-bold" style={{ color: '#0D0D0D' }}>
                Authentication
              </h1>
              <p className="text-lg text-gray-600">
                Aprenda como autenticar suas requisições usando API keys.
              </p>
              
              <Card className="p-6 border-l-4" style={{ borderLeftColor: '#9BF2CA' }}>
                <div className="flex items-start gap-3">
                  <Zap className="w-6 h-6 flex-shrink-0 mt-1" style={{ color: '#3C35F2' }} />
                  <div>
                    <h3 className="font-bold mb-2">Segurança</h3>
                    <p className="text-gray-600">
                      Nunca exponha sua API key em código client-side ou repositórios públicos.
                    </p>
                  </div>
                </div>
              </Card>

              <Card className="p-6">
                <h3 className="text-xl font-bold mb-4">Obtendo uma API Key</h3>
                <ol className="list-decimal list-inside space-y-2 text-gray-600">
                  <li>Faça login no dashboard do InferencePool</li>
                  <li>Navegue até Settings → API Keys</li>
                  <li>Clique em "Create New Key"</li>
                  <li>Copie e armazene sua chave com segurança</li>
                </ol>
              </Card>

              <Card className="p-6">
                <h3 className="text-xl font-bold mb-4">Usando sua API Key</h3>
                <p className="text-gray-600 mb-4">
                  Adicione o header de autorização em todas as requisições:
                </p>
                <div className="bg-gray-100 p-4 rounded-lg font-mono text-sm">
                  Authorization: Bearer sk-pool-your-api-key-here
                </div>
              </Card>
            </div>
          )}

          {currentSection !== 'getting-started' && 
           currentSection !== 'installation' && 
           currentSection !== 'quick-start' &&
           currentSection !== 'authentication' && (
            <div className="space-y-6">
              <h1 id={currentSection} className="text-4xl font-bold" style={{ color: '#0D0D0D' }}>
                {sectionTitle}
              </h1>
              <p className="text-lg text-gray-600">
                Conteúdo para esta seção será adicionado em breve.
              </p>
              <Card className="p-6 bg-amber-50 border-amber-200">
                <p className="text-amber-800">
                  📝 Esta seção está em desenvolvimento. Volte em breve para mais conteúdo!
                </p>
              </Card>
            </div>
          )}
        </article>

        {/* Page Navigation */}
        <PageNavigation
          previous={previousSection}
          next={nextSection}
          onNavigate={navigateToSection}
        />
      </DocumentationLayout>

      {/* Search Modal */}
      <SearchModal
        isOpen={isSearchModalOpen}
        onClose={() => setIsSearchModalOpen(false)}
        onResultSelect={handleSearchResultSelect}
      />
    </>
  );
}
