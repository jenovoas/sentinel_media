#!/bin/bash

echo "🔍 Buscando procesos de Tauri existentes..."

# Matar todos los procesos relacionados con tauri dev
pkill -f "tauri dev" 2>/dev/null
pkill -f "boveda-cortex-gui" 2>/dev/null
pkill -f "vite.*1420" 2>/dev/null

# Esperar un momento para que los procesos terminen
sleep 2

echo "✅ Procesos anteriores eliminados"
echo "🚀 Iniciando Tauri dev server..."

# Limpiar build cache si es necesario
cd "$(dirname "$0")"

# Iniciar el dev server
npm run tauri dev
