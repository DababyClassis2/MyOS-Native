# MyOS v8.0 - Native Vision

## Project Overview
MyOS v8.0 (Native Vision) is a performance-oriented evolution of the Yfitops OS. It transitions from a purely browser-based environment to a native desktop experience using **Rust** and **Tauri**.

### Key Objectives
*   **Performance:** Leveraging Rust for system-level tasks (File I/O, Process Management).
*   **Native Integration:** Utilizing Tauri to provide native windowing, system tray integration, and OS-level APIs.
*   **Modern UI:** A high-fidelity, glassmorphism-based interface built with HTML/CSS/JS.

## Repository Structure
*   `MyOS-Native/`: The core native application.
    *   `src/`: Frontend UI assets (HTML, CSS, JS).
    *   `src-tauri/`: Rust backend, configuration, and build scripts.
*   `MyOS/`: (Legacy/Documentation) Originally the browser-based implementation, now serves as the project hub.

## Tech Stack
*   **Backend:** Rust (Tauri Framework)
*   **Frontend:** Vanilla HTML5, CSS3 (Glassmorphism), JavaScript (ES6+)
*   **Communication:** Tauri IPC (Invoke/Events)
*   **System Monitoring:** `sysinfo` (Rust crate)

## Getting Started
To run the project in development mode:
```bash
cd MyOS-Native
npm install
npm run tauri dev
```
