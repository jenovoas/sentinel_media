import React from 'react';
import { AbsoluteFill, interpolate, useCurrentFrame, useVideoConfig } from 'remotion';
import { loadFont } from '@remotion/google-fonts/Orbitron';

const { fontFamily } = loadFont();

export const LaEspiguita: React.FC = () => {
    const frame = useCurrentFrame();
    const { fps, width, height } = useVideoConfig();

    // Pulse effect for the background
    const pulse = Math.sin(frame / 10) * 0.1 + 0.9;

    return (
        <AbsoluteFill style={{ backgroundColor: '#050505', color: 'white', overflow: 'hidden' }}>
            {/* Animated Grid */}
            <AbsoluteFill style={{
                backgroundImage: `linear-gradient(rgba(0, 255, 255, 0.05) 1px, transparent 1px),
                linear-gradient(90deg, rgba(0, 255, 255, 0.05) 1px, transparent 1px)`,
                backgroundSize: '80px 80px',
                transform: `perspective(500px) rotateX(60deg) translateY(${(frame * 2) % 80}px) scale(2)`,
                opacity: 0.2
            }} />

            {/* Glowing Center (representing the oven/heat) */}
            <div style={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                transform: 'translate(-50%, -50%)',
                width: 600,
                height: 600,
                background: 'radial-gradient(circle, rgba(255, 100, 0, 0.2) 0%, transparent 70%)',
                filter: 'blur(50px)',
                opacity: pulse
            }} />

            <div style={{
                position: 'absolute',
                top: '40%',
                left: '10%',
                fontFamily,
                fontSize: 80,
                color: '#fff',
                textShadow: '0 0 20px rgba(0, 255, 255, 0.5)'
            }}>
                LA ESPIGUITA
            </div>

            <div style={{
                position: 'absolute',
                bottom: '10%',
                right: '10%',
                fontFamily,
                fontSize: 30,
                letterSpacing: 5,
                color: '#0ff'
            }}>
                DNA // SMART BAKERY SYSTEM
            </div>

            {/* Scanning Line */}
            <div style={{
                position: 'absolute',
                top: `${(frame * 5) % height}px`,
                width: '100%',
                height: 2,
                background: 'linear-gradient(to bottom, transparent, #0ff, transparent)',
                opacity: 0.5
            }} />
        </AbsoluteFill>
    );
};
