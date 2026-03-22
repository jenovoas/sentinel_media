import React from 'react';
import { AbsoluteFill, Series, interpolate, useCurrentFrame, useVideoConfig, Audio, Img, Video } from 'remotion';
import { loadFont } from '@remotion/google-fonts/Orbitron';
import { SentinelManifest } from './types';

// Load Cyberpunk font
const { fontFamily } = loadFont();

const GlitchText: React.FC<{ text: string; fontSize?: number; color?: string }> = ({ text, fontSize = 80, color = '#0ff' }) => {
    const frame = useCurrentFrame();
    const opacity = interpolate(frame, [0, 10], [0, 1]);
    const glitchOffset = Math.sin(frame / 2) * 5;

    return (
        <div style={{ position: 'relative', fontFamily, fontSize, color, opacity }}>
            <div style={{ position: 'absolute', transform: `translate(${glitchOffset}px, 0)`, opacity: 0.7, color: '#f0f' }}>{text}</div>
            <div style={{ position: 'absolute', transform: `translate(${-glitchOffset}px, 0)`, opacity: 0.7, color: '#0ff' }}>{text}</div>
            <div style={{ position: 'relative', color: 'white' }}>{text}</div>
        </div>
    );
};

export const CyberCortex: React.FC<{ manifest: SentinelManifest }> = ({ manifest }) => {
    const { fps } = useVideoConfig();
    const frame = useCurrentFrame();

    // Background Grid Animation
    const gridMove = (frame * 2) % 100;

    return (
        <AbsoluteFill style={{ backgroundColor: '#050505', color: 'white', overflow: 'hidden' }}>

            {/* Animated Grid Background */}
            <AbsoluteFill style={{
                backgroundImage: `linear-gradient(rgba(0, 255, 255, 0.1) 1px, transparent 1px),
                linear-gradient(90deg, rgba(0, 255, 255, 0.1) 1px, transparent 1px)`,
                backgroundSize: '100px 100px',
                transform: `perspective(500px) rotateX(60deg) translateY(${gridMove}px) scale(2)`,
                opacity: 0.3
            }} />

            <Series>
                {/* INTRO */}
                <Series.Sequence durationInFrames={4 * fps}>
                    <AbsoluteFill style={{ justifyContent: 'center', alignItems: 'center' }}>
                        <GlitchText text="SENTINEL CORTEX" fontSize={100} />
                        <div style={{
                            fontFamily,
                            marginTop: 30,
                            fontSize: 30,
                            letterSpacing: 10,
                            color: '#0ff',
                            opacity: interpolate(frame, [20, 50], [0, 1])
                        }}>
                            SYSTEM INITIALIZED
                        </div>
                        <div style={{
                            fontFamily,
                            marginTop: 80,
                            fontSize: 40,
                            color: 'white',
                            border: '2px solid #0ff',
                            padding: '10px 40px',
                            background: 'rgba(0,255,255,0.1)'
                        }}>
                            {manifest.title.toUpperCase()}
                        </div>
                    </AbsoluteFill>
                </Series.Sequence>

                {/* CONTENT SEQUENCE */}
                {manifest.clips.map((clip, index) => {
                    const durationFrames = clip.durationInSec * fps;
                    return (
                        <Series.Sequence key={index} durationInFrames={durationFrames}>
                            <AbsoluteFill>
                                {clip.type === 'video' && clip.src && <Video src={clip.src} style={{ width: '100%', height: '100%', objectFit: 'cover' }} />}
                                {clip.type === 'image' && clip.src && (
                                    <Img
                                        src={clip.src}
                                        style={{
                                            width: '100%',
                                            height: '100%',
                                            objectFit: 'cover',
                                            transform: `scale(${interpolate(frame, [0, durationFrames], [1, 1.1])})` // Slow zoom
                                        }}
                                    />
                                )}

                                {/* Overlay / HUD Elements */}
                                <AbsoluteFill style={{ padding: 40, justifyContent: 'space-between' }}>
                                    <div style={{ display: 'flex', justifyContent: 'space-between', fontFamily, fontSize: 20, color: '#0ff' }}>
                                        <div>REC • [ {index + 1} / {manifest.clips.length} ]</div>
                                        <div>T-{frame}</div>
                                    </div>

                                    {/* Lower Thirds Text */}
                                    {clip.text && (
                                        <div style={{
                                            background: 'rgba(0,0,0,0.8)',
                                            padding: 20,
                                            borderLeft: '5px solid #0ff',
                                            maxWidth: '80%'
                                        }}>
                                            <h3 style={{ margin: 0, fontFamily, fontSize: 35 }}>{clip.text}</h3>
                                        </div>
                                    )}
                                </AbsoluteFill>
                            </AbsoluteFill>
                        </Series.Sequence>
                    );
                })}

                {/* OUTRO */}
                <Series.Sequence durationInFrames={3 * fps}>
                    <AbsoluteFill style={{ backgroundColor: 'black', justifyContent: 'center', alignItems: 'center' }}>
                        <GlitchText text="END TRANSMISSION" fontSize={60} />
                        <div style={{ fontFamily, color: '#555', marginTop: 20 }}>SENTINEL v8.0</div>
                    </AbsoluteFill>
                </Series.Sequence>
            </Series>

            {/* Global Audio Track (Optional) */}
            {manifest.audioUrl && <Audio src={manifest.audioUrl} volume={0.5} />}
        </AbsoluteFill>
    );
};
