# THEMING.md — Design System and Theming Specification

**Version:** 8.0  
**Status:** Active — all modules must conform  
**Last updated:** 2026-05

---

## 1. Design Language

Yfitops OS uses a design language called **Sovereign Dark** — a cyberpunk-adjacent, glass-panel aesthetic that communicates power, privacy, and precision. It is characterised by:

- Near-black backgrounds with subtle depth layering
- A single vivid accent colour (Sovereign Green `#00ff88`)
- Glassmorphism — transparent surfaces with backdrop blur
- Monospace typography throughout
- Minimal chrome — no decorative gradients, no shadows on text
- State communicated through colour temperature: green = sovereign/safe, amber = caution/amnesic, red = error/blocked

---

## 2. CSS Token System

All visual properties are expressed as CSS custom properties defined in `src/shared/theme.css`. No module defines its own colour values. Every module imports this file first.

### 2.1 Colour Tokens

```css
:root {
  /* Backgrounds */
  --yos-bg:           #0a0a0a;    /* Page / window background */
  --yos-surface:      #111111;    /* Card / panel surface */
  --yos-surface-2:    #1a1a1a;    /* Elevated surface (dropdown, modal) */
  --yos-surface-3:    #222222;    /* Highest elevation (tooltip) */

  /* Borders */
  --yos-border:       rgba(255, 255, 255, 0.07);
  --yos-border-focus: rgba(0, 255, 136, 0.4);

  /* Text */
  --yos-text:         #e8e8e8;    /* Primary text */
  --yos-text-muted:   #666666;    /* Secondary / placeholder */
  --yos-text-dim:     #333333;    /* Disabled / very secondary */

  /* Accent — Sovereign Green */
  --yos-accent:       #00ff88;
  --yos-accent-dim:   rgba(0, 255, 136, 0.15);
  --yos-accent-glow:  0 0 12px rgba(0, 255, 136, 0.3);

  /* Semantic colours */
  --yos-danger:       #ff4444;
  --yos-danger-dim:   rgba(255, 68, 68, 0.12);
  --yos-warn:         #ffaa00;
  --yos-warn-dim:     rgba(255, 170, 0, 0.12);
  --yos-success:      #00ff88;    /* Same as accent */
  --yos-info:         #4499ff;

  /* Amnesic Mode overlay (applied via JS on mode change) */
  --yos-amnesic-accent: #ffaa00;
}
```

### 2.2 Typography Tokens

```css
:root {
  /* Font stack — monospace first, system fallbacks */
  --yos-font-mono: 'JetBrains Mono', 'Cascadia Code', 'Fira Code',
                   'SF Mono', 'Consolas', monospace;
  --yos-font-ui:   'Inter', 'Segoe UI', system-ui, -apple-system, sans-serif;

  /* Type scale */
  --yos-text-xs:   11px;
  --yos-text-sm:   12px;
  --yos-text-base: 13px;
  --yos-text-md:   14px;
  --yos-text-lg:   16px;
  --yos-text-xl:   20px;

  /* Weight */
  --yos-weight-normal:  400;
  --yos-weight-medium:  500;
  --yos-weight-semibold:600;
}
```

### 2.3 Spacing Tokens

```css
:root {
  --yos-space-1:  4px;
  --yos-space-2:  8px;
  --yos-space-3:  12px;
  --yos-space-4:  16px;
  --yos-space-5:  20px;
  --yos-space-6:  24px;
  --yos-space-8:  32px;
  --yos-space-10: 40px;
}
```

### 2.4 Geometry Tokens

```css
:root {
  --yos-radius-sm:    4px;
  --yos-radius-md:    8px;
  --yos-radius-lg:    12px;
  --yos-radius-xl:    16px;
  --yos-radius-pill:  9999px;

  --yos-blur-sm:      blur(8px);
  --yos-blur-md:      blur(16px);
  --yos-blur-lg:      blur(24px);
}
```

### 2.5 Transition Tokens

```css
:root {
  --yos-transition-fast:   0.12s ease;
  --yos-transition:        0.18s ease;
  --yos-transition-slow:   0.30s ease;
  --yos-transition-spring: 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}
```

