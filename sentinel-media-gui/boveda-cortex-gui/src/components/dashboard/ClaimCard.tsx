import React from 'react';
import { motion } from 'framer-motion';
import {
    Activity,
    Zap,
    Shield,
    Fingerprint
} from 'lucide-react';

interface ClaimCardProps {
    name: string;
    status: string;
    active: boolean;
    value: string;
    icon_type: string;
}

const getIcon = (type: string) => {
    switch (type) {
        case 'truthsync': return Activity;
        case 'multimedia': return Zap;
        case 'ebpf': return Shield;
        case 'scanner': return Fingerprint;
        default: return Shield;
    }
};

export const ClaimCard: React.FC<ClaimCardProps> = ({
    name,
    status,
    active,
    value,
    icon_type
}) => {
    const Icon = getIcon(icon_type);

    return (
        <motion.div
            whileHover={{ y: -4, backgroundColor: 'rgba(255,255,255,0.03)' }}
            className="p-6 rounded-[2rem] glass-panel border border-white/5 flex flex-col items-start text-left group transition-all"
        >
            <div className={`p-3 rounded-2xl mb-4 ${active
                    ? 'bg-sentinel-green/10 text-sentinel-green'
                    : 'bg-sentinel-blue/10 text-sentinel-blue'
                }`}>
                <Icon size={20} />
            </div>
            <h3 className="text-[10px] font-black text-white/80 uppercase tracking-tight mb-1">
                {name}
            </h3>
            <p className="text-[9px] text-white/20 leading-tight mb-4 uppercase tracking-widest">
                {status}
            </p>
            <div className="mt-auto pt-4 border-t border-white/5 w-full">
                <span className={`text-[9px] font-mono font-bold ${active ? 'text-sentinel-green' : 'text-sentinel-blue/60'
                    }`}>
                    {value}
                </span>
            </div>
        </motion.div>
    );
};
