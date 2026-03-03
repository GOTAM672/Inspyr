project('inspyr', 'rust',
          version: '0.1.0',
    meson_version: '>= 1.0.0',
  default_options: [ 'warning_level=2', 'werror=false', ],
)

i18n = import('i18n')
gnome = import('gnome')

# --- Rust workspace build (no src/ dir) ---
pkgdatadir = get_option('prefix') / get_option('datadir') / meson.project_name()

conf = configuration_data()
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('GETTEXT_PACKAGE', 'inspyr')
conf.set_quoted('LOCALEDIR', get_option('prefix') / get_option('localedir'))
conf.set_quoted('PKGDATADIR', pkgdatadir)

configure_file(
  input: 'crates/inspyr-ui/config.rs.in',
  output: 'inspyr-ui-config.rs',
  configuration: conf,
)

run_command(
  'cp',
  meson.project_build_root() / 'inspyr-ui-config.rs',
  meson.project_source_root() / 'crates' / 'inspyr-ui' / 'src' / 'config.rs',
  check: true,
)

cargo_bin = find_program('cargo')
cargo_opt = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_opt += [ '--target-dir', meson.project_build_root() / 'rust' ]
cargo_env = [ 'CARGO_HOME=' + meson.project_build_root() / 'cargo-home' ]

if get_option('buildtype') == 'release'
  cargo_opt += [ '--release' ]
  rust_target = 'release'
else
  rust_target = 'debug'
endif

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    'env', cargo_env,
    cargo_bin, 'build',
    cargo_opt, '&&', 'cp', 'rust' / rust_target / meson.project_name(), '@OUTPUT@',
  ],
)

database_daemon_build = custom_target(
  'cargo-build-database-daemon',
  build_by_default: true,
  build_always_stale: true,
  output: 'inspyr-daemon',
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [ 'cp', 'rust' / rust_target / 'inspyr-daemon', '@OUTPUT@' ],
  depends: cargo_build,
)

subdir('data')
subdir('po')

gnome.post_install(
     glib_compile_schemas: true,
    gtk_update_icon_cache: true,
  update_desktop_database: true,
)
