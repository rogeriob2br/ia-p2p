import { Button } from "@/app/components/ui/button";

interface HeaderProps {
  onNavigate: (page: string) => void;
}

export function Header({ onNavigate }: HeaderProps) {
  return (
    <header className="absolute top-0 left-0 right-0 z-50 bg-transparent">
      <div className="max-w-7xl mx-auto px-6 py-6 flex items-center justify-between">
        <div className="flex items-center gap-2 cursor-pointer" onClick={() => onNavigate("landing")}>
          <div className="w-10 h-10 rounded-lg flex items-center justify-center" style={{ backgroundColor: '#9BF2CA' }}>
            <div className="w-6 h-6 rounded" style={{ backgroundColor: '#3C35F2' }}></div>
          </div>
          <span className="text-xl font-bold text-white">InferencePool</span>
        </div>
        
        <nav className="hidden md:flex items-center gap-8">
          <button onClick={() => onNavigate("landing")} className="text-white/90 hover:text-white transition-colors">
            Home
          </button>
          <button onClick={() => onNavigate("documentation")} className="text-white/90 hover:text-white transition-colors">
            Documentação
          </button>
          <a href="#pricing" className="text-white/90 hover:text-white transition-colors">
            Preços
          </a>
        </nav>

        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            onClick={() => onNavigate("dashboard")}
            className="text-white hover:bg-white/10"
          >
            Login
          </Button>
          <Button
            onClick={() => onNavigate("signup")}
            style={{ backgroundColor: '#9BF2CA', color: '#0D0D0D' }}
          >
            Começar Agora
          </Button>
        </div>
      </div>
    </header>
  );
}
