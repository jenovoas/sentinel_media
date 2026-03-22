import React from 'react';
import {
    Activity,
    LayoutDashboard,
    Terminal,
    Factory,
    MessageSquare,
    Rocket,
    Database,
    Search,
    TrendingUp,
    Brain,
    Settings,
    Shield,
    Globe
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';

interface SidebarProps {
    currentView: string;
    setView: (view: string) => void;
}

const Sidebar: React.FC<SidebarProps> = ({ currentView, setView }) => {
    const [balancerStatus, setBalancerStatus] = React.useState('Cargando...');

    React.useEffect(() => {
        const checkBalancer = async () => {
            try {
                const status = await invoke<string>('get_balancer_status');
                setBalancerStatus(status);
            } catch (e) {
                setBalancerStatus('Error');
            }
        };
        checkBalancer();
        const interval = setInterval(checkBalancer, 30000);
        return () => clearInterval(interval);
    }, []);
    const menuItems = [
        { id: 'observe', label: 'DASHBOARD', icon: LayoutDashboard, desc: 'Estado General' },
        { id: 'production', label: 'PLAN DE PRODUCCIÓN', icon: Factory, desc: 'Pipeline de Contenido' },
        { id: 'dialog', label: 'CONSULTA', icon: MessageSquare, desc: 'Interfaz LLM' },
        { id: 'swarm', label: 'AGENTES', icon: Rocket, desc: 'Orquestación Rust' },
        { id: 'vault', label: 'BIBLIOTECA', icon: Database, desc: 'Bóveda Obsidian' },
        { id: 'factory', label: 'FÁBRICA', icon: Factory, desc: 'Producción Media' },
        { id: 'commands', label: 'COMANDOS', icon: Terminal, desc: 'Machete Sentinel' },
        { id: 'hacker', label: 'CONSOLA', icon: Activity, desc: 'Terminal Nodo' },
        { id: 'research', label: 'RESEARCH', icon: Search, desc: 'Investigación Deep' },
        { id: 'sentinomics', label: 'SENTINOMICS', icon: TrendingUp, desc: 'ROI & Monetización' },
        { id: 'cognitive', label: 'CAPA COGNITIVA', icon: Brain, desc: 'Personalidad Agentes' },
        { id: 'settings', label: 'SETTINGS', icon: Settings, desc: 'Configuración \u0026 Costos' },
    ];

    return (
        <aside className="w-72 bg-cyber-dark border-r border-white/5 flex flex-col p-8 select-none relative z-20">
            <div className="mb-12">
                <div className="flex items-center gap-4 px-2 mb-2">
                    <div className="w-10 h-10 rounded-2xl bg-sentinel-blue/10 border border-sentinel-blue/20 flex items-center justify-center">
                        <Shield className="text-sentinel-blue" size={20} />
                    </div>
                    <div>
                        <h2 className="text-xl font-black text-white tracking-tighter uppercase">GESTIÓN</h2>
                        <p className="text-[10px] font-black text-white/20 uppercase tracking-[0.3em]">NODO NATIVO</p>
                    </div>
                </div>
            </div>

            <nav className="flex-1 space-y-2 overflow-y-auto custom-scrollbar pr-2 mb-4">
                <p className="px-4 text-[9px] font-black text-white/10 uppercase tracking-[0.4em] mb-4 sticky top-0 bg-cyber-dark z-10 py-2">Módulos del Sistema</p>
                {menuItems.map((item) => (
                    <button
                        key={item.id}
                        onClick={() => setView(item.id)}
                        className={`w-full group flex items-center gap-4 px-4 py-4 rounded-2xl transition-all relative overflow-hidden flex-shrink-0 ${currentView === item.id
                            ? 'bg-sentinel-blue/10 text-sentinel-blue border border-sentinel-blue/20'
                            : 'text-white/30 hover:bg-white/[0.03] hover:text-white/60 border border-transparent'
                            }`}
                    >
                        {currentView === item.id && (
                            <motion.div
                                layoutId="active-pill"
                                className="absolute left-0 w-1 h-6 bg-sentinel-blue rounded-r-full"
                            />
                        )}
                        <item.icon size={20} className={`flex-shrink-0 ${currentView === item.id ? 'text-sentinel-blue' : 'text-inherit'}`} />
                        <div className="text-left">
                            <div className="text-xs font-black uppercase tracking-widest">{item.label}</div>
                            <div className="text-[9px] font-bold uppercase tracking-wider opacity-40 group-hover:opacity-100 transition-opacity italic">{item.desc}</div>
                        </div>
                    </button>
                ))}
            </nav>

            <div className="mt-auto pt-8 border-t border-white/5">
                <div className="p-5 rounded-2xl bg-white/[0.02] border border-white/5 space-y-4">
                    <div className="flex justify-between items-center text-[9px] font-black uppercase tracking-widest text-white/20">
                        <span>Estado Sincronía</span>
                        <span className="text-sentinel-green">Online</span>
                    </div>
                    <div className="flex gap-1.5">
                        {[1, 2, 3, 4, 5].map(i => (
                            <div key={i} className={`h-1 flex-1 rounded-full ${i <= 3 ? 'bg-sentinel-blue/40' : 'bg-white/5'}`} />
                        ))}
                    </div>

                    <div className="pt-4 border-t border-white/5 flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <Globe size={12} className={balancerStatus.includes('gen-lang') ? 'text-sentinel-green' : 'text-yellow-500'} />
                            <span className="text-[8px] font-black uppercase tracking-widest text-white/40">GCP POOL</span>
                        </div>
                        <span className={`text-[8px] font-mono font-bold truncate max-w-[100px] ${balancerStatus.includes('gen-lang') ? 'text-sentinel-green' : 'text-yellow-500'}`}>
                            {balancerStatus}
                        </span>
                    </div>
                </div>

                <div className="mt-6 flex items-center justify-center gap-4 opacity-20 hover:opacity-100 transition-opacity">
                    <Activity size={14} />
                    <p className="text-[9px] font-black uppercase tracking-[0.2em]">{new Date().toLocaleTimeString('es-ES', { hour: '2-digit', minute: '2-digit' })}</p>
                </div>
            </div>
        </aside>
    );
};

export default Sidebar;
