# Autunpad

Editor de texto desktop multi-abas com execução de scripts integrada.

## Recursos

- Multi-abas com drag-and-drop
- Explorador de arquivos na sidebar
- Execução de scripts (Python, PowerShell, Batch, Shell)
- Terminal integrado com múltiplas sessões
- Syntax highlighting (JavaScript, Python, Markdown, JSON, C, Rust)
- Buscar e substituir (Ctrl+H)
- Histórico de arquivos salvos
- Configurações persistentes (tamanho da fonte, quebra de linha, números de linha)

## Atalhos

| Atalho | Ação |
|--------|------|
| Ctrl+N | Nova aba |
| Ctrl+O | Abrir arquivo |
| Ctrl+S | Salvar arquivo |
| Ctrl+H | Buscar e substituir |
| F5 | Executar script |

## Tecnologias

- **Backend**: Rust + Tauri 2
- **Frontend**: HTML/CSS/JS vanilla
- **Diálogos**: rfd
- **Build**: Cargo

## Desenvolvimento

```bash
# Instalar CLI Tauri 2 (local)
npm install

# Executar em modo dev
npx tauri dev

# Build release
npx tauri build
```

> **Nota:** o `cargo tauri` global na máquina pode estar na v1. Use `npx tauri` (v2) neste projeto.

## Estrutura

```
src/
  main.rs        # Comandos Tauri e setup
  file_ops.rs    # Operações de arquivo
  security.rs    # Validação de paths e extensões
  executor.rs    # Execução de scripts em paralelo
ui/
  index.html     # Interface completa
```
