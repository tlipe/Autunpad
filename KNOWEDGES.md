# Conhecimentos Técnicos - Autunpad

Este documento contém conhecimentos técnicos importantes adquiridos durante o desenvolvimento do Autunpad.

## Arquitetura

### Tauri + Web Frontend
- **Backend**: Rust com Tauri framework
- **Frontend**: HTML5 + CSS3 + JavaScript vanilla (sem frameworks)
- **Comunicação**: `window.__TAURI__.invoke()` para chamar comandos Rust
- **Segurança**: Validação de paths no backend para prevenir acesso não autorizado

### Estrutura de Arquivos
```
Autunpad/
├── src/
│   ├── main.rs          # Entry point + comandos Tauri
│   ├── executor.rs      # Execução de scripts com controle de processo
│   ├── file_ops.rs      # Operações de arquivo (ler/salvar)
│   ├── security.rs      # Validação de paths
│   └── runtime.rs       # Detecção de runtimes disponíveis
├── ui/
│   └── index.html       # Interface completa (HTML + CSS + JS inline)
└── tauri.conf.json      # Configuração Tauri
```

## Syntax Highlighting

### Sistema Token-Based
O highlighting usa um sistema de tokens para evitar problemas de sobreposição:

1. **Coleta**: Todos os padrões regex são executados e matches coletados como tokens `{start, end, cls, text}`
2. **Ordenação**: Tokens ordenados por posição, depois por tamanho (maior primeiro)
3. **Deduplicação**: Remove tokens sobrepostos (primeiro/maior match vence)
4. **Renderização**: Build do HTML em uma única passagem com escape de entidades HTML

### Problema Resolvido
**Antes**: `result.replace(regex, '<span>...</span>')` sequencial causava corrupção quando padrões subsequentes matchavam dentro de HTML gerado.

**Depois**: Sistema de tokens com tracking de posições usadas previne sobreposição.

## PowerShell & Batch Highlighting

### PowerShell (.ps1)
- **Comentários**: `#` (linha) e `<# ... #>` (bloco)
- **Strings**: `"..."` e `'...'`
- **Variáveis**: `$variableName`
- **Keywords**: `function`, `if`, `foreach`, `while`, `try`, `catch`, etc.
- **Cmdlets**: Padrão `Verb-Noun` como `Get-Process`, `Set-Location`
- **Operadores**: `-eq`, `-ne`, `-like`, `-match`, etc.

### Batch (.bat/.cmd)
- **Comentários**: `::` e `REM`
- **Strings**: `"..."`
- **Variáveis**: `%variable%` e `%%i` (em loops)
- **Keywords**: `echo`, `if`, `for`, `goto`, `call`, `exit`, `set`, etc.
- **Labels**: `:labelName`
- **Operadores**: `==`, `!=`, `EQU`, `NEQ`, `LSS`, `LEQ`, `GTR`, `GEQ`

## Tab & Indentação

### Implementação
```javascript
// Tab: insere 4 espaços
editor.value = v.substring(0, s) + '    ' + v.substring(en);

// Shift+Tab: remove até 4 espaços do início das linhas
const stripped = lineText.replace(/^( {1,4}|\t)/gm, '');

// Tab com seleção: indenta múltiplas linhas
const indented = lineText.replace(/^/gm, '    ');
```

### CSS
```css
tab-size: 4; /* Define largura visual do tab */
```

## Drag-and-Drop de Abas

### HTML5 Drag and Drop API
- `draggable="true"` no elemento
- `dragstart`: Define dados transferidos
- `dragover`: **Crítico** - deve chamar `e.preventDefault()` para permitir drop
- `drop`: Processa a reordenação
- `dragend`: Limpa classes visuais

### Problema Resolvido
**Erro**: Ícone de "proibido" ao arrastar sobre espaço vazio do container.

**Solução**: Adicionar `dragover` handler no **container**, não apenas nos elementos:
```javascript
ct.addEventListener('dragover', (e) => {
  e.preventDefault();
  e.dataTransfer.dropEffect = 'move';
});
```

## Execução de Scripts

### Controle de Processo
```rust
// Global para rastrear processo atual
static CURRENT_PROCESS: Mutex<Option<std::process::Child>> = Mutex::new(None);

// Executar: spawn + armazenar no global
let child = Command::new("cmd.exe").arg("/c").arg(path).spawn()?;
*CURRENT_PROCESS.lock()? = Some(child);

// Parar: taskkill força árvore de processos
taskkill /F /T /PID <pid>
child.kill();
```

### Detecção de Runtime
- `.ps1` → `powershell.exe` (sempre disponível no Windows)
- `.py` → `python` ou `python3` (verificar com `which`/`where`)
- `.sh` → `bash` (WSL ou Git Bash)
- `.bat/.cmd` → `cmd.exe` (sempre disponível no Windows)

## Segurança

