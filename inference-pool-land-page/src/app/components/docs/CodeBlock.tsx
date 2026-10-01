/**
 * CodeBlock Component
 * Feature: landing-page-documentation
 * 
 * Syntax-highlighted code block with copy-to-clipboard functionality.
 * Uses Prism.js for syntax highlighting.
 * 
 * Requirements: 2.5, 5.1, 5.2, 5.3, 5.4
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { Copy, Check, Terminal, FileCode } from 'lucide-react';
import { highlightCode, getLanguageDisplayName, normalizeLanguage } from '@/app/utils/prism';
import { cn } from '@/app/components/ui/utils';

export interface CodeBlockProps {
  /** The code to display */
  code: string;
  /** Programming language for syntax highlighting */
  language: string;
  /** Optional filename to display */
  filename?: string;
  /** Whether to show line numbers */
  showLineNumbers?: boolean;
  /** Additional class names */
  className?: string;
}

export function CodeBlock({
  code,
  language,
  filename,
  showLineNumbers = false,
  className
}: CodeBlockProps) {
  const [copied, setCopied] = useState(false);
  const [highlightedCode, setHighlightedCode] = useState('');
  const codeRef = useRef<HTMLPreElement>(null);

  // Highlight code on mount and when code/language changes
  useEffect(() => {
    const highlighted = highlightCode(code.trim(), language);
    setHighlightedCode(highlighted);
  }, [code, language]);

  // Copy to clipboard
  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(code.trim());
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      // Fallback for browsers without clipboard API
      const textArea = document.createElement('textarea');
      textArea.value = code.trim();
      textArea.style.position = 'fixed';
      textArea.style.left = '-9999px';
      document.body.appendChild(textArea);
      textArea.select();
      try {
        document.execCommand('copy');
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch {
        console.error('Failed to copy code');
      }
      document.body.removeChild(textArea);
    }
  }, [code]);

  const normalizedLang = normalizeLanguage(language);
  const displayName = getLanguageDisplayName(language);
  const isTerminal = normalizedLang === 'bash' || normalizedLang === 'shell';
  const Icon = isTerminal ? Terminal : FileCode;

  // Generate line numbers
  const lines = code.trim().split('\n');
  const lineNumbers = showLineNumbers ? lines.map((_, i) => i + 1) : [];

  return (
    <div className={cn('relative group rounded-lg overflow-hidden', className)}>
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center gap-2">
          <Icon className="w-4 h-4 text-gray-400" />
          {filename ? (
            <span className="text-sm text-gray-300 font-mono">{filename}</span>
          ) : (
            <span className="text-sm text-gray-400">{displayName}</span>
          )}
        </div>
        
        {/* Copy button */}
        <button
          onClick={handleCopy}
          className={cn(
            'flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium transition-all',
            copied
              ? 'bg-green-500/20 text-green-400'
              : 'bg-gray-700 text-gray-300 hover:bg-gray-600 hover:text-white'
          )}
          aria-label={copied ? 'Copied!' : 'Copy code'}
        >
          {copied ? (
            <>
              <Check className="w-3.5 h-3.5" />
              <span>Copiado!</span>
            </>
          ) : (
            <>
              <Copy className="w-3.5 h-3.5" />
              <span className="hidden sm:inline">Copiar</span>
            </>
          )}
        </button>
      </div>

      {/* Code content */}
      <div className="relative bg-[#0D0D0D] overflow-x-auto">
        {showLineNumbers && (
          <div 
            className="absolute left-0 top-0 bottom-0 w-12 bg-gray-900/50 border-r border-gray-800 select-none"
            aria-hidden="true"
          >
            <div className="py-4 px-2 text-right">
              {lineNumbers.map(num => (
                <div key={num} className="text-xs text-gray-600 leading-6">
                  {num}
                </div>
              ))}
            </div>
          </div>
        )}
        
        <pre
          ref={codeRef}
          className={cn(
            'p-4 text-sm leading-6 overflow-x-auto',
            showLineNumbers && 'pl-16'
          )}
        >
          <code
            className={`language-${normalizedLang}`}
            dangerouslySetInnerHTML={{ __html: highlightedCode }}
          />
        </pre>
      </div>
    </div>
  );
}

/**
 * Inline code component for use within text
 */
export function InlineCode({ children }: { children: React.ReactNode }) {
  return (
    <code className="px-1.5 py-0.5 bg-gray-100 text-[#3C35F2] rounded text-sm font-mono">
      {children}
    </code>
  );
}
