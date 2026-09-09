# hernei

Wrapper de cero overhead para CLIs de LLM (`claude`, `codex`, lo que sea).
Graba la sesión interactiva sin romper el TUI y te deja retomar el contexto después.

## Uso

```sh
hernei claude          # graba en ~/.hernei/session_claude_YYYYMMDD_HHMMSS.txt
hernei claude -s       # elegís una sesión anterior y arranca con ese contexto
```

Con `-s`: menú de sesiones → limpia los ANSI → escribe `~/.hernei/contexto_para_llm.txt`
→ copia al portapapeles un prompt que apunta a ese archivo → lanza el LLM (grabando también).

## Cómo funciona

El comando corre dentro de un PTY (`portable-pty`), así que ve un TTY real y conserva
colores, TUI y tamaño de ventana. Dos hilos bombean el I/O: `stdin → pty` y
`pty → stdout + log`. El código de salida del comando envuelto se propaga.

## Build

```sh
cargo build --release
./test.sh              # smoke test del PTY (necesita `script`)
```

## Estado

El grabado anda. El `-s` todavía no: `strip-ansi-escapes` saca las secuencias pero no
las *interpreta*, y los TUI posicionan el cursor en vez de emitir espacios — el texto
resultante sale con el layout destruido. La solución es replayear el log por un emulador
de terminal (`vt100`) y volcar el scrollback. Pendiente.
