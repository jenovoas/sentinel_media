use anyhow::{Context, Result};
use std::fs;

/// Carga un archivo SKILL.md y retorna el contenido markdown sin el frontmatter YAML
///
/// # Formato esperado
/// ```markdown
/// ---
/// name: "Nombre del Skill"
/// description: "Descripción"
/// ---
///
/// # Contenido markdown...
/// ```
///
/// # Argumentos
/// * `skill_path` - Ruta relativa o absoluta al archivo SKILL.md
///
/// # Retorna
/// El contenido markdown limpio (sin el frontmatter YAML)
pub fn load_skill(skill_path: &str) -> Result<String> {
    // Leer archivo completo
    let content = fs::read_to_string(skill_path)
        .with_context(|| format!("No se pudo leer SKILL.md en: {}", skill_path))?;

    // Parsear frontmatter YAML
    let parts: Vec<&str> = content.split("---").collect();

    if parts.len() < 3 {
        anyhow::bail!("Formato inválido de SKILL.md. Se esperaba frontmatter YAML entre ---");
    }

    // Retornar contenido markdown (índice 2, después del segundo ---)
    Ok(parts[2].trim().to_string())
}

/// Carga un SKILL.md desde el directorio del agente actual
///
/// Asume que el binario está en `target/release/` y el SKILL.md en `skills/SKILL.md`
///
/// # Ejemplo
/// ```rust,ignore
/// use sentinel_media_core::load_agent_skill;
/// let skill = load_agent_skill()?;
/// ```
pub fn load_agent_skill() -> Result<String> {
    // Obtener directorio del ejecutable
    let exe_path = std::env::current_exe().context("No se pudo obtener ruta del ejecutable")?;
    let mut current_dir = exe_path.parent();

    // Buscar hacia arriba en la jerarquía de directorios por la carpeta 'skills'
    while let Some(dir) = current_dir {
        let skill_path = dir.join("skills/SKILL.md");
        if skill_path.exists() {
            return load_skill(skill_path.to_str().context("La ruta al skill contiene caracteres inválidos")?);
        }
        current_dir = dir.parent();
    }

    anyhow::bail!("No se pudo encontrar el directorio 'skills' subiendo desde la ruta del ejecutable.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_skill_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "---\nname: \"Test Skill\"\ndescription: \"Test\"\n---\n\n# Test Content\n\nThis is a test."
        )
        .unwrap();

        let result = load_skill(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let content = result.unwrap();
        assert!(content.contains("# Test Content"));
        assert!(!content.contains("name:"));
    }

    #[test]
    fn test_load_skill_invalid_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# Just markdown without frontmatter").unwrap();

        let result = load_skill(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}
