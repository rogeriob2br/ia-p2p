import { useState } from "react";
import { Button } from "@/app/components/ui/button";
import { Card } from "@/app/components/ui/card";
import { Progress } from "@/app/components/ui/progress";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/app/components/ui/tabs";
import {
  LayoutDashboard,
  Key,
  TrendingUp,
  Server,
  Settings,
  LogOut,
  Copy,
  Check,
  Eye,
  EyeOff,
  Plus,
  Activity
} from "lucide-react";
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, BarChart, Bar } from "recharts";

interface DashboardProps {
  onNavigate: (page: string) => void;
}

const usageData = [
  { date: "15 Jan", tokens: 45000, requests: 120 },
  { date: "16 Jan", tokens: 52000, requests: 145 },
  { date: "17 Jan", tokens: 48000, requests: 130 },
  { date: "18 Jan", tokens: 61000, requests: 165 },
  { date: "19 Jan", tokens: 55000, requests: 150 },
  { date: "20 Jan", tokens: 70000, requests: 180 },
  { date: "Hoje", tokens: 38000, requests: 95 }
];

const latencyData = [
  { time: "00:00", latency: 85 },
  { time: "04:00", latency: 72 },
  { time: "08:00", latency: 95 },
  { time: "12:00", latency: 110 },
  { time: "16:00", latency: 88 },
  { time: "20:00", latency: 78 },
  { time: "Agora", latency: 92 }
];

