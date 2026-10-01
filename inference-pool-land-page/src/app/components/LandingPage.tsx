import { Header } from "@/app/components/Header";
import { Hero } from "@/app/components/Hero";
import { Features } from "@/app/components/Features";
import { HowItWorks } from "@/app/components/HowItWorks";
import { Pricing } from "@/app/components/Pricing";
import { CTA } from "@/app/components/CTA";
import { Footer } from "@/app/components/Footer";

interface LandingPageProps {
  onNavigate: (page: string) => void;
}

export function LandingPage({ onNavigate }: LandingPageProps) {
  return (
    <div className="min-h-screen">
      <Header onNavigate={onNavigate} />
      <Hero onNavigate={onNavigate} />
      <Features />
      <HowItWorks />
      <div id="pricing">
        <Pricing />
      </div>
      <CTA onNavigate={onNavigate} />
      <Footer />
    </div>
  );
}
