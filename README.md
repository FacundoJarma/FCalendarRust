# FCalendar (Rust)

Calendario personal de escritorio, nativo y local. Es la migración a Rust de
[facujarma/fcalendar](https://github.com/facujarma/fcalendar): conserva el modelo
de eventos, categorías y shortcuts de la versión React, pero funciona sin
navegador, sin Firebase y sin conexión a internet.

## Ejecutar

```bash
cargo run --release
```

Requiere Rust 1.88 o superior. En Linux necesita un entorno gráfico con X11 o
Wayland.

## Datos

Todo se guarda en un único JSON local con guardado automático y copias diarias.

- Linux: `~/.local/share/fcalendar/data.json`
- Copias: `~/.local/share/fcalendar/backups/`
- Para usar otra carpeta: variable `FCALENDAR_DATA_DIR`

La ruta exacta aparece en **Ajustes → Datos**.

## Funciones

- Vista mensual con navegación por teclado y doble clic para crear eventos.
- Eventos con título, fecha, categorías y propiedades personalizadas
  (texto, número, casilla, enlace y color).
- Categorías con color, visibilidad y enfoque.
- Shortcuts que aplican un estilo a todos los eventos de una categoría, con
  prioridad configurable.
- Búsqueda por nombre, categoría o fecha.
- Papelera con deshacer, copias de seguridad y exportación/importación JSON.
- Importación de los datos exportados desde la versión React/Firebase.

## Atajos

| Tecla | Acción |
| --- | --- |
| `←` / `→` | Cambiar de mes |
| `T` | Ir a hoy |
| `N` | Nuevo evento |
| `Ctrl+F` o `/` | Buscar |
| `Esc` | Cerrar detalle o búsqueda |

## Desarrollo

```bash
cargo test
cargo check
```
