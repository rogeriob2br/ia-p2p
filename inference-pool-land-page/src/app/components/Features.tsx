import { Network, Zap, Shield, BarChart3, Globe, Coins } from "lucide-react";
import { Card } from "@/app/components/ui/card";

const features = [
  {
    icon: Network,
    title: "P2P Orchestration",
    description: "Decentralized network topology with automatic node discovery and intelligent load balancing across the pool."
  },
  {
    icon: Zap,
    title: "Lightning Payments",
    description: "Instant micropayments with Lightning Network integration. Pay only for what you use with zero delays."
  },
  {
    icon: Shield,
    title: "Enterprise Security",
    description: "API key authentication, rate limiting, and TLS encryption ensure your data and models stay secure."
  },
  {
    icon: BarChart3,
    title: "Smart Routing",
    description: "Context-aware routing strategies optimize for latency, load, and success rate automatically."
  },
  {
    icon: Globe,
    title: "Multi-Region",
    description: "Deploy inference nodes globally with automatic failover and geographic load distribution."
  },
  {
    icon: Coins,
    title: "Flexible Billing",
    description: "Pay-per-token pricing with configurable pool fees. Track usage in real-time with detailed analytics."
  }
];

export function Features() {
  return (
    <section className="py-24 px-6 lg:px-8 bg-white">
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-16">
          <h2 className="text-4xl md:text-5xl font-bold mb-4" style={{ color: '#0D0D0D' }}>
            Built for Scale
          </h2>
          <p className="text-xl text-gray-600 max-w-2xl mx-auto">
            Everything you need to deploy and manage AI inference at any scale
          </p>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {features.map((feature, index) => (
            <Card 
              key={index}
              className="p-8 hover:shadow-xl transition-all duration-300 border-2 hover:border-[#3C35F2] group"
            >
              <div 
                className="w-14 h-14 rounded-xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform"
                style={{ backgroundColor: '#3C35F2' }}
              >
                <feature.icon className="w-7 h-7 text-white" />
              </div>
              <h3 className="text-xl font-bold mb-3" style={{ color: '#0D0D0D' }}>
                {feature.title}
              </h3>
              <p className="text-gray-600 leading-relaxed">
                {feature.description}
              </p>
            </Card>
          ))}
        </div>
      </div>
    </section>
  );
}