export function Dashboard({ onNavigate }: DashboardProps) {
  const [activeTab, setActiveTab] = useState("overview");
  const [apiKeys, setApiKeys] = useState([
    { id: 1, name: "Production Key", key: "sk-pool-prod-abc123...", created: "15 Jan 2026", lastUsed: "Hoje" },
    { id: 2, name: "Development Key", key: "sk-pool-dev-xyz789...", created: "10 Jan 2026", lastUsed: "Ontem" }
  ]);
  const [showKey, setShowKey] = useState<number | null>(null);
  const [copiedKey, setCopiedKey] = useState<number | null>(null);

  const copyToClipboard = (text: string, id: number) => {
    navigator.clipboard.writeText(text);
    setCopiedKey(id);
    setTimeout(() => setCopiedKey(null), 2000);
  };

  const toggleKeyVisibility = (id: number) => {
    setShowKey(showKey === id ? null : id);
  };

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <div className="w-10 h-10 rounded-lg flex items-center justify-center" style={{ backgroundColor: '#3C35F2' }}>
                <div className="w-6 h-6 rounded" style={{ backgroundColor: '#9BF2CA' }}></div>
              </div>
              <div>
                <h1 className="font-bold text-lg">InferencePool</h1>
                <p className="text-xs text-gray-500">Dashboard</p>
              </div>
            </div>
          </div>
          
          <div className="flex items-center gap-4">
            <Button
              variant="outline"
              onClick={() => onNavigate("documentation")}
            >
              Documentação
            </Button>
            <Button
              variant="outline"
              onClick={() => onNavigate("landing")}
              className="flex items-center gap-2"
            >
              <LogOut className="w-4 h-4" />
              Sair
            </Button>
          </div>
        </div>
      </header>

      <div className="max-w-7xl mx-auto px-6 py-8">
        <div className="mb-8">
          <h2 className="text-3xl font-bold mb-2" style={{ color: '#0D0D0D' }}>
            Bem-vindo de volta!
          </h2>
          <p className="text-gray-600">Aqui está um resumo da sua atividade</p>
        </div>

        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <Card className="p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 rounded-lg flex items-center justify-center" style={{ backgroundColor: '#3C35F2' }}>
                <TrendingUp className="w-6 h-6 text-white" />
              </div>
              <span className="text-sm text-green-600 font-medium">+12.5%</span>
            </div>
            <p className="text-gray-600 text-sm mb-1">Tokens Usados (Hoje)</p>
            <p className="text-2xl font-bold">38,245</p>
          </Card>

          <Card className="p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 rounded-lg flex items-center justify-center" style={{ backgroundColor: '#4A51D9' }}>
                <Activity className="w-6 h-6 text-white" />
              </div>
              <span className="text-sm text-green-600 font-medium">+8.3%</span>
            </div>
            <p className="text-gray-600 text-sm mb-1">Requisições</p>
            <p className="text-2xl font-bold">1,245</p>
          </Card>

          <Card className="p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 rounded-lg flex items-center justify-center" style={{ backgroundColor: '#9BF2CA' }}>
                <Server className="w-6 h-6" style={{ color: '#0D0D0D' }} />
              </div>
              <span className="text-sm text-gray-500">Média</span>
            </div>
            <p className="text-gray-600 text-sm mb-1">Latência</p>
            <p className="text-2xl font-bold">92ms</p>
          </Card>

          <Card className="p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 rounded-lg flex items-center justify-center bg-green-100">
                <Check className="w-6 h-6 text-green-600" />
              </div>
              <span className="text-sm text-green-600 font-medium">99.8%</span>
            </div>
            <p className="text-gray-600 text-sm mb-1">Uptime</p>
            <p className="text-2xl font-bold">Online</p>
          </Card>
        </div>

        {/* Tabs */}
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList className="mb-6">
            <TabsTrigger value="overview" className="flex items-center gap-2">
              <LayoutDashboard className="w-4 h-4" />
              Visão Geral
            </TabsTrigger>
            <TabsTrigger value="api-keys" className="flex items-center gap-2">
              <Key className="w-4 h-4" />
              API Keys
            </TabsTrigger>
            <TabsTrigger value="settings" className="flex items-center gap-2">
              <Settings className="w-4 h-4" />
              Configurações
            </TabsTrigger>
          </TabsList>

          <TabsContent value="overview" className="space-y-6">
            {/* Usage Chart */}
            <Card className="p-6">
              <div className="mb-6">
                <h3 className="text-xl font-bold mb-2">Uso de Tokens</h3>
                <p className="text-gray-600 text-sm">Últimos 7 dias</p>
              </div>
              <ResponsiveContainer width="100%" height={300}>
                <AreaChart data={usageData}>
                  <defs>
                    <linearGradient id="colorTokens" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#3C35F2" stopOpacity={0.3}/>
                      <stop offset="95%" stopColor="#3C35F2" stopOpacity={0}/>
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                  <XAxis dataKey="date" stroke="#6b7280" />
                  <YAxis stroke="#6b7280" />
                  <Tooltip />
                  <Area 
                    type="monotone" 
                    dataKey="tokens" 
                    stroke="#3C35F2" 
                    fillOpacity={1} 
                    fill="url(#colorTokens)" 
                  />
                </AreaChart>
              </ResponsiveContainer>
            </Card>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Latency Chart */}
              <Card className="p-6">
                <div className="mb-6">
                  <h3 className="text-xl font-bold mb-2">Latência</h3>
                  <p className="text-gray-600 text-sm">Últimas 24 horas</p>
                </div>
                <ResponsiveContainer width="100%" height={250}>
                  <BarChart data={latencyData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                    <XAxis dataKey="time" stroke="#6b7280" />
                    <YAxis stroke="#6b7280" />
                    <Tooltip />
                    <Bar dataKey="latency" fill="#9BF2CA" radius={[8, 8, 0, 0]} />
                  </BarChart>
                </ResponsiveContainer>
              </Card>

              {/* Usage Quota */}
              <Card className="p-6">
                <div className="mb-6">
                  <h3 className="text-xl font-bold mb-2">Cota Mensal</h3>
                  <p className="text-gray-600 text-sm">Período atual: Jan 2026</p>
                </div>
                
                <div className="space-y-6">
                  <div>
                    <div className="flex justify-between mb-2">
                      <span className="text-sm font-medium">Tokens</span>
                      <span className="text-sm text-gray-600">369K / 1M</span>
                    </div>
                    <Progress value={36.9} className="h-3" style={{ backgroundColor: '#e5e7eb' }}>
                      <div className="h-full transition-all" style={{ width: '36.9%', backgroundColor: '#3C35F2' }}></div>
                    </Progress>
                    <p className="text-xs text-gray-500 mt-1">36.9% utilizado</p>
                  </div>

                  <div>
                    <div className="flex justify-between mb-2">
                      <span className="text-sm font-medium">Requisições</span>
                      <span className="text-sm text-gray-600">Ilimitado</span>
                    </div>
                    <Progress value={100} className="h-3" style={{ backgroundColor: '#e5e7eb' }}>
                      <div className="h-full transition-all" style={{ width: '100%', backgroundColor: '#9BF2CA' }}></div>
                    </Progress>
                    <p className="text-xs text-gray-500 mt-1">Plano Professional</p>
                  </div>

                  <div className="pt-4 border-t border-gray-200">
                    <div className="flex justify-between items-center mb-2">
                      <span className="font-medium">Custo Estimado</span>
                      <span className="text-2xl font-bold" style={{ color: '#3C35F2' }}>$0.37</span>
                    </div>
                    <p className="text-xs text-gray-500">
                      Baseado em $0.001 por 1K tokens
                    </p>
                  </div>
                </div>
              </Card>
            </div>

            {/* Recent Activity */}
            <Card className="p-6">
              <h3 className="text-xl font-bold mb-4">Atividade Recente</h3>
              <div className="space-y-4">
                {[
                  { time: "há 2 minutos", action: "Requisição completada", model: "gpt-4", tokens: "1,245 tokens" },
                  { time: "há 15 minutos", action: "Requisição completada", model: "gpt-3.5-turbo", tokens: "856 tokens" },
                  { time: "há 1 hora", action: "Requisição completada", model: "gpt-4", tokens: "2,341 tokens" },
                  { time: "há 2 horas", action: "Nova API key criada", model: "Production Key", tokens: "" }
                ].map((activity, index) => (
                  <div key={index} className="flex items-center justify-between py-3 border-b border-gray-100 last:border-0">
                    <div className="flex items-center gap-4">
                      <div className="w-2 h-2 rounded-full" style={{ backgroundColor: '#9BF2CA' }}></div>
                      <div>
                        <p className="font-medium">{activity.action}</p>
                        <p className="text-sm text-gray-500">{activity.time}</p>
                      </div>
                    </div>
                    <div className="text-right">
                      <p className="font-medium">{activity.model}</p>
                      {activity.tokens && <p className="text-sm text-gray-500">{activity.tokens}</p>}
                    </div>
                  </div>
                ))}
              </div>
            </Card>
          </TabsContent>

          <TabsContent value="api-keys" className="space-y-6">
            <div className="flex justify-between items-center">
              <div>
                <h3 className="text-xl font-bold mb-2">API Keys</h3>
                <p className="text-gray-600">Gerencie suas chaves de API para autenticação</p>
              </div>
              <Button
                className="flex items-center gap-2"
                style={{ backgroundColor: '#3C35F2', color: 'white' }}
              >
                <Plus className="w-4 h-4" />
                Nova API Key
              </Button>
            </div>

            <div className="space-y-4">
              {apiKeys.map((apiKey) => (
                <Card key={apiKey.id} className="p-6">
                  <div className="flex items-start justify-between mb-4">
                    <div>
                      <h4 className="font-bold text-lg mb-1">{apiKey.name}</h4>
                      <p className="text-sm text-gray-500">Criada em {apiKey.created}</p>
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => toggleKeyVisibility(apiKey.id)}
                        className="p-2 hover:bg-gray-100 rounded transition-colors"
                      >
                        {showKey === apiKey.id ? (
                          <EyeOff className="w-4 h-4 text-gray-600" />
                        ) : (
                          <Eye className="w-4 h-4 text-gray-600" />
                        )}
                      </button>
                      <button
                        onClick={() => copyToClipboard(apiKey.key, apiKey.id)}
                        className="p-2 hover:bg-gray-100 rounded transition-colors"
                      >
                        {copiedKey === apiKey.id ? (
                          <Check className="w-4 h-4 text-green-600" />
                        ) : (
                          <Copy className="w-4 h-4 text-gray-600" />
                        )}
                      </button>
                    </div>
                  </div>
                  
                  <div className="bg-gray-50 p-4 rounded-lg font-mono text-sm mb-4">
                    {showKey === apiKey.id ? apiKey.key : apiKey.key.substring(0, 20) + "••••••••••••"}
                  </div>

                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-600">Último uso: {apiKey.lastUsed}</span>
                    <Button variant="outline" size="sm" className="text-red-600 hover:text-red-700">
                      Revogar
                    </Button>
                  </div>
                </Card>
              ))}
            </div>
          </TabsContent>

          <TabsContent value="settings" className="space-y-6">
            <Card className="p-6">
              <h3 className="text-xl font-bold mb-6">Configurações da Conta</h3>
              
              <div className="space-y-6">
                <div>
                  <label className="block font-medium mb-2">Nome</label>
                  <input
                    type="text"
                    defaultValue="João Silva"
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>

                <div>
                  <label className="block font-medium mb-2">Email</label>
                  <input
                    type="email"
                    defaultValue="joao@exemplo.com"
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>

                <div>
                  <label className="block font-medium mb-2">Empresa</label>
                  <input
                    type="text"
                    defaultValue="Minha Empresa"
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>

                <div className="pt-4 border-t border-gray-200">
                  <Button
                    style={{ backgroundColor: '#3C35F2', color: 'white' }}
                  >
                    Salvar Alterações
                  </Button>
                </div>
              </div>
            </Card>

            <Card className="p-6 border-red-200 bg-red-50">
              <h3 className="text-xl font-bold mb-4 text-red-900">Zona de Perigo</h3>
              <p className="text-sm text-red-700 mb-4">
                Ações irreversíveis. Tenha cuidado.
              </p>
              <Button variant="outline" className="border-red-600 text-red-600 hover:bg-red-50">
                Excluir Conta
              </Button>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}