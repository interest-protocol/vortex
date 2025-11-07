#!/bin/bash

EXT_DIR="$HOME/.vscode/extensions/circom-syntax"

echo "Creating Circom syntax extension..."
mkdir -p "$EXT_DIR/syntaxes"

# Create package.json
cat > "$EXT_DIR/package.json" << 'EOF'
{
  "name": "circom-syntax",
  "displayName": "Circom Syntax",
  "description": "Syntax highlighting for Circom",
  "version": "1.0.0",
  "engines": { "vscode": "^1.60.0" },
  "categories": ["Programming Languages"],
  "contributes": {
    "languages": [{
      "id": "circom",
      "aliases": ["Circom", "circom"],
      "extensions": [".circom"],
      "configuration": "./language-configuration.json"
    }],
    "grammars": [{
      "language": "circom",
      "scopeName": "source.circom",
      "path": "./syntaxes/circom.tmLanguage.json"
    }]
  }
}
EOF

# Create language-configuration.json
cat > "$EXT_DIR/language-configuration.json" << 'EOF'
{
  "comments": {
    "lineComment": "//",
    "blockComment": ["/*", "*/"]
  },
  "brackets": [["{", "}"], ["[", "]"], ["(", ")"]],
  "autoClosingPairs": [
    { "open": "{", "close": "}" },
    { "open": "[", "close": "]" },
    { "open": "(", "close": ")" }
  ]
}
EOF

# Create syntax file (simplified version)
cat > "$EXT_DIR/syntaxes/circom.tmLanguage.json" << 'EOF'
{
  "scopeName": "source.circom",
  "patterns": [
    {
      "name": "comment.line.double-slash.circom",
      "match": "//.*$"
    },
    {
      "name": "keyword.control.circom",
      "match": "\\b(template|component|signal|var|input|output|public|pragma|include)\\b"
    },
    {
      "name": "keyword.operator.circom",
      "match": "(===|<==|==>|<--|-->)"
    }
  ]
}
EOF

echo "✓ Extension created at: $EXT_DIR"
echo ""
echo "Now:"
echo "1. Press Ctrl+Shift+P in VSCode"
echo "2. Type: 'Developer: Reload Window'"
echo "3. Open any .circom file"