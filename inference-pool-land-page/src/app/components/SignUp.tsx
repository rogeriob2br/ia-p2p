import { useState } from "react";
import { Button } from "@/app/components/ui/button";
import { Input } from "@/app/components/ui/input";
import { Label } from "@/app/components/ui/label";
import { Card } from "@/app/components/ui/card";
import { ArrowLeft, Mail, Lock, User, Building } from "lucide-react";

interface SignUpProps {
  onNavigate: (page: string) => void;
}

export function SignUp({ onNavigate }: SignUpProps) {
  const [formData, setFormData] = useState({
    name: "",
    email: "",
    company: "",
    password: "",
    confirmPassword: ""
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Simulação de cadastro - em produção, chamaria uma API
    console.log("Form submitted:", formData);
    onNavigate("dashboard");
  };

  return (
    <div className="min-h-screen flex items-center justify-center p-6" style={{ background: 'linear-gradient(135deg, #3C35F2 0%, #4A51D9 100%)' }}>
      <div className="absolute inset-0 overflow-hidden">
        <div className="absolute top-20 right-10 w-96 h-96 rounded-full opacity-20 blur-3xl" style={{ backgroundColor: '#9BF2CA' }}></div>
        <div className="absolute bottom-20 left-10 w-96 h-96 rounded-full opacity-10 blur-3xl" style={{ backgroundColor: '#9BF2CA' }}></div>
      </div>

      <div className="w-full max-w-md relative z-10">
        <button
          onClick={() => onNavigate("landing")}
          className="flex items-center gap-2 text-white/80 hover:text-white mb-6 transition-colors"
        >
          <ArrowLeft className="w-4 h-4" />
          Voltar para home
        </button>

        <Card className="p-8 shadow-2xl">
          <div className="text-center mb-8">
            <div className="flex items-center justify-center gap-2 mb-4">
              <div className="w-12 h-12 rounded-lg flex items-center justify-center" style={{ backgroundColor: '#3C35F2' }}>
                <div className="w-7 h-7 rounded" style={{ backgroundColor: '#9BF2CA' }}></div>
              </div>
            </div>
            <h1 className="text-3xl font-bold mb-2" style={{ color: '#0D0D0D' }}>
              Criar Conta
            </h1>
            <p className="text-gray-600">
              Comece com 1M tokens grátis
            </p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-6">
            <div>
              <Label htmlFor="name" className="flex items-center gap-2 mb-2">
                <User className="w-4 h-4" />
                Nome completo
              </Label>
              <Input
                id="name"
                type="text"
                placeholder="Seu nome"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                required
                className="h-12"
              />
            </div>

            <div>
              <Label htmlFor="email" className="flex items-center gap-2 mb-2">
                <Mail className="w-4 h-4" />
                Email
              </Label>
              <Input
                id="email"
                type="email"
                placeholder="seu@email.com"
                value={formData.email}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                required
                className="h-12"
              />
            </div>

            <div>
              <Label htmlFor="company" className="flex items-center gap-2 mb-2">
                <Building className="w-4 h-4" />
                Empresa (opcional)
              </Label>
              <Input
                id="company"
                type="text"
                placeholder="Nome da empresa"
                value={formData.company}
                onChange={(e) => setFormData({ ...formData, company: e.target.value })}
                className="h-12"
              />
            </div>

            <div>
              <Label htmlFor="password" className="flex items-center gap-2 mb-2">
                <Lock className="w-4 h-4" />
                Senha
              </Label>
              <Input
                id="password"
                type="password"
                placeholder="Mínimo 8 caracteres"
                value={formData.password}
                onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                required
                className="h-12"
              />
            </div>

            <div>
              <Label htmlFor="confirmPassword" className="flex items-center gap-2 mb-2">
                <Lock className="w-4 h-4" />
                Confirmar senha
              </Label>
              <Input
                id="confirmPassword"
                type="password"
                placeholder="Digite a senha novamente"
                value={formData.confirmPassword}
                onChange={(e) => setFormData({ ...formData, confirmPassword: e.target.value })}
                required
                className="h-12"
              />
            </div>

            <Button
              type="submit"
              className="w-full h-12 text-lg"
              style={{ backgroundColor: '#3C35F2', color: 'white' }}
            >
              Criar Conta
            </Button>

            <p className="text-center text-sm text-gray-600">
              Já tem uma conta?{" "}
              <button
                type="button"
                onClick={() => onNavigate("dashboard")}
                className="font-bold hover:underline"
                style={{ color: '#3C35F2' }}
              >
                Fazer login
              </button>
            </p>
          </form>

          <div className="mt-6 pt-6 border-t border-gray-200">
            <p className="text-xs text-gray-500 text-center">
              Ao criar uma conta, você concorda com nossos Termos de Serviço e Política de Privacidade
            </p>
          </div>
        </Card>
      </div>
    </div>
  );
}
