# hernei

Wrapper de cero overhead para CLIs de LLM (`claude`, `codex`, lo que sea).
Graba la sesión interactiva sin romper el TUI y te deja retomar el contexto después.

## Uso

```sh
hernei claude                              # título automático: carpeta actual + comando
hernei -n "Arreglar login" claude          # título elegido por vos
hernei codex -s                            # elegís una sesión y Codex recibe el contexto
hernei claude -s                           # elegís una sesión y Claude recibe el contexto
```

El menú muestra el título y la fecha de cada sesión, con las más recientes primero.
La fecha también queda en el nombre del archivo para evitar colisiones. Poné `-n` o
`--name` **antes del comando**: los argumentos posteriores se pasan al CLI envuelto.
Las sesiones grabadas con versiones anteriores siguen apareciendo en el menú.

Con `-s`, hernei muestra el menú, limpia los ANSI y escribe un archivo
`~/.hernei/contexto_*.txt` propio de la sesión elegida. Para `codex`, pasa un prompt
inicial al CLI de `codex`, `claude` o `claude-ds`: el agente recibe la instrucción de
leer el archivo al abrir y esperar tu próximo pedido, sin copiar ni pegar nada.
Para otros comandos, hernei muestra el
prompt para pegarlo manualmente y también intenta copiarlo al portapapeles. La
nueva sesión se graba como siempre.

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
