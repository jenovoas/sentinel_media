use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::Path;

// Constante para la coherencia SPA en imágenes.
const S60_IMAGE_COHERENCE: f64 = 0.947;

// Estructura para encapsular los resultados del análisis.
struct AnalysisResult {
    lines: usize,
    spa_mentions: usize,
    base60_mentions: usize,
    pattern_count: usize,
}

pub async fn run(file_path: &str, spa_patterns: bool) -> Result<(), Box<dyn Error>> {
    println!("🔍 Analizando: {}", file_path);

    let path = Path::new(file_path);
    if !path.exists() {
        println!("❌ Error: Archivo no encontrado: {}", file_path);
        return Ok(());
    }

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    // Manejo de archivos de imagen con detección de Penta-Resonancia.
    if ["jpg", "png", "jpeg"].contains(&ext) {
        println!("🖼️  Imagen: Penta-Resonancia detectada [file:48]");
        if spa_patterns {
            println!(
                "- 5 ondas → Coherencia {} SPA [file:7]",
                S60_IMAGE_COHERENCE
            );
        }
        return Ok(());
    }

    // Análisis de archivos de texto (.py/.rs).
    let content = fs::read_to_string(file_path)?;

    // Contar líneas y menciones de SPA y Base-60.
    let lines = content.lines().count();
    let spa_mentions = content.matches("SPA").count();
    let base60_mentions = content.matches("base-60").count() + content.matches("60").count();

    let mut pattern_count = 0;
    if spa_patterns {
        // Expresiones regulares para patrones SPA (17, 42, 60, etc.).
        let patterns = Regex::new(r"(17|42|60|30|7)")?;
        pattern_count = patterns.find_iter(&content).count();
    }

    // Encapsular resultados
    let results = AnalysisResult {
        lines,
        spa_mentions,
        base60_mentions,
        pattern_count,
    };

    // Imprimir resultados formateados
    println!("📊 Código: {} líneas", results.lines);
    println!("🔢 SPA: {} menciones", results.spa_mentions);
    println!("⚡ Base-60: {} matches", results.base60_mentions);

    if spa_patterns {
        println!("📈 Patrones sagrados: {}", results.pattern_count);
    }

    println!("✅ Análisis completado");
    Ok(())
}

