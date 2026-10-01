/**
 * Navigation structure for the documentation
 * Feature: landing-page-documentation
 */

import { 
  BookOpen, 
  Lightbulb, 
  Code, 
  Plug, 
  Settings, 
  HelpCircle,
  Rocket,
  Key,
  Layers,
  Zap,
  Route,
  Terminal,
  Wrench,
  BarChart3,
  Gauge
} from 'lucide-react';
import type { NavigationSection } from '@/app/types';

/**
 * Complete navigation structure for the documentation sidebar
 */
export const navigationStructure: NavigationSection[] = [
  {
    id: 'getting-started',
    title: 'Getting Started',
    icon: BookOpen,
    path: '/docs/getting-started',
    children: [
      {
        id: 'installation',
        title: 'Installation',
        path: '/docs/getting-started/installation',
        icon: Rocket
      },
      {
        id: 'quick-start',
        title: 'Quick Start',
        path: '/docs/getting-started/quick-start',
        icon: Zap
      },
      {
        id: 'authentication',
        title: 'Authentication',
        path: '/docs/getting-started/authentication',
        icon: Key
      }
    ]
  },
  {
    id: 'core-concepts',
    title: 'Core Concepts',
    icon: Lightbulb,
    path: '/docs/core-concepts',
    children: [
      {
        id: 'p2p-orchestration',
        title: 'P2P Orchestration',
        path: '/docs/core-concepts/p2p-orchestration',
        icon: Layers
      },
      {
        id: 'lightning-payments',
        title: 'Lightning Payments',
        path: '/docs/core-concepts/lightning-payments',
        icon: Zap
      },
      {
        id: 'routing-strategies',
        title: 'Routing Strategies',
        path: '/docs/core-concepts/routing-strategies',
        icon: Route
      }
    ]
  },
  {
    id: 'api-reference',
    title: 'API Reference',
    icon: Code,
    path: '/docs/api-reference',
    children: [
      {
        id: 'client',
        title: 'InferencePool Client',
        path: '/docs/api-reference/client',
        icon: Terminal
      },
      {
        id: 'methods',
        title: 'Methods',
        path: '/docs/api-reference/methods',
        icon: Code
      },
      {
        id: 'configuration',
        title: 'Configuration',
        path: '/docs/api-reference/configuration',
        icon: Settings
      },
      {
        id: 'errors',
        title: 'Error Handling',
        path: '/docs/api-reference/errors',
        icon: HelpCircle
      }
    ]
  },
  {
    id: 'integration-guides',
    title: 'Integration Guides',
    icon: Plug,
    path: '/docs/integration-guides',
    children: [
      {
        id: 'langchain',
        title: 'LangChain',
        path: '/docs/integration-guides/langchain'
      },
      {
        id: 'crewai',
        title: 'CrewAI',
        path: '/docs/integration-guides/crewai'
      },
      {
        id: 'nodejs',
        title: 'Node.js',
        path: '/docs/integration-guides/nodejs'
      }
    ]
  },
  {
    id: 'advanced',
    title: 'Advanced Topics',
    icon: Settings,
    path: '/docs/advanced',
    children: [
      {
        id: 'custom-routing',
        title: 'Custom Routing',
        path: '/docs/advanced/custom-routing',
        icon: Wrench
      },
      {
        id: 'monitoring',
        title: 'Monitoring & Analytics',
        path: '/docs/advanced/monitoring',
        icon: BarChart3
      },
      {
        id: 'performance',
        title: 'Performance Optimization',
        path: '/docs/advanced/performance',
        icon: Gauge
      }
    ]
  },
  {
    id: 'faq',
    title: 'FAQ',
    icon: HelpCircle,
    path: '/docs/faq'
  }
];

/**
 * Find a section by its ID in the navigation structure
 */
export function findSectionById(
  id: string, 
  sections: NavigationSection[] = navigationStructure
): NavigationSection | null {
  for (const section of sections) {
    if (section.id === id) {
      return section;
    }
    if (section.children) {
      const found = findSectionById(id, section.children);
      if (found) return found;
    }
  }
  return null;
}

/**
 * Get the breadcrumb path for a given section ID
 */
export function getBreadcrumbPath(
  id: string, 
  sections: NavigationSection[] = navigationStructure,
  path: NavigationSection[] = []
): NavigationSection[] | null {
  for (const section of sections) {
    const currentPath = [...path, section];
    
    if (section.id === id) {
      return currentPath;
    }
    
    if (section.children) {
      const found = getBreadcrumbPath(id, section.children, currentPath);
      if (found) return found;
    }
  }
  return null;
}

/**
 * Get the previous and next sections for navigation
 */
export function getPrevNextSections(id: string): {
  previous: NavigationSection | null;
  next: NavigationSection | null;
} {
  const flatList = flattenNavigation(navigationStructure);
  const currentIndex = flatList.findIndex(s => s.id === id);
  
  return {
    previous: currentIndex > 0 ? flatList[currentIndex - 1] : null,
    next: currentIndex < flatList.length - 1 ? flatList[currentIndex + 1] : null
  };
}

/**
 * Flatten the navigation structure into a single array
 */
export function flattenNavigation(
  sections: NavigationSection[]
): NavigationSection[] {
  const result: NavigationSection[] = [];
  
  for (const section of sections) {
    result.push(section);
    if (section.children) {
      result.push(...flattenNavigation(section.children));
    }
  }
  
  return result;
}

/**
 * Get all section IDs for search indexing
 */
export function getAllSectionIds(): string[] {
  return flattenNavigation(navigationStructure).map(s => s.id);
}