### Validação de Paths
```rust
// Prevenir path traversal
if path.contains("..") || path.contains("~") {
    return Err("Path inválido".into());
}

// Whitelist de extensões executáveis
let allowed = ["bat", "cmd", "ps1", "py", "sh", "exe"];
if !allowed.contains(&ext) {
    return Err("Tipo não suportado".into());
}
```

### Validação de Nomes de Arquivo
```rust
// Windows: caracteres proibidos
const FORBIDDEN: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

// Não pode terminar com . ou espaço
if name.ends_with('.') || name.ends_with(' ') {
    return Err("Nome inválido".into());
}
```

## Auto-Save Antes de Executar

### Lógica
```javascript
if (!tab.path || tab.dirty) {
  await saveFile();  // Salva se não tem path ou foi modificado
  showSavedIndicator();
}
const r = await invoke('execute_file', { path: tab.path });
```

### Por que `tab.dirty`?
- `tab.content` é atualizado em tempo real via `input` event
- Mas o **arquivo no disco** só é atualizado quando `saveFile()` é chamado
- Sem essa verificação, scripts modificados executariam versão antiga

## Detecção de Linguagem

### Para Arquivos Salvos
```javascript
function detectLanguage(path) {
  const ext = path.split('.').pop().toLowerCase();
  const map = {
    js: 'javascript', py: 'python', ps1: 'powershell',
    bat: 'batch', cmd: 'batch', // ...
  };
  return map[ext] || 'plaintext';
}
```

### Para Arquivos Não Salvos
```javascript
// Usa título da aba ao invés de path
const lang = tab.format === 'script' 
  ? detectLanguage(tab.path || tab.title)
  : 'plaintext';
```

## Performance

### Otimizações
- **Debounce implícito**: `renderTabs()` chamado apenas quando necessário
- **Scroll sync**: Line numbers e highlight-layer sincronizados com editor
- **Token deduplication**: Evita renderizar spans sobrepostos
- **Regex caching**: Padrões definidos uma vez, reutilizados

## Debugging Tips

### Syntax Highlighting Quebrado
**Sintoma**: HTML aparece como texto (ex: `class="hl-string">`)

**Causa**: Regex subsequente matchando dentro de HTML gerado

**Solução**: Sistema de tokens com tracking de posições usadas

### Drag-and-Drop Não Funciona
**Sintoma**: Ícone de "proibido" ao arrastar

**Causa**: Falta `e.preventDefault()` no `dragover` do container

**Solução**: Adicionar handler de `dragover` no container pai

### Tab Não Insere Espaços
**Sintoma**: Tab move foco para próximo elemento

**Causa**: Falta `e.preventDefault()` no `keydown`

**Solução**:
```javascript
editor.addEventListener('keydown', e => {
  if (e.key === 'Tab') {
    e.preventDefault();
    // insere espaços
  }
});
```

## Build & Deploy

### Build Release
```bash
cargo build --release
```

Output: `target/release/autunpad.exe`

### Estrutura do Binário
- Single-file executable (~8-12 MB)
- Inclui webview2 runtime (Windows)
- Não requer instalação

### Distribuição
- Distribuir apenas `autunpad.exe`
- Usuário precisa de WebView2 runtime (incluso no Windows 10/11)
- Não requer permissões administrativas

## Convenções de Código

### JavaScript
- **Inline**: Todo código em `index.html` (single-file app)
- **Minificado**: Funções em uma linha quando possível
- **Naming**: camelCase para funções e variáveis

### Rust
- **Estrutura**: Módulos separados por responsabilidade
- **Error handling**: `Result<T, String>` para simplicidade
- **Concurrency**: `Mutex` para estado compartilhado

### CSS
- **Dark theme**: Background `#0a0a0a`, text `#e0e0e0`
- **Accent**: `#0078d4` (Windows blue)
- **Font**: JetBrains Mono, monospace

## Futuras Melhorias

### Possíveis Adições
- Multi-cursor editing
- Code folding
- Autocomplete
- Integração com Git
- Themes customizáveis
- Plugins/extensions
- Search em múltiplos arquivos
- Split view (múltiplos editores lado a lado)
- Terminal integrado
- Debug mode com breakpoints

### Otimizações
- Virtual scrolling para arquivos grandes
- Web Workers para highlighting pesado
- IndexedDB para cache de arquivos
- Compressão de histórico

## Referências

### Documentação
- [Tauri Docs](https://tauri.app/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [MDN Web Docs](https://developer.mozilla.org/)

### APIs Usadas
- `std::process::Command` - Execução de processos
- `std::sync::Mutex` - Sincronização
- `HTML5 Drag and Drop API` - Reordenação de abas
- `Selection API` - Manipulação de texto

### Ferramentas
- **Cargo**: Build system do Rust
- **PowerShell**: Shell padrão do Windows
- **taskkill**: Utilitário para matar processos
