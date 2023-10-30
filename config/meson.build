# Basic
yaml_build = custom_target(
  'basic.yml',
  input: 'basic.yml.in',
  output: 'basic.yml',
  console: true,
  install: true,
  install_dir: pkgconfdir / 'icicle',
  command: [
    ymlconvert,
    '@INPUT@',
    '@OUTPUT@'
  ]
)

# Advanced
yaml_build = custom_target(
  'advanced.yml',
  input: 'advanced.yml.in',
  output: 'advanced.yml',
  console: true,
  install: true,
  install_dir: pkgconfdir / 'icicle',
  command: [
    ymlconvert,
    '@INPUT@',
    '@OUTPUT@'
  ]
)

# Flatpak
yaml_build = custom_target(
  'flatpak.yml',
  input: 'flatpak.yml.in',
  output: 'flatpak.yml',
  console: true,
  install: true,
  install_dir: pkgconfdir / 'icicle',
  command: [
    ymlconvert,
    '@INPUT@',
    '@OUTPUT@'
  ]
)

install_data('config.yml', install_dir : pkgconfdir / 'icicle')

install_subdir('basic', install_dir : pkgconfdir / 'icicle')
install_subdir('advanced', install_dir : pkgconfdir / 'icicle')
install_subdir('flatpak', install_dir : pkgconfdir / 'icicle')

subdir('branding/snowflakeos')
