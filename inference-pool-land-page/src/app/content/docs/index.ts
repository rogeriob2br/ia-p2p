/**
 * Documentation content index
 * Feature: landing-page-documentation
 * 
 * This file provides access to all documentation Markdown content.
 */

import type { DocumentationMetadata } from '@/app/types';
import {
  gettingStartedContent,
  installationContent,
  quickStartContent,
  authenticationContent
} from './getting-started.md';
import {
  p2pOrchestrationContent,
  lightningPaymentsContent,
  routingStrategiesContent
} from './core-concepts.md';
import {
  clientContent,
  methodsContent,
  configurationContent,
  errorsContent
} from './api-reference.md';

/**
 * All documentation content mapped by section ID
 */
export const documentationContent: Record<string, string> = {
  // Getting Started
  'getting-started': gettingStartedContent,
  'installation': installationContent,
  'quick-start': quickStartContent,
  'authentication': authenticationContent,
  
  // Core Concepts
  'core-concepts': p2pOrchestrationContent,
  'p2p-orchestration': p2pOrchestrationContent,
  'lightning-payments': lightningPaymentsContent,
  'routing-strategies': routingStrategiesContent,
  
  // API Reference
  'api-reference': clientContent,
  'client': clientContent,
  'methods': methodsContent,
  'configuration': configurationContent,
  'errors': errorsContent,
};

/**
 * Documentation metadata
 */
export const documentationMetadata: DocumentationMetadata = {
  version: '0.1.0',
  lastUpdated: new Date().toISOString(),
  sections: [
    // Getting Started
    {
      id: 'getting-started',
      title: 'Getting Started',
      description: 'Begin your journey with InferencePool',
      keywords: ['start', 'begin', 'intro', 'setup', 'guide']
    },
    {
      id: 'installation',
      title: 'Installation',
      description: 'Install the InferencePool SDK',
      keywords: ['install', 'npm', 'yarn', 'pnpm', 'package', 'setup']
    },
    {
      id: 'quick-start',
      title: 'Quick Start',
      description: 'Get up and running quickly',
      keywords: ['quick', 'start', 'example', 'code', 'tutorial']
    },
    {
      id: 'authentication',
      title: 'Authentication',
      description: 'Secure your API requests',
      keywords: ['auth', 'key', 'api', 'security', 'token', 'bearer']
    },
    
    // Core Concepts
    {
      id: 'p2p-orchestration',
      title: 'P2P Orchestration',
      description: 'Learn about decentralized inference',
      keywords: ['p2p', 'peer', 'network', 'decentralized', 'orchestration']
    },
    {
      id: 'lightning-payments',
      title: 'Lightning Payments',
      description: 'Instant Bitcoin payments',
      keywords: ['lightning', 'bitcoin', 'payment', 'sats', 'invoice']
    },
    {
      id: 'routing-strategies',
      title: 'Routing Strategies',
      description: 'Choose how requests are routed',
      keywords: ['routing', 'strategy', 'provider', 'selection', 'latency', 'price']
    },
    
    // API Reference
    {
      id: 'client',
      title: 'InferencePool Client',
      description: 'Main SDK client class',
      keywords: ['client', 'sdk', 'constructor', 'config', 'initialize']
    },
    {
      id: 'methods',
      title: 'Methods',
      description: 'All available SDK methods',
      keywords: ['methods', 'complete', 'stream', 'embed', 'function', 'api']
    },
    {
      id: 'configuration',
      title: 'Configuration',
      description: 'Advanced configuration options',
      keywords: ['config', 'options', 'settings', 'environment', 'typescript']
    },
    {
      id: 'errors',
      title: 'Error Handling',
      description: 'Handle errors gracefully',
      keywords: ['error', 'exception', 'catch', 'retry', 'handling']
    }
  ]
};

/**
 * Get content for a specific section
 */
export function getSectionContent(sectionId: string): string {
  return documentationContent[sectionId] || `
# ${sectionId.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase())}

Content for this section is coming soon.

Check back later for updates!
`;
}

/**
 * Get estimated reading time for a section (words / 200 wpm)
 */
export function getEstimatedReadTime(content: string): number {
  const wordCount = content.trim().split(/\s+/).length;
  return Math.max(1, Math.ceil(wordCount / 200));
}

/**
 * Get all sections that match a search term
 */
export function searchSections(query: string): typeof documentationMetadata.sections {
  const lowerQuery = query.toLowerCase();
  return documentationMetadata.sections.filter(section => 
    section.title.toLowerCase().includes(lowerQuery) ||
    section.description.toLowerCase().includes(lowerQuery) ||
    section.keywords.some(k => k.includes(lowerQuery))
  );
}
