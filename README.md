# hernei

Wrapper de cero overhead para CLIs de LLM (`claude`, `codex`, lo que sea).
Graba la sesión interactiva sin romper el TUI y te deja retomar el contexto después.

## Instalación

Necesitás [Git](https://git-scm.com/) y una instalación de Rust que incluya `cargo`.

```sh
git clone https://github.com/hernei152/cli-context-switcher.git
cd cli-context-switcher
cargo build --release
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/hernei "$HOME/.local/bin/hernei"
```

Comprobá que quedó instalado:

```sh
hernei --version
```

Si la terminal no encuentra el comando, agregá `~/.local/bin` al `PATH` de tu shell:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

Para actualizar una instalación existente:

```sh
cd cli-context-switcher
git pull
cargo build --release
install -m 755 target/release/hernei "$HOME/.local/bin/hernei"
```

## Uso

```sh
hernei claude                              # título automático: carpeta actual + comando
hernei -n "Arreglar login" claude          # título elegido por vos
hernei claude -n "Arreglar login"          # también funciona después del comando
hernei codex -s                            # elegís una sesión y Codex recibe el contexto
hernei claude -s                           # elegís una sesión y Claude recibe el contexto
```

El menú muestra el título y la fecha de cada sesión, con las más recientes primero.
La fecha también queda en el nombre del archivo para evitar colisiones. Podés poner
`-n` o `--name` antes o después del comando. Si necesitás pasarle `-n` al CLI envuelto,
separá sus argumentos con `--`, por ejemplo `hernei claude -- -n "nombre en Claude"`.
Las sesiones grabadas con versiones anteriores siguen apareciendo en el menú.

Con `-s`, hernei muestra el menú. Para `claude` y `claude-ds`, usa la sesión nativa
que Claude guarda y el comando `claude --resume`, así se conserva la conversación
completa aunque el TUI limpie la pantalla al salir. Los nombres elegidos con `-n`
también se asignan a la sesión nativa de Claude.

Para `codex` y otros comandos, hernei limpia los ANSI y escribe un archivo
`~/.hernei/contexto_*.txt` propio de la sesión elegida. Codex recibe automáticamente
la instrucción de leerlo; con comandos no reconocidos, hernei muestra el prompt para
pegarlo manualmente y también intenta copiarlo al portapapeles. La nueva sesión se
graba como siempre.

## Cómo funciona

El comando corre dentro de un PTY (`portable-pty`), así que ve un TTY real y conserva
colores, TUI y tamaño de ventana. Dos hilos bombean el I/O: `stdin → pty` y
`pty → stdout + log`. El código de salida del comando envuelto se propaga.

## Desarrollo

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
  sale sólo el último frame. Claude se recupera mediante su sesión nativa.
