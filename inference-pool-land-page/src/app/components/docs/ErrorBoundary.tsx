/**
 * ErrorBoundary Component
 * Feature: landing-page-documentation
 * 
 * Catches React errors and displays user-friendly error messages
 * with recovery actions.
 */

import { Component, type ReactNode } from 'react';
import { AlertTriangle, RefreshCw, Home } from 'lucide-react';
import { Button } from '@/app/components/ui/button';
import { Card } from '@/app/components/ui/card';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onReset?: () => void;
  onNavigateHome?: () => void;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: React.ErrorInfo | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo): void {
    this.setState({ errorInfo });
    
    // Log error for monitoring
    console.error('[Documentation Error]', error, errorInfo);
  }

  handleReset = (): void => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null
    });
    this.props.onReset?.();
  };

  handleNavigateHome = (): void => {
    this.handleReset();
    this.props.onNavigateHome?.();
  };

  render(): ReactNode {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="min-h-[400px] flex items-center justify-center p-8">
          <Card className="max-w-md w-full p-8 text-center">
            <div 
              className="w-16 h-16 rounded-full mx-auto mb-6 flex items-center justify-center"
              style={{ backgroundColor: 'rgba(60, 53, 242, 0.1)' }}
            >
              <AlertTriangle className="w-8 h-8" style={{ color: '#3C35F2' }} />
            </div>
            
            <h2 className="text-xl font-bold text-gray-900 mb-2">
              Algo deu errado
            </h2>
            
            <p className="text-gray-600 mb-6">
              Ocorreu um erro ao carregar esta seção da documentação. 
              Tente recarregar a página ou volte para a página inicial.
            </p>

            {process.env.NODE_ENV === 'development' && this.state.error && (
              <details className="mb-6 text-left">
                <summary className="cursor-pointer text-sm text-gray-500 hover:text-gray-700">
                  Detalhes do erro (dev only)
                </summary>
                <pre className="mt-2 p-3 bg-gray-100 rounded text-xs overflow-x-auto">
                  {this.state.error.toString()}
                  {this.state.errorInfo?.componentStack}
                </pre>
              </details>
            )}

            <div className="flex gap-3 justify-center">
              <Button
                variant="outline"
                onClick={this.handleNavigateHome}
                className="flex items-center gap-2"
              >
                <Home className="w-4 h-4" />
                Página Inicial
              </Button>
              <Button
                onClick={this.handleReset}
                style={{ backgroundColor: '#3C35F2', color: 'white' }}
                className="flex items-center gap-2"
              >
                <RefreshCw className="w-4 h-4" />
                Tentar Novamente
              </Button>
            </div>
          </Card>
        </div>
      );
    }

    return this.props.children;
  }
}
