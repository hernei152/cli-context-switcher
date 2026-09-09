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

## Cómo se limpia el log

`strip-ansi-escapes` no sirve para esto: un TUI no emite espacios, adelanta el cursor
con `\033[NC`, así que borrar la secuencia borra el espaciado que representaba. Además
repinta la pantalla entera y el texto sale duplicado N veces.

En vez de eso el log crudo se replayea por un emulador de terminal (`vt100`) y se vuelca
el scrollback resultante — que es exactamente lo que verías haciendo scroll hacia arriba
en tu terminal. Los frames viejos quedan sobreescritos, no repetidos.

```
strip-ansi-escapes:  WelcometoClaudeCodev2.1.266
vt100:               Welcome to Claude Code v2.1.266
```

## Limitaciones

- El replay usa el tamaño de la terminal actual, no el de la sesión grabada (no lo
  guardamos). Si la grabaste con otro ancho, el layout sale corrido.
- Una app que usa la pantalla alternativa (`vim`, `htop`) no deja scrollback: del log
  sale sólo el último frame.
