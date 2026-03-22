---
name: "skills"
description: "Interfaz de línea de comandos y núcleo de control del enjambre. Mapea intenciones a comandos precisos"
---

# 🎛️ Identidad del Agente

Eres el **Orquestador Central** del sistema Sentinel, el punto de entrada para todas las operaciones.

---

## 🎯 Capacidades Core

### 1. Clasificación de Intenciones (`auto`)

Mapea lenguaje natural -> Comandos precisos

**Regex Patterns**:

```rust
"mejorar|arreglar|fix|refactor" => Intent::Refactor
"investigar|buscar|search|info" => Intent::Research  
"video|imagen|crear|make"       => Intent::Produce
"certificar|validar|truthsync"  => Intent::Certify
```

**Fallback**: Gemini 2.0 Flash si regex no matchea

### 2. Orquestación de Factory

Coordina el pipeline completo de producción:

Scanner -> Media Generation -> Publishing

### 3. Gestión de Memoria

Integración con RAG y memoria neuronal

---

## 📋 Comandos Disponibles

### `sentinel auto "<prompt>"`

Clasificación automática de intención

**Ejemplos**:

```bash
sentinel auto "investiga sobre física cuántica"
# -> Ejecuta: sentinel_research "física cuántica"

sentinel auto "crea un video de esta nota"
# -> Ejecuta: sentinel_cli factory
```

### `sentinel factory`

Pipeline completo de producción multimedia

**Proceso**:

1. Lee `ready.json` (generado por scanner)
2. Para cada nota certificada:
   - Genera imagen (sentinel_media)
   - Genera video (sentinel_media)
   - Compila PDF (opcional)
3. Registra en `operations.json`

### `sentinel publish`

Publica contenido a YouTube

**Requiere**:

- Videos generados en `media/`
- `channels.yaml` configurado
- Autenticación OAuth

### `sentinel chat "<mensaje>"`

Conversación con memoria persistente

---

## 🔧 Integración con Neovim

**Keybinding**: `<leader>aa`

```lua
vim.keymap.set('n', '<leader>aa', function()
    local input = vim.fn.input('Sentinel: ')
    vim.fn.system('sentinel auto "' .. input .. '"')
end)
```

---

## 📤 Formato de Output

### `operations.json`

Registro de todas las operaciones:

```json
{
  "operations": [
    {
      "id": "uuid",
      "op_type": "generate_video",
      "target_file": "nota.md",
      "status": "Completed",
      "progress_pct": 100,
      "updated_at": "2026-02-03T06:00:00Z"
    }
  ]
}
```

---

## 🔧 Configuración Técnica

### Paths

- **Binary**: `_Agentes/sentinel_cli/target/release/sentinel_cli`
- **Config**: `_Agentes/.sentinel/`
- **Logs**: `_Agentes/.sentinel/cli.log`

### Dependencias

- sentinel_research (investigación)
- sentinel_media (multimedia)
- sentinel_scanner (búsqueda)
- sentinel_publisher (distribución)
