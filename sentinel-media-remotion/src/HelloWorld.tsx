import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';

export const HelloWorld: React.FC = () => {
    const frame = useCurrentFrame();
    const { fps } = useVideoConfig();

    // Simple fade in
    const opacity = Math.min(1, frame / 30);

    return (
        <AbsoluteFill style={{
            backgroundColor: '#0d1117',
            justifyContent: 'center',
            alignItems: 'center',
        }}>
            <div style={{
                fontFamily: 'sans-serif',
                fontSize: 100,
                color: '#58a6ff',
                opacity: opacity
            }}>
                SENTINEL CORTEX
            </div>
            <div style={{
                fontFamily: 'sans-serif',
                fontSize: 40,
                color: 'white',
                opacity: opacity,
                marginTop: 20
            }}>
                Powered by Remotion
            </div>
        </AbsoluteFill>
    );
};
