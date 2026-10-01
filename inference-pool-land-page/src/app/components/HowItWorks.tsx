import { Code, Server, Rocket } from "lucide-react";

const steps = [
  {
    icon: Code,
    title: "Integrate",
    description: "Add our SDK to your LangChain or CrewAI project. Simple integration with just a few lines of code."
  },
  {
    icon: Server,
    title: "Deploy",
    description: "Your inference requests are routed to the optimal node in our decentralized pool automatically."
  },
  {
    icon: Rocket,
    title: "Scale",
    description: "Pay as you grow with instant Lightning payments. No upfront costs, no infrastructure management."
  }
];

export function HowItWorks() {
  return (
    <section className="py-24 px-6 lg:px-8" style={{ backgroundColor: '#0D0D0D' }}>
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-16">
          <h2 className="text-4xl md:text-5xl font-bold text-white mb-4">
            How It Works
          </h2>
          <p className="text-xl text-gray-400 max-w-2xl mx-auto">
            From integration to production in minutes
          </p>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-12 relative">
          {/* Connecting lines for desktop */}
          <div className="hidden md:block absolute top-16 left-0 right-0 h-0.5 bg-gradient-to-r from-transparent via-[#9BF2CA]/30 to-transparent"></div>
          
          {steps.map((step, index) => (
            <div key={index} className="relative">
              <div className="flex flex-col items-center text-center">
                <div 
                  className="w-32 h-32 rounded-full flex items-center justify-center mb-6 relative z-10"
                  style={{ backgroundColor: '#3C35F2' }}
                >
                  <step.icon className="w-16 h-16 text-white" />
                </div>
                <div 
                  className="text-6xl font-bold mb-4 opacity-20"
                  style={{ color: '#9BF2CA' }}
                >
                  {(index + 1).toString().padStart(2, '0')}
                </div>
                <h3 className="text-2xl font-bold text-white mb-4">
                  {step.title}
                </h3>
                <p className="text-gray-400 leading-relaxed">
                  {step.description}
                </p>
              </div>
            </div>
          ))}
        </div>
        
        <div className="mt-16 text-center">
          <div className="inline-block bg-white/5 backdrop-blur-sm border border-white/10 rounded-2xl p-8 max-w-3xl">
            <pre className="text-left text-sm overflow-x-auto">
              <code className="text-[#9BF2CA]">
{`import { InferencePool } from '@inference-pool/sdk';

const pool = new InferencePool({
  apiKey: 'sk-pool-...',
  network: 'mainnet'
});

const response = await pool.complete({
  model: 'gpt-4',
  messages: [{ role: 'user', content: 'Hello!' }]
});`}
              </code>
            </pre>
          </div>
        </div>
      </div>
    </section>
  );
}
