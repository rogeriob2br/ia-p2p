import { Check } from "lucide-react";
import { Card } from "@/app/components/ui/card";
import { Button } from "@/app/components/ui/button";

const tiers = [
  {
    name: "Starter",
    price: "0.001",
    unit: "per 1K tokens",
    description: "Perfect for development and testing",
    features: [
      "60 requests per minute",
      "1M tokens per day",
      "Community support",
      "API access",
      "Basic monitoring"
    ],
    cta: "Start Free",
    highlighted: false
  },
  {
    name: "Professional",
    price: "0.0008",
    unit: "per 1K tokens",
    description: "For production applications",
    features: [
      "Unlimited requests",
      "Unlimited tokens",
      "Priority support",
      "Advanced routing",
      "Real-time analytics",
      "Custom rate limits",
      "SLA guarantee"
    ],
    cta: "Get Started",
    highlighted: true
  },
  {
    name: "Enterprise",
    price: "Custom",
    unit: "volume pricing",
    description: "For large-scale deployments",
    features: [
      "Everything in Professional",
      "Dedicated nodes",
      "Private pools",
      "Custom integrations",
      "24/7 dedicated support",
      "Custom SLAs",
      "On-premise option"
    ],
    cta: "Contact Sales",
    highlighted: false
  }
];

export function Pricing() {
  return (
    <section className="py-24 px-6 lg:px-8 bg-gray-50">
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-16">
          <h2 className="text-4xl md:text-5xl font-bold mb-4" style={{ color: '#0D0D0D' }}>
            Simple, Transparent Pricing
          </h2>
          <p className="text-xl text-gray-600 max-w-2xl mx-auto">
            Pay only for what you use. No hidden fees, no commitments.
          </p>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          {tiers.map((tier, index) => (
            <Card 
              key={index}
              className={`p-8 flex flex-col ${
                tier.highlighted 
                  ? 'border-4 shadow-2xl relative transform md:scale-105' 
                  : 'border-2'
              }`}
              style={tier.highlighted ? { borderColor: '#3C35F2' } : {}}
            >
              {tier.highlighted && (
                <div 
                  className="absolute -top-4 left-1/2 transform -translate-x-1/2 px-4 py-1 rounded-full text-sm font-bold text-white"
                  style={{ backgroundColor: '#3C35F2' }}
                >
                  Most Popular
                </div>
              )}
              
              <div className="mb-6">
                <h3 className="text-2xl font-bold mb-2" style={{ color: '#0D0D0D' }}>
                  {tier.name}
                </h3>
                <p className="text-gray-600 mb-4">{tier.description}</p>
                <div className="flex items-baseline">
                  <span className="text-4xl font-bold" style={{ color: '#3C35F2' }}>
                    ${tier.price}
                  </span>
                  <span className="text-gray-500 ml-2">{tier.unit}</span>
                </div>
              </div>
              
              <ul className="space-y-4 mb-8 flex-grow">
                {tier.features.map((feature, idx) => (
                  <li key={idx} className="flex items-start gap-3">
                    <Check className="w-5 h-5 flex-shrink-0 mt-0.5" style={{ color: '#9BF2CA' }} />
                    <span className="text-gray-700">{feature}</span>
                  </li>
                ))}
              </ul>
              
              <Button 
                className="w-full py-6"
                style={
                  tier.highlighted 
                    ? { backgroundColor: '#3C35F2', color: 'white' }
                    : { backgroundColor: 'white', color: '#3C35F2', border: '2px solid #3C35F2' }
                }
              >
                {tier.cta}
              </Button>
            </Card>
          ))}
        </div>
        
        <div className="mt-12 text-center">
          <p className="text-gray-600">
            All plans include 10% pool fee. Lightning Network payments processed instantly.
          </p>
        </div>
      </div>
    </section>
  );
}
