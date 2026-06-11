# Changelog

All notable changes to Hermes Remote Manager will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Dioxus UI framework migration (100% Rust)
- Native terminal emulation with termwiz
- Signal-based state management
- Modular UI architecture

### Changed
- Replaced React + TypeScript frontend with Dioxus
- Removed Node.js/Vite build dependencies
- Updated Tauri configuration for Dioxus

### Removed
- React frontend (src/)
- TypeScript configuration
- Vite build system
- npm dependencies

## [0.1.0] - 2024-01-01

### Added
- Initial release
- SSH connection management
- Terminal emulation
- SFTP file browser
- RDP connection support
- Credential vault with encryption
- Connection folders and organization
- Dark/Light theme support
