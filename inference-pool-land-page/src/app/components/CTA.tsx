import { Button } from "@/app/components/ui/button";
import { ArrowRight } from "lucide-react";

interface CTAProps {
  onNavigate: (page: string) => void;
}

export function CTA({ onNavigate }: CTAProps) {
  return (
    <section className="py-24 px-6 lg:px-8 relative overflow-hidden" style={{ background: 'linear-gradient(135deg, #3C35F2 0%, #4A51D9 100%)' }}>
      {/* Background decorative elements */}
      <div className="absolute inset-0 overflow-hidden opacity-30">
        <div className="absolute top-0 left-1/4 w-96 h-96 rounded-full blur-3xl" style={{ backgroundColor: '#9BF2CA' }}></div>
        <div className="absolute bottom-0 right-1/4 w-96 h-96 rounded-full blur-3xl" style={{ backgroundColor: '#9BF2CA' }}></div>
      </div>
      
      <div className="max-w-4xl mx-auto text-center relative z-10">
        <h2 className="text-4xl md:text-5xl font-bold text-white mb-6">
          Ready to Scale Your AI Infrastructure?
        </h2>
        <p className="text-xl text-white/80 mb-10 max-w-2xl mx-auto">
          Join hundreds of developers leveraging decentralized inference for their AI applications.
        </p>
        
        <div className="flex flex-col sm:flex-row gap-4 justify-center items-center">
          <Button 
            size="lg" 
            className="text-lg px-10 py-6 group"
            style={{ backgroundColor: '#9BF2CA', color: '#0D0D0D' }}
            onClick={() => onNavigate("signup")}
          >
            Start Building Now
            <ArrowRight className="ml-2 w-5 h-5 group-hover:translate-x-1 transition-transform" />
          </Button>
          <Button 
            size="lg" 
            variant="outline" 
            className="text-lg px-10 py-6 border-2 bg-transparent hover:bg-white/10"
            style={{ borderColor: 'rgba(255,255,255,0.3)', color: 'white' }}
            onClick={() => onNavigate("dashboard")}
          >
            Talk to Sales
          </Button>
        </div>
        
        <p className="text-white/60 mt-8 text-sm">
          No credit card required • 1M free tokens to start
        </p>
      </div>
    </section>
  );
}