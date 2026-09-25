#!/usr/bin/env bash
# ponytail: única prueba runnable. Corre hernei bajo `script` (necesita un TTY real),
# verifica que el log se cree, que conserve los ANSI y que salgan limpios al retomar.
set -euo pipefail
BIN="$(dirname "$0")/target/release/hernei"
export HOME="$(mktemp -d)"

script -qec "$BIN bash -c 'printf \"\\033[31mhola-rojo\\033[0m\\n\"'" /dev/null >/dev/null

LOG=$(ls "$HOME"/.hernei/session_bash_*.txt)
grep -q $'\033\[31mhola-rojo' "$LOG"        || { echo "FAIL: el log no conservó los ANSI"; exit 1; }
grep -q "hola-rojo" <(cat "$LOG")           || { echo "FAIL: falta el texto"; exit 1; }

# el código de salida del comando envuelto tiene que propagarse
set +e; script -qec "$BIN bash -c 'exit 42' -n 'Arreglar login'" /dev/null >/dev/null; CODE=$?; set -e
[ "$CODE" = 42 ] || { echo "FAIL: exit code $CODE, esperaba 42"; exit 1; }
[ "$(ls "$HOME"/.hernei | wc -l)" = 2 ] || { echo "FAIL: la segunda sesión pisó a la primera"; exit 1; }
ls "$HOME"/.hernei/session_bash_*__Arreglar-login.txt >/dev/null || { echo "FAIL: falta el título de la sesión"; exit 1; }

# Claude reanuda su sesión nativa; Codex recibe el contexto como prompt inicial.
printf '\033[2mResume this session with:\033[22m\r\n\033[2mclaude --resume "Prueba nativa"\033[22m\r\n' > "$HOME/.hernei/session_claude_20990101_000000__Prueba.txt"
mkdir -p "$HOME/bin"
cat > "$HOME/bin/claude" <<'EOF'
#!/bin/sh
printf '%s\n' "$@" > "$HOME/args_claude.txt"
EOF
cat > "$HOME/bin/codex" <<'EOF'
#!/bin/sh
printf '%s\n' "$@" > "$HOME/args_codex.txt"
EOF
chmod +x "$HOME/bin/claude" "$HOME/bin/codex"
printf '\r' | PATH="$HOME/bin:$PATH" script -qec "$BIN claude -s" /dev/null > "$HOME/salida_claude.txt"
grep -qx -- '--resume' "$HOME/args_claude.txt" || { echo "FAIL: Claude no recibió --resume"; exit 1; }
grep -qx 'Prueba nativa' "$HOME/args_claude.txt" || { echo "FAIL: Claude no recibió el nombre nativo"; exit 1; }
if grep -q 'copiá y pegá' "$HOME/salida_claude.txt"; then echo "FAIL: Claude pide copiar y pegar"; exit 1; fi

printf 'historial para Codex\r\n' > "$HOME/.hernei/session_codex_20990101_000001__Prueba-codex.txt"
printf '\r' | PATH="$HOME/bin:$PATH" script -qec "$BIN codex -s" /dev/null > "$HOME/salida_retomar.txt"
grep -q 'Leé el historial de nuestra sesión anterior' "$HOME/args_codex.txt" || { echo "FAIL: Codex no recibió el prompt"; exit 1; }
if grep -q 'copiá y pegá' "$HOME/salida_retomar.txt"; then echo "FAIL: Codex pide copiar y pegar"; exit 1; fi
grep -l 'historial para Codex' "$HOME"/.hernei/contexto_*.txt >/dev/null || { echo "FAIL: falta el contexto limpio"; exit 1; }
[ "$(find "$HOME/.hernei" -maxdepth 1 -name 'session_codex_*__Prueba-codex.txt' | wc -l)" = 2 ] || { echo "FAIL: la sesión retomada cambió de nombre"; exit 1; }

echo "OK (PTY, nombres e inyección de contexto)"
rm -rf "$HOME"
