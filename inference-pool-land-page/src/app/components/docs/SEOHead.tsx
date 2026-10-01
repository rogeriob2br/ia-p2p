/**
 * SEOHead Component
 * Feature: landing-page-documentation
 * 
 * Manages document head with meta tags, Open Graph, and structured data.
 */

import { useEffect } from 'react';

interface SEOHeadProps {
  title: string;
  description: string;
  section?: string;
  path?: string;
}

const BASE_URL = 'https://inferencepool.ai';
const SITE_NAME = 'InferencePool Documentation';

export function SEOHead({ title, description, section, path }: SEOHeadProps) {
  useEffect(() => {
    // Update document title
    document.title = `${title} | ${SITE_NAME}`;

    // Update or create meta tags
    updateMetaTag('description', description);
    updateMetaTag('og:title', `${title} | ${SITE_NAME}`);
    updateMetaTag('og:description', description);
    updateMetaTag('og:type', 'article');
    updateMetaTag('og:url', path ? `${BASE_URL}${path}` : BASE_URL);
    updateMetaTag('og:site_name', SITE_NAME);
    updateMetaTag('twitter:card', 'summary_large_image');
    updateMetaTag('twitter:title', title);
    updateMetaTag('twitter:description', description);

    // Add structured data for documentation
    const structuredData = {
      '@context': 'https://schema.org',
      '@type': 'TechArticle',
      headline: title,
      description: description,
      author: {
        '@type': 'Organization',
        name: 'InferencePool'
      },
      publisher: {
        '@type': 'Organization',
        name: 'InferencePool',
        logo: {
          '@type': 'ImageObject',
          url: `${BASE_URL}/logo.png`
        }
      },
      mainEntityOfPage: {
        '@type': 'WebPage',
        '@id': path ? `${BASE_URL}${path}` : BASE_URL
      }
    };

    updateStructuredData(structuredData);

    // Cleanup on unmount
    return () => {
      document.title = SITE_NAME;
    };
  }, [title, description, section, path]);

  return null;
}

/**
 * Update or create a meta tag
 */
function updateMetaTag(name: string, content: string): void {
  const isOgTag = name.startsWith('og:');
  const isTwitterTag = name.startsWith('twitter:');
  
  let meta: HTMLMetaElement | null;
  
  if (isOgTag || isTwitterTag) {
    meta = document.querySelector(`meta[property="${name}"]`);
    if (!meta) {
      meta = document.createElement('meta');
      meta.setAttribute('property', name);
      document.head.appendChild(meta);
    }
  } else {
    meta = document.querySelector(`meta[name="${name}"]`);
    if (!meta) {
      meta = document.createElement('meta');
      meta.setAttribute('name', name);
      document.head.appendChild(meta);
    }
  }
  
  meta.setAttribute('content', content);
}

/**
 * Update or create structured data script
 */
function updateStructuredData(data: object): void {
  const scriptId = 'structured-data';
  let script = document.getElementById(scriptId) as HTMLScriptElement | null;
  
  if (!script) {
    script = document.createElement('script');
    script.id = scriptId;
    script.type = 'application/ld+json';
    document.head.appendChild(script);
  }
  
  script.textContent = JSON.stringify(data);
}

/**
 * Generate breadcrumb structured data
 */
export function generateBreadcrumbSchema(
  items: { name: string; url: string }[]
): object {
  return {
    '@context': 'https://schema.org',
    '@type': 'BreadcrumbList',
    itemListElement: items.map((item, index) => ({
      '@type': 'ListItem',
      position: index + 1,
      name: item.name,
      item: item.url
    }))
  };
}
