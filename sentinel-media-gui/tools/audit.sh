#!/bin/bash
# Sentinel Cortex - Auditoría Automatizada
# Para integración con NotebookLM y Agentes IA

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORTS_DIR="$PROJECT_ROOT/reports"

mkdir -p "$REPORTS_DIR"

# Argument Parsing
APPLY_FIXES=false
for arg in "$@"; do
    case $arg in
        --fix)
            APPLY_FIXES=true
            shift
            ;;
    esac
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 SENTINEL CORTEX - AUDIT"
if [ "$APPLY_FIXES" = true ]; then
    echo "🔧 MODO REPARACIÓN ACTIVADO (--fix)"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "📐 Fase 1: Formateo automático (cargo fmt)..."
cd "$PROJECT_ROOT/src-tauri"
cargo fmt --all --quiet
echo "   ✅ Formato aplicado correctamente"
echo ""

if [ "$APPLY_FIXES" = true ]; then
    echo "🔧 Fase 1.5: Aplicando correcciones automáticas (cargo clippy --fix)..."
    cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features -- -D warnings || true
    echo "   ✅ Correcciones aplicadas (revisar diff)"
    echo ""
fi

# 2. Análisis Clippy (pedantic mode)
echo "🧠 Fase 2: Análisis estático profundo (Clippy)..."
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee "$REPORTS_DIR/clippy_report.txt"
CLIPPY_EXIT=${PIPESTATUS[0]}
if [ $CLIPPY_EXIT -eq 0 ]; then
    echo "   ✅ Clippy: Sin warnings críticos"
else
    echo "   ⚠️  Clippy: Revisa $REPORTS_DIR/clippy_report.txt"
fi
echo ""

# 3. Auditoría de seguridad
echo "🛡️  Fase 3: Auditoría de vulnerabilidades (cargo-audit)..."
if command -v cargo-audit &> /dev/null; then
    cargo audit 2>&1 | tee "$REPORTS_DIR/security_audit.txt"
    echo "   ✅ Reporte de seguridad generado"
else
    echo "   ⚠️  cargo-audit no instalado. Ejecuta: cargo install cargo-audit"
fi
echo ""

# 4. Tests
echo "🧪 Fase 4: Ejecutando tests..."
cargo test --all-features --quiet 2>&1 | tee "$REPORTS_DIR/test_results.txt"
TEST_EXIT=${PIPESTATUS[0]}
if [ $TEST_EXIT -eq 0 ]; then
    echo "   ✅ Tests pasaron correctamente"
else
    echo "   ❌ Tests fallaron. Revisa $REPORTS_DIR/test_results.txt"
fi
echo ""

# 5. Métricas de código (si tokei está disponible)
echo "📊 Fase 5: Generando métricas de código..."
if command -v tokei &> /dev/null; then
    tokei "$PROJECT_ROOT/src-tauri" 2>&1 | tee "$REPORTS_DIR/code_metrics.txt"
    echo "   ✅ Métricas generadas"
else
    echo "   ℹ️  tokei no instalado (opcional). Instala con: cargo install tokei"
fi
echo ""

# 6. Dependencias desactualizadas
echo "🔄 Fase 6: Verificando dependencias..."
if command -v cargo-outdated &> /dev/null; then
    cargo outdated 2>&1 | tee "$REPORTS_DIR/outdated_deps.txt"
    echo "   ✅ Reporte de dependencias generado"
else
    echo "   ℹ️  cargo-outdated no instalado (opcional)"
fi
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ AUDITORÍA COMPLETADA"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📁 Reportes disponibles en: $REPORTS_DIR/"
echo ""
echo "Reportes generados:"
echo "  - clippy_report.txt (análisis de código)"
echo "  - security_audit.txt (vulnerabilidades)"
echo "  - test_results.txt (resultados de tests)"
echo "  - code_metrics.txt (estadísticas)"
echo ""
echo "💡 Alimenta estos reportes a NotebookLM para análisis con IA"
echo ""

exit 0
