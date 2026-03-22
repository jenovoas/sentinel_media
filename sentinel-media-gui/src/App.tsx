import React, { useState } from 'react';
import Sidebar from './components/Sidebar';
import Dashboard from './components/Dashboard';
import VaultView from './components/VaultView';
import Chat from './components/Chat';
import SwarmView from './components/SwarmView';
import FactoryView from './components/FactoryView';
import SystemConsole from './components/SystemConsole';
import ResearchView from './components/ResearchView';
import CommandPalette from './components/CommandPalette';
import SettingsView from './components/SettingsView';
import CognitiveLayer from './components/CognitiveLayer';
import SentinomicsView from './components/SentinomicsView';
import ProductionView from './components/ProductionView';
import VaultMap from './components/VaultMap';
import { motion, AnimatePresence } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';

const App: React.FC = () => {
  const [currentView, setCurrentView] = useState('observe');

  // --- Sistema de Bio-Sync (Human Pulse Injection) ---
  React.useEffect(() => {
    let lastPulse = 0;
    const PULSE_THROTTLE_MS = 1000; // Un pulso por segundo max

    const handleActivity = async () => {
      const now = Date.now();
      if (now - lastPulse > PULSE_THROTTLE_MS) {
        lastPulse = now;
        try {
          await invoke('inject_bio_pulse');
          // No logueamos para no saturar la consola del usuario, 
          // pero el backend de Fenix lo registrará.
        } catch (e) {
          console.error("Fallo en Bio-Sync Bridge:", e);
        }
      }
    };

    window.addEventListener('keydown', handleActivity);
    window.addEventListener('mousedown', handleActivity);

    return () => {
      window.removeEventListener('keydown', handleActivity);
      window.removeEventListener('mousedown', handleActivity);
    };
  }, []);

  const renderView = () => {
    switch (currentView) {
      case 'observe': return <Dashboard setView={setCurrentView} />;
      case 'production': return <ProductionView />;
      case 'dialog': return <Chat />;
      case 'swarm': return <SwarmView />;
      case 'vault': return <VaultView />;
      case 'factory': return <FactoryView />;
      case 'hacker': return <SystemConsole />;
      case 'research': return <ResearchView />;
      case 'sentinomics': return <SentinomicsView />;
      case 'cognitive': return <CognitiveLayer />;
      case 'settings': return <SettingsView />;
      case 'commands': return <CommandPalette />;
      case 'map': return <VaultMap />;
      default: return <Dashboard />;
    }
  };

  return (
    <div className="flex h-screen bg-cyber-dark text-white overflow-hidden font-sans selection:bg-sentinel-blue/30 antialiased selection:text-white">
      <Sidebar currentView={currentView} setView={setCurrentView} />
      <main className="flex-1 relative overflow-hidden">
        {/* Orbs de fondo dinámicos ya definidos en CSS, pero podemos añadir ruido aquí */}
        <div className="absolute inset-0 bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-[0.02] pointer-events-none" />
        <AnimatePresence mode="wait">
          <motion.div
            key={currentView}
            initial={{ opacity: 0, scale: 0.98, filter: "blur(10px)" }}
            animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }}
            exit={{ opacity: 0, scale: 1.02, filter: "blur(10px)" }}
            transition={{ duration: 0.4, ease: [0.22, 1, 0.36, 1] }}
            className="h-full"
          >
            {renderView()}
          </motion.div>
        </AnimatePresence>
      </main>
    </div>
  );
};

export default App;
