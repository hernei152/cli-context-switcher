use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use arboard::Clipboard;
use clap::Parser;
use crossterm::terminal;
use inquire::Select;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

#[derive(Parser)]
#[command(
    name = "hernei",
    version,
    about = "Envuelve un CLI de LLM en un PTY, graba la sesión y te deja retomarla",
    trailing_var_arg = true
)]
struct Cli {
    /// Elegir una sesión anterior y arrancar con ese contexto
    #[arg(short, long)]
    session: bool,

    /// Comando a envolver, p.ej. `claude` o `codex --model x`
    #[arg(required = true, allow_hyphen_values = true)]
    cmd: Vec<String>,
}

fn main() -> Result<()> {
    // ponytail: la spec usa `hernei claude -s`, pero trailing_var_arg le pasaría
    // el -s al comando envuelto. Lo sacamos antes de que clap lo vea.
    let mut argv: Vec<String> = std::env::args().collect();
    let flag_al_final = matches!(argv.last().map(String::as_str), Some("-s" | "--session"));
    if flag_al_final {
        argv.pop();
    }
    let cli = Cli::parse_from(argv);

    let codigo = if cli.session || flag_al_final {
        retomar(&cli.cmd)?
    } else {
        grabar(&cli.cmd)?
    };
    std::process::exit(codigo);
}

fn dir_hernei() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("no encuentro $HOME")?;
    let dir = PathBuf::from(home).join(".hernei");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Deja la terminal usable aunque el proceso se caiga a mitad de camino.
struct ModoRaw;

impl ModoRaw {
    fn activar() -> Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(ModoRaw)
    }
}

impl Drop for ModoRaw {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

fn tam_actual() -> PtySize {
    let (cols, rows) = terminal::size().unwrap_or((80, 24));
    PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }
}

/// Corre `cmd` dentro de un PTY, escupiendo todo a stdout y a un log en ~/.hernei/.
fn grabar(cmd: &[String]) -> Result<i32> {
    let dir = dir_hernei()?;
    let nombre = PathBuf::from(&cmd[0])
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "cmd".into());
    let sello = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

    // Dos sesiones dentro del mismo segundo colisionan; antes que pisar el log
    // de la anterior, le agregamos un sufijo.
    let (log_path, mut log) = (1..)
        .find_map(|i| {
            let sufijo = if i == 1 { String::new() } else { format!("_{i}") };
            let p = dir.join(format!("session_{nombre}_{sello}{sufijo}.txt"));
            File::create_new(&p).ok().map(|f| (p, f))
        })
        .context("no pude crear el archivo de sesión")?;

    let par = native_pty_system().openpty(tam_actual())?;

    let mut builder = CommandBuilder::new(&cmd[0]);
    builder.args(&cmd[1..]);
    builder.cwd(std::env::current_dir()?);
    let mut hijo = par
        .slave
        .spawn_command(builder)
        .with_context(|| format!("no pude ejecutar `{}`", cmd[0]))?;

    // Sin esto el lector del master nunca ve EOF: el padre seguiría teniendo
    // abierto un extremo esclavo del PTY.
    drop(par.slave);

    let mut lector = par.master.try_clone_reader()?;
    let mut escritor = par.master.take_writer()?;

    let _raw = ModoRaw::activar()?;
    let vivo = Arc::new(AtomicBool::new(true));

    // ponytail: sondeo del tamaño cada 200ms en vez de manejar SIGWINCH.
    // Nadie percibe el retardo al redimensionar; si molesta, signal-hook + SIGWINCH.
    {
        let vivo = vivo.clone();
        let master = par.master;
        thread::spawn(move || {
            let mut ultimo = tam_actual();
            while vivo.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(200));
                let ahora = tam_actual();
                if (ahora.rows, ahora.cols) != (ultimo.rows, ultimo.cols) {
                    let _ = master.resize(ahora);
                    ultimo = ahora;
                }
            }
        });
    }

    // stdin -> pty. No se puede joinear: read() sobre stdin bloquea para siempre.
    thread::spawn(move || {
        let mut entrada = std::io::stdin();
        let mut buf = [0u8; 1024];
        while let Ok(n) = entrada.read(&mut buf) {
            if n == 0 || escritor.write_all(&buf[..n]).is_err() {
                break;
            }
            let _ = escritor.flush();
        }
    });

    // pty -> stdout + log
    let bombeo = thread::spawn(move || {
        let mut salida = std::io::stdout();
        let mut buf = [0u8; 8192];
        while let Ok(n) = lector.read(&mut buf) {
            if n == 0 {
                break;
            }
            let _ = log.write_all(&buf[..n]);
            let _ = salida.write_all(&buf[..n]);
            let _ = salida.flush();
        }
        let _ = log.flush();
    });

    let estado = hijo.wait()?;
    let _ = bombeo.join();
    vivo.store(false, Ordering::Relaxed);
    drop(_raw);

    eprintln!("\r\n[hernei] sesión guardada en {}", log_path.display());
    Ok(estado.exit_code() as i32)
}

/// Menú de sesiones -> limpia ANSI -> copia el prompt -> arranca el LLM (grabando).
fn retomar(cmd: &[String]) -> Result<i32> {
    let dir = dir_hernei()?;

    let mut sesiones: Vec<PathBuf> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("session_") && n.ends_with(".txt"))
        })
        .collect();

    if sesiones.is_empty() {
        bail!("no hay sesiones grabadas en {}", dir.display());
    }
    // El timestamp está en el nombre, así que ordenar por nombre alcanza.
    sesiones.sort();
    sesiones.reverse();

    let etiquetas: Vec<String> = sesiones
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();

    let elegida = Select::new("¿Qué sesión retomás?", etiquetas.clone()).prompt()?;
    let origen = &sesiones[etiquetas.iter().position(|e| *e == elegida).unwrap()];

    let crudo = fs::read(origen)?;
    let limpio = strip_ansi_escapes::strip(&crudo);
    let destino = dir.join("contexto_para_llm.txt");
    fs::write(&destino, &limpio)?;

    let prompt = format!(
        "Por favor, leé el historial de nuestra sesión anterior en este archivo: {}",
        destino.display()
    );
    copiar(prompt.clone());

    println!("[hernei] contexto limpio en {}", destino.display());
    println!("[hernei] prompt copiado al portapapeles, pegalo y seguimos:\n  {prompt}\n");

    grabar(cmd)
}

/// En X11/Wayland el portapapeles muere con el proceso dueño, así que este hilo
/// se queda bloqueado sirviéndolo mientras dure la sesión del LLM.
fn copiar(texto: String) {
    thread::spawn(move || {
        let Ok(mut cb) = Clipboard::new() else {
            eprintln!("[hernei] no pude acceder al portapapeles; copiá el prompt a mano");
            return;
        };
        #[cfg(target_os = "linux")]
        {
            use arboard::SetExtLinux;
            let _ = cb.set().wait().text(texto);
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = cb.set_text(texto);
        }
    });
}
