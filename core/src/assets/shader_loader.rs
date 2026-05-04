use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

const SHADER_DIRECTORIES: &[&str] = &[
    "res/shaders",
    "../core/res/shaders",
    "core/res/shaders",
];

pub fn load_shader_bytes(name: &str) -> Result<Vec<u8>> {
    let path = resolve_shader_path(name)
        .with_context(|| format!("Shader '{}' was not found in app or core shader directories", name))?;

    if path.extension().and_then(|e| e.to_str()) == Some("spv") {
        return fs::read(&path)
            .with_context(|| format!("Failed to read SPIR-V shader file {}", path.display()));
    }

    let source = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read shader source file {}", path.display()))?;
    compile_shader(&path, &source)
}

fn resolve_shader_path(name: &str) -> Option<PathBuf> {
    let requested = Path::new(name);
    let stem = requested.file_stem()?.to_str()?;
    let shader_paths = if requested.extension().and_then(|e| e.to_str()) == Some("spv") {
        vec![format!("{stem}.spv")]
    } else {
        vec![format!("{stem}.spv"), name.to_string()]
    };

    for dir in SHADER_DIRECTORIES {
        for shader_name in &shader_paths {
            let candidate = Path::new(dir).join(shader_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}

fn compile_shader(path: &Path, source: &str) -> Result<Vec<u8>> {
    let mut compiler = shaderc::Compiler::new()
        .context("Failed to create shader compiler")?;
    let options = shaderc::CompileOptions::new()
        .context("Failed to create shader compiler options")?;
    let kind = shader_kind_from_path(path)?;
    let output = compiler
        .compile_into_spirv(source, kind, path.file_name().unwrap().to_str().unwrap(), "main", Some(&options))
        .with_context(|| format!("Failed to compile shader {}", path.display()))?;
    Ok(output.as_binary_u8().to_vec())
}

fn shader_kind_from_path(path: &Path) -> Result<shaderc::ShaderKind> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("vert") => Ok(shaderc::ShaderKind::Vertex),
        Some("frag") => Ok(shaderc::ShaderKind::Fragment),
        Some(ext) => anyhow::bail!("Unsupported shader extension '{}', expected .vert or .frag", ext),
        None => anyhow::bail!("Shader path {} has no extension", path.display()),
    }
}
