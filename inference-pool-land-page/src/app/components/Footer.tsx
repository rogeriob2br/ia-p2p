import { Github, Twitter, MessageCircle } from "lucide-react";

const navigation = {
  product: [
    { name: 'Features', href: '#' },
    { name: 'Pricing', href: '#' },
    { name: 'Documentation', href: '#' },
    { name: 'API Reference', href: '#' },
  ],
  company: [
    { name: 'About', href: '#' },
    { name: 'Blog', href: '#' },
    { name: 'Careers', href: '#' },
    { name: 'Contact', href: '#' },
  ],
  resources: [
    { name: 'Community', href: '#' },
    { name: 'Guides', href: '#' },
    { name: 'Status', href: '#' },
    { name: 'Support', href: '#' },
  ],
  legal: [
    { name: 'Privacy', href: '#' },
    { name: 'Terms', href: '#' },
    { name: 'Security', href: '#' },
  ],
};

const social = [
  { name: 'GitHub', icon: Github, href: '#' },
  { name: 'Twitter', icon: Twitter, href: '#' },
  { name: 'Discord', icon: MessageCircle, href: '#' },
];

export function Footer() {
  return (
    <footer className="bg-[#0D0D0D] text-white">
      <div className="max-w-7xl mx-auto px-6 lg:px-8 py-16">
        <div className="grid grid-cols-2 md:grid-cols-5 gap-8 mb-12">
          <div className="col-span-2 md:col-span-1">
            <div className="flex items-center gap-2 mb-4">
              <div className="w-10 h-10 rounded-lg flex items-center justify-center" style={{ backgroundColor: '#3C35F2' }}>
                <div className="w-6 h-6 rounded" style={{ backgroundColor: '#9BF2CA' }}></div>
              </div>
              <span className="text-xl font-bold">InferencePool</span>
            </div>
            <p className="text-gray-400 text-sm mb-6">
              Decentralized AI inference for the next generation of applications.
            </p>
            <div className="flex gap-4">
              {social.map((item) => (
                <a
                  key={item.name}
                  href={item.href}
                  className="text-gray-400 hover:text-white transition-colors"
                >
                  <item.icon className="w-5 h-5" />
                </a>
              ))}
            </div>
          </div>
          
          <div>
            <h3 className="font-bold mb-4">Product</h3>
            <ul className="space-y-3">
              {navigation.product.map((item) => (
                <li key={item.name}>
                  <a href={item.href} className="text-gray-400 hover:text-white transition-colors text-sm">
                    {item.name}
                  </a>
                </li>
              ))}
            </ul>
          </div>
          
          <div>
            <h3 className="font-bold mb-4">Company</h3>
            <ul className="space-y-3">
              {navigation.company.map((item) => (
                <li key={item.name}>
                  <a href={item.href} className="text-gray-400 hover:text-white transition-colors text-sm">
                    {item.name}
                  </a>
                </li>
              ))}
            </ul>
          </div>
          
          <div>
            <h3 className="font-bold mb-4">Resources</h3>
            <ul className="space-y-3">
              {navigation.resources.map((item) => (
                <li key={item.name}>
                  <a href={item.href} className="text-gray-400 hover:text-white transition-colors text-sm">
                    {item.name}
                  </a>
                </li>
              ))}
            </ul>
          </div>
          
          <div>
            <h3 className="font-bold mb-4">Legal</h3>
            <ul className="space-y-3">
              {navigation.legal.map((item) => (
                <li key={item.name}>
                  <a href={item.href} className="text-gray-400 hover:text-white transition-colors text-sm">
                    {item.name}
                  </a>
                </li>
              ))}
            </ul>
          </div>
        </div>
        
        <div className="border-t border-gray-800 pt-8 flex flex-col md:flex-row justify-between items-center gap-4">
          <p className="text-gray-400 text-sm">
            © {new Date().getFullYear()} InferencePool. All rights reserved.
          </p>
          <p className="text-gray-400 text-sm">
            Version 0.1.0 • Built with ⚡ Lightning Network
          </p>
        </div>
      </div>
    </footer>
  );
}
