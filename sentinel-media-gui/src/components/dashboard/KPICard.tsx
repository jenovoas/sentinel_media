import React from 'react';
import { motion } from 'framer-motion';

interface KPICardProps {
    value: number | string;
    label: string;
    color?: 'blue' | 'green' | 'red' | 'white';
    className?: string;
}

const colorClasses = {
    blue: 'text-sentinel-blue hover:border-sentinel-blue/20',
    green: 'text-sentinel-green hover:border-sentinel-green/20',
    red: 'text-red-400 hover:border-red-400/20',
    white: 'text-white/80 hover:border-white/10'
};

export const KPICard: React.FC<KPICardProps> = ({
    value,
    label,
    color = 'white',
    className = ''
}) => {
    return (
        <motion.div
            whileHover={{ y: -4, scale: 1.02 }}
            className={`p-8 glass-card flex flex-col items-center text-center group ${colorClasses[color]} ${className}`}
        >
            <span className={`text-3xl font-black mb-2 font-mono ${color === 'white' ? 'text-white/80' : `text-sentinel-${color}`}`}>
                {value}
            </span>
            <span className="text-[10px] font-black text-white/20 uppercase tracking-[0.2em]">
                {label}
            </span>
        </motion.div>
    );
};
