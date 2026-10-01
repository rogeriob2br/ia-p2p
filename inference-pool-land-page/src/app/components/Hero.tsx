import { Button } from "@/app/components/ui/button";
import { ArrowRight, Zap } from "lucide-react";

interface HeroProps {
  onNavigate: (page: string) => void;
}

export function Hero({ onNavigate }: HeroProps) {
  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden" style={{ background: 'linear-gradient(135deg, #3C35F2 0%, #4A51D9 50%, #4A5FD9 100%)' }}>
      {/* Background decorative elements */}
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute top-20 left-10 w-72 h-72 rounded-full opacity-20 blur-3xl" style={{ backgroundColor: '#9BF2CA' }}></div>
        <div className="absolute bottom-20 right-10 w-96 h-96 rounded-full opacity-10 blur-3xl" style={{ backgroundColor: '#9BF2CA' }}></div>
      </div>
      
      <div className="relative z-10 max-w-7xl mx-auto px-6 lg:px-8 py-24 text-center">
        <div className="inline-flex items-center gap-2 bg-white/10 backdrop-blur-sm px-4 py-2 rounded-full mb-8 border border-white/20">
          <Zap className="w-4 h-4 text-[#9BF2CA]" />
          <span className="text-white/90 text-sm">Decentralized AI Infrastructure</span>
        </div>
        
        <h1 className="text-5xl md:text-7xl font-bold text-white mb-6 leading-tight">
          Scale Your AI Inference<br />
          <span style={{ color: '#9BF2CA' }}>Without Limits</span>
        </h1>
        
        <p className="text-xl md:text-2xl text-white/80 mb-12 max-w-3xl mx-auto">
          Join the decentralized inference pool. Deploy AI models with P2P orchestration, 
          Lightning-fast payments, and unlimited scalability.
        </p>
        
        <div className="flex flex-col sm:flex-row gap-4 justify-center items-center">
          <Button 
            size="lg" 
            className="text-lg px-8 py-6 group"
            style={{ backgroundColor: '#9BF2CA', color: '#0D0D0D' }}
            onClick={() => onNavigate("signup")}
          >
            Get Started
            <ArrowRight className="ml-2 w-5 h-5 group-hover:translate-x-1 transition-transform" />
          </Button>
          <Button 
            size="lg" 
            variant="outline" 
            className="text-lg px-8 py-6 border-2 bg-transparent hover:bg-white/10"
            style={{ borderColor: 'rgba(255,255,255,0.3)', color: 'white' }}
            onClick={() => onNavigate("documentation")}
          >
            View Documentation
          </Button>
        </div>
        
        <div className="mt-16 grid grid-cols-1 md:grid-cols-3 gap-8 max-w-4xl mx-auto">
          <div className="bg-white/10 backdrop-blur-sm rounded-2xl p-6 border border-white/20">
            <div className="text-4xl font-bold text-white mb-2">99.9%</div>
            <div className="text-white/70">Uptime Guarantee</div>
          </div>
          <div className="bg-white/10 backdrop-blur-sm rounded-2xl p-6 border border-white/20">
            <div className="text-4xl font-bold text-white mb-2">&lt;100ms</div>
            <div className="text-white/70">Average Latency</div>
          </div>
          <div className="bg-white/10 backdrop-blur-sm rounded-2xl p-6 border border-white/20">
            <div className="text-4xl font-bold text-white mb-2">1000+</div>
            <div className="text-white/70">Active Nodes</div>
          </div>
        </div>
      </div>
    </section>
  );
}