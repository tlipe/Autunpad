# Changelog

Todas as mudanças notáveis deste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/), e este projeto adere ao [Semantic Versioning](https://semver.org/).

## [1.0.0] - 2026-07-09

### Adicionado
- Sistema de abas multi-documento com suporte a drag-and-drop para reordenar
- Syntax highlighting para múltiplas linguagens:
  - JavaScript, Python, Markdown, JSON, C/C++, Rust
  - PowerShell (.ps1) com suporte a cmdlets e variáveis
  - Batch (.bat/.cmd) com keywords e labels
- Editor de texto com:
  - Suporte a Tab (4 espaços) e Shift+Tab para indentação
  - Numeração de linhas
  - Contagem de palavras e caracteres em tempo real
  - Indicador de posição (linha/coluna)
- Sistema de execução de scripts:
  - Suporte a .bat, .cmd, .ps1, .py, .sh, .exe
  - Botão de parar execução (stop) com taskkill no Windows
  - Painel de output integrado
  - Auto-save antes de executar
- Funcionalidades de edição:
  - Find & Replace (Ctrl+H)
  - Undo/Redo (Ctrl+Z/Ctrl+Y)
  - Copiar/Colar/Recortar
- Sistema de arquivos:
  - Criar novo arquivo (Ctrl+N)
  - Abrir arquivo (Ctrl+O)
  - Salvar arquivo (Ctrl+S)
  - Renomear abas (duplo-clique no nome)
  - Validação de nomes de arquivo (caracteres proibidos)
- Interface:
  - Design escuro moderno
  - Status bar com informações em tempo real
  - Indicador visual de "Salvo" após auto-save
  - Indicadores visuais de drop durante drag-and-drop
- Backend Rust com:
  - Validação de segurança de paths
  - Detecção de runtime disponível
  - Execução assíncrona de processos
  - Sistema de timeout (30 segundos)
  - Kill de processos com taskkill (Windows)

### Corrigido
- Syntax highlighting com sistema de tokens para evitar sobreposição incorreta
- Drag-and-drop de abas funcionando corretamente (adicionado dragover no container)
- Sincronização de conteúdo entre editor e highlight-layer
- Auto-save de arquivos modificados antes de executar
- Detecção de linguagem para arquivos não salvos (usando título da aba)

### Melhorado
- Tab size aumentado de 2 para 4 espaços
- Sistema de highlighting token-based para prevenir corrupção de HTML
- Performance de renderização com atualização otimizada

## [0.1.0] - 2026-07-09

### Adicionado
- Estrutura inicial do projeto com Tauri + Rust
- Interface web básica com HTML/CSS/JavaScript
- Sistema de build com Cargo
- Documentação inicial em português