---

## 3. Component Patterns

These are the standard HTML+CSS patterns shared across all modules. Copy them exactly. Do not invent alternatives.

### 3.1 Title Bar (macOS-style)

```html
<div class="yos-titlebar" data-tauri-drag-region>
  <div class="yos-traffic-lights" data-tauri-drag-region="false">
    <button class="yos-tl yos-tl--close"   aria-label="Close"></button>
    <button class="yos-tl yos-tl--min"     aria-label="Minimize"></button>
    <button class="yos-tl yos-tl--expand"  aria-label="Maximize"></button>
  </div>
  <span class="yos-titlebar-title">Module Name</span>
  <div class="yos-titlebar-actions">
    <!-- optional action buttons -->
  </div>
</div>
```

```css
.yos-titlebar {
  display: flex;
  align-items: center;
  height: 38px;
  padding: 0 var(--yos-space-3);
  background: rgba(10, 10, 10, 0.75);
  backdrop-filter: var(--yos-blur-md);
  -webkit-backdrop-filter: var(--yos-blur-md);
  border-bottom: 1px solid var(--yos-border);
  border-radius: var(--yos-radius-lg) var(--yos-radius-lg) 0 0;
  flex-shrink: 0;
  user-select: none;
}

.yos-traffic-lights {
  display: flex;
  gap: 7px;
  align-items: center;
}

.yos-tl {
  width: 12px;
  height: 12px;
  border-radius: var(--yos-radius-pill);
  border: none;
  cursor: pointer;
  position: relative;
  transition: filter var(--yos-transition-fast);
}
.yos-tl:hover { filter: brightness(1.2); }
.yos-tl--close  { background: #ff5f57; }
.yos-tl--min    { background: #febc2e; }
.yos-tl--expand { background: #28c840; }

/* Hide symbols until hover on title bar */
.yos-titlebar:not(:hover) .yos-tl::after { opacity: 0; }
.yos-tl::after {
  content: attr(data-symbol);
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 8px;
  font-weight: 700;
  color: rgba(0,0,0,0.6);
  transition: opacity var(--yos-transition-fast);
}

.yos-titlebar-title {
  flex: 1;
  text-align: center;
  font-size: var(--yos-text-sm);
  font-weight: var(--yos-weight-medium);
  color: var(--yos-text-muted);
  letter-spacing: 0.03em;
}
```

### 3.2 Sidebar Layout

Used by: File Browser, Notes, Settings, Package Manager.

```css
.yos-layout-sidebar {
  display: flex;
  height: 100%;
  overflow: hidden;
}

.yos-sidebar {
  width: 200px;
  flex-shrink: 0;
  background: var(--yos-surface);
  border-right: 1px solid var(--yos-border);
  overflow-y: auto;
  padding: var(--yos-space-2) 0;
}

.yos-sidebar-item {
  display: flex;
  align-items: center;
  gap: var(--yos-space-2);
  padding: var(--yos-space-2) var(--yos-space-3);
  font-size: var(--yos-text-sm);
  color: var(--yos-text-muted);
  cursor: pointer;
  border-radius: var(--yos-radius-sm);
  margin: 1px var(--yos-space-2);
  transition: all var(--yos-transition-fast);
}
.yos-sidebar-item:hover      { background: var(--yos-surface-2); color: var(--yos-text); }
.yos-sidebar-item.active     { background: var(--yos-accent-dim); color: var(--yos-accent); }

.yos-main-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--yos-space-4);
}
```

### 3.3 Button Variants

```css
.yos-btn {
  display: inline-flex;
  align-items: center;
  gap: var(--yos-space-1);
  padding: var(--yos-space-2) var(--yos-space-3);
  font-size: var(--yos-text-sm);
  font-family: var(--yos-font-mono);
  border-radius: var(--yos-radius-md);
  border: 1px solid var(--yos-border);
  cursor: pointer;
  transition: all var(--yos-transition-fast);
  background: var(--yos-surface-2);
  color: var(--yos-text);
}
.yos-btn:hover { border-color: var(--yos-border-focus); color: var(--yos-accent); }

.yos-btn--primary {
  background: var(--yos-accent);
  color: #000;
  border-color: transparent;
}
.yos-btn--primary:hover { filter: brightness(1.1); box-shadow: var(--yos-accent-glow); }

.yos-btn--danger {
  background: var(--yos-danger-dim);
  color: var(--yos-danger);
  border-color: transparent;
}
.yos-btn--danger:hover { background: var(--yos-danger); color: #fff; }
```

