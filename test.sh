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
set +e; script -qec "$BIN bash -c 'exit 42'" /dev/null >/dev/null; CODE=$?; set -e
[ "$CODE" = 42 ] || { echo "FAIL: exit code $CODE, esperaba 42"; exit 1; }
[ "$(ls "$HOME"/.hernei | wc -l)" = 2 ] || { echo "FAIL: la segunda sesión pisó a la primera"; exit 1; }

echo "OK ($(ls "$HOME"/.hernei | wc -l) sesiones grabadas)"
rm -rf "$HOME"
