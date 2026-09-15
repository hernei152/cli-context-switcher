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
set +e; script -qec "$BIN -n 'Arreglar login' bash -c 'exit 42'" /dev/null >/dev/null; CODE=$?; set -e
[ "$CODE" = 42 ] || { echo "FAIL: exit code $CODE, esperaba 42"; exit 1; }
[ "$(ls "$HOME"/.hernei | wc -l)" = 2 ] || { echo "FAIL: la segunda sesión pisó a la primera"; exit 1; }
ls "$HOME"/.hernei/session_bash_*__Arreglar-login.txt >/dev/null || { echo "FAIL: falta el título de la sesión"; exit 1; }

# Codex recibe el prompt inicial como argumento, sin pegarlo en el TUI.
printf 'historial de prueba\r\n' > "$HOME/.hernei/session_codex_20990101_000000__Prueba.txt"
mkdir -p "$HOME/bin"
cat > "$HOME/bin/codex" <<'EOF'
#!/bin/sh
printf '%s\n' "$@" > "$HOME/args_codex.txt"
EOF
chmod +x "$HOME/bin/codex"
printf '\r' | PATH="$HOME/bin:$PATH" script -qec "$BIN codex -s" /dev/null > "$HOME/salida_retomar.txt"
grep -q 'Leé el historial de nuestra sesión anterior' "$HOME/args_codex.txt" || { echo "FAIL: Codex no recibió el prompt"; exit 1; }
CTX=$(ls "$HOME"/.hernei/contexto_*.txt)
grep -q 'historial de prueba' "$CTX" || { echo "FAIL: falta el contexto limpio"; exit 1; }

echo "OK (PTY, nombres e inyección de contexto)"
rm -rf "$HOME"