### 3.4 Toggle (iOS-style)

```css
.yos-toggle-wrap {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--yos-space-3) var(--yos-space-4);
}

.yos-toggle {
  position: relative;
  width: 40px;
  height: 22px;
  flex-shrink: 0;
}
.yos-toggle input { opacity: 0; width: 0; height: 0; }

.yos-toggle-track {
  position: absolute;
  inset: 0;
  background: var(--yos-surface-3);
  border-radius: var(--yos-radius-pill);
  transition: background var(--yos-transition);
  cursor: pointer;
}
.yos-toggle-track::after {
  content: '';
  position: absolute;
  left: 3px;
  top: 3px;
  width: 16px;
  height: 16px;
  border-radius: var(--yos-radius-pill);
  background: var(--yos-text-muted);
  transition: all var(--yos-transition-spring);
}
.yos-toggle input:checked + .yos-toggle-track {
  background: var(--yos-accent-dim);
}
.yos-toggle input:checked + .yos-toggle-track::after {
  left: calc(100% - 19px);
  background: var(--yos-accent);
  box-shadow: var(--yos-accent-glow);
}
```

### 3.5 Skeleton Loader

```css
@keyframes yos-skeleton-wave {
  0%   { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

.yos-skeleton {
  background: linear-gradient(
    90deg,
    var(--yos-surface) 25%,
    var(--yos-surface-2) 50%,
    var(--yos-surface) 75%
  );
  background-size: 200% 100%;
  animation: yos-skeleton-wave 1.5s infinite;
  border-radius: var(--yos-radius-sm);
}
```

---

## 4. Wallpaper-Aware Theming (Phase 4)

When the user sets a wallpaper, the Rust `set_wallpaper()` command extracts a colour palette using the `image` crate and derives theme tokens from it. These override the base tokens via CSS custom property injection.

```typescript
// Frontend: listen for theme updates
await listen('theme:updated', (event: { payload: ThemePalette }) => {
    const root = document.documentElement;
    root.style.setProperty('--yos-bg',       event.payload.background);
    root.style.setProperty('--yos-surface',  event.payload.surface);
    root.style.setProperty('--yos-accent',   event.payload.accent);
});
```

The accent colour is clamped to minimum contrast ratios (WCAG AA against `--yos-bg`) before being applied. If the extracted accent does not meet contrast requirements, it is replaced with the default `#00ff88`.

---

## 5. Amnesic Mode Theming

When the session switches to Amnesic Mode, all module title bars transition to an amber tint:

```javascript
// Applied by session_manager listener in every module
document.documentElement.style.setProperty(
    '--yos-accent', 'var(--yos-amnesic-accent)'
);
document.body.classList.add('yos-amnesic');
```

```css
/* Applied globally when body.yos-amnesic is present */
body.yos-amnesic .yos-titlebar {
  border-bottom-color: rgba(255, 170, 0, 0.2);
}
body.yos-amnesic .yos-tl--close  { opacity: 0.5; }
body.yos-amnesic .yos-tl--expand { opacity: 0.5; }
```

---

## 6. Accessibility Baseline

All components must meet these minimum standards:

| Requirement | Standard |
|-------------|----------|
| Colour contrast (text on bg) | WCAG AA — 4.5:1 minimum |
| Focus indicator | Visible — `outline: 2px solid var(--yos-border-focus)` |
| Interactive target size | Minimum 24×24px |
| Font size minimum | 11px (`var(--yos-text-xs)`) |
| Keyboard navigation | All interactive elements reachable via Tab |
| `aria-label` | All icon-only buttons must have one |
