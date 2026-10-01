/**
 * Prism.js configuration and utilities
 * Feature: landing-page-documentation
 */

import Prism from 'prismjs';

// Import additional languages
import 'prismjs/components/prism-typescript';
import 'prismjs/components/prism-javascript';
import 'prismjs/components/prism-python';
import 'prismjs/components/prism-bash';
import 'prismjs/components/prism-json';
import 'prismjs/components/prism-yaml';
import 'prismjs/components/prism-markdown';
import 'prismjs/components/prism-jsx';
import 'prismjs/components/prism-tsx';

/**
 * Supported languages for syntax highlighting
 */
export const supportedLanguages = [
  'typescript',
  'javascript',
  'python',
  'bash',
  'json',
  'yaml',
  'markdown',
  'jsx',
  'tsx',
  'shell',
  'ts',
  'js',
  'py'
] as const;

export type SupportedLanguage = typeof supportedLanguages[number];

/**
 * Map of language aliases to Prism language names
 */
const languageAliases: Record<string, string> = {
  'ts': 'typescript',
  'js': 'javascript',
  'py': 'python',
  'shell': 'bash',
  'sh': 'bash',
  'yml': 'yaml',
  'md': 'markdown'
};

/**
 * Normalize a language name to its Prism-compatible version
 */
export function normalizeLanguage(language: string): string {
  const lowercased = language.toLowerCase();
  return languageAliases[lowercased] || lowercased;
}

/**
 * Check if a language is supported
 */
export function isLanguageSupported(language: string): boolean {
  const normalized = normalizeLanguage(language);
  return normalized in Prism.languages;
}

/**
 * Highlight code with Prism
 */
export function highlightCode(code: string, language: string): string {
  const normalizedLang = normalizeLanguage(language);
  
  if (!Prism.languages[normalizedLang]) {
    console.warn(`Language "${language}" is not supported, falling back to plaintext`);
    return escapeHtml(code);
  }
  
  return Prism.highlight(code, Prism.languages[normalizedLang], normalizedLang);
}

/**
 * Escape HTML entities in code
 */
function escapeHtml(text: string): string {
  const htmlEscapes: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#39;'
  };
  
  return text.replace(/[&<>"']/g, char => htmlEscapes[char]);
}

/**
 * Get display name for a language
 */
export function getLanguageDisplayName(language: string): string {
  const displayNames: Record<string, string> = {
    typescript: 'TypeScript',
    javascript: 'JavaScript',
    python: 'Python',
    bash: 'Bash',
    shell: 'Shell',
    json: 'JSON',
    yaml: 'YAML',
    markdown: 'Markdown',
    jsx: 'JSX',
    tsx: 'TSX'
  };
  
  const normalized = normalizeLanguage(language);
  return displayNames[normalized] || language;
}

export { Prism };

