# Changelog

## [0.9.2](https://github.com/guilhermeprokisch/see/compare/v0.9.1...v0.9.2) (2026-07-08)


### Bug Fixes

* preserve whitespace around inline bold/italic spans ([#82](https://github.com/guilhermeprokisch/see/issues/82)) ([#83](https://github.com/guilhermeprokisch/see/issues/83)) ([9fa7fe3](https://github.com/guilhermeprokisch/see/commit/9fa7fe3318f2c426c9a567fb08c9d689f436993e))
* render soft and hard line breaks within paragraphs ([#84](https://github.com/guilhermeprokisch/see/issues/84)) ([#85](https://github.com/guilhermeprokisch/see/issues/85)) ([31728f2](https://github.com/guilhermeprokisch/see/commit/31728f27b059b3d9394ba08e9bf8f6129ba466dd))

## [0.9.1](https://github.com/guilhermeprokisch/see/compare/v0.9.0...v0.9.1) (2026-03-15)

### Bug Fixes

* fix recursive page-mode startup when `page = true` is enabled in config
* add pager capture regression coverage for config-driven paging

## [0.9.0](https://github.com/guilhermeprokisch/see/compare/v0.8.1...v0.9.0) (2026-03-12)

### Features

* add built-in page mode for long rendered output
* add file watch mode with bottom-following live preview
* add library API and HTML rendering support
* add configurable syntax themes and file extension language mappings
* add Nix flake support for development and builds

### Bug Fixes

* fix shell script syntax highlighting
* fix URL rendering when links touch surrounding text

## [0.8.1](https://github.com/guilhermeprokisch/see/compare/v0.8.0...v0.8.1) (2024-09-14)


### Bug Fixes

* admonition render ([#59](https://github.com/guilhermeprokisch/see/issues/59)) ([4e76547](https://github.com/guilhermeprokisch/see/commit/4e7654771c5c18a9fac899903a274ae807aefe1c))

## [0.8.0](https://github.com/guilhermeprokisch/see/compare/v0.7.1...v0.8.0) (2024-09-14)


### Features

* concatenate output and support base64 image encoding  ([#57](https://github.com/guilhermeprokisch/see/issues/57)) ([19c77e7](https://github.com/guilhermeprokisch/see/commit/19c77e74ac2ad1ace42ae2b33f2fa86db6fd0cba))

## [0.7.1](https://github.com/guilhermeprokisch/see/compare/v0.7.0...v0.7.1) (2024-09-13)


### Bug Fixes

* ansi pipe markdown ([6dcf235](https://github.com/guilhermeprokisch/see/commit/6dcf235644d8f37bbfc6c5fcc1aa76383e40a96c))
* ansi pipe markdown ([#40](https://github.com/guilhermeprokisch/see/issues/40)) ([b52d0a8](https://github.com/guilhermeprokisch/see/commit/b52d0a8705e528bbfa1eb76708bae0266eaac34f))
* remove ansi codes from piped markdown ([#42](https://github.com/guilhermeprokisch/see/issues/42)) ([bfcf9de](https://github.com/guilhermeprokisch/see/commit/bfcf9de4abce3960df3463e1e4868d9048d03f27))

## [0.7.0](https://github.com/guilhermeprokisch/see/compare/v0.6.0...v0.7.0) (2024-09-13)


### Features

* add markdown blocks rendering ([#23](https://github.com/guilhermeprokisch/see/issues/23)) ([4e29b90](https://github.com/guilhermeprokisch/see/commit/4e29b901aa7d3844d7c67cadf958e35139b3b78f))

## [0.6.0](https://github.com/guilhermeprokisch/see/compare/v0.5.3...v0.6.0) (2024-09-13)


### Features

* add multi visualization ([#20](https://github.com/guilhermeprokisch/see/issues/20)) ([e34ecb8](https://github.com/guilhermeprokisch/see/commit/e34ecb8ce576f55eb79cb53bc091d37e811fb259))
