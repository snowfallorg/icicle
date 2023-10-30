# Slides
yaml_build = custom_target(
  'slides.yml',
  input: 'slides.yml.in',
  output: 'slides.yml',
  console: true,
  install: true,
  install_dir: pkgconfdir / 'icicle' / 'branding' / 'snowflakeos',
  command: [
    ymlconvert,
    '@INPUT@',
    '@OUTPUT@'
  ]
)

install_data(
  ['apps.svg', 'configuration.svg', 'nix.svg', 'snowflakeos.svg'],
  install_dir: pkgconfdir / 'icicle' / 'branding' / 'snowflakeos'
)
