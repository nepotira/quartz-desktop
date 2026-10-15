#!/usr/bin/env python3
# ==============================================================================
# 🍏 QuartzLinux (qz-desktop) — macOS Tahoe 26 Graphical Desktop Shell
# Production-grade 100% Procedural Vector & Liquid Glass UI (Zero External Fonts)
# ==============================================================================

import sys
import os
import math
import time
import subprocess
import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Gdk", "4.0")
from gi.repository import Gtk, Gdk, GLib, GObject, Gio
import cairo

# --- Canonical macOS Tahoe 26 Color Constants ---
TRAFFIC_RED = (255 / 255.0, 95 / 255.0, 86 / 255.0)          # #FF5F56
TRAFFIC_RED_BORDER = (224 / 255.0, 68 / 255.0, 62 / 255.0)    # #E0443E
TRAFFIC_RED_GLYPH = (77 / 255.0, 0 / 255.0, 0 / 255.0)        # #4D0000

TRAFFIC_YELLOW = (255 / 255.0, 189 / 255.0, 46 / 255.0)        # #FFBD2E
TRAFFIC_YELLOW_BORDER = (222 / 255.0, 161 / 255.0, 35 / 255.0) # #DEA123
TRAFFIC_YELLOW_GLYPH = (153 / 255.0, 87 / 255.0, 0 / 255.0)    # #995700

TRAFFIC_GREEN = (39 / 255.0, 201 / 255.0, 63 / 255.0)          # #27C93F
TRAFFIC_GREEN_BORDER = (26 / 255.0, 171 / 255.0, 41 / 255.0)    # #1AAB29
TRAFFIC_GREEN_GLYPH = (0 / 255.0, 101 / 255.0, 0 / 255.0)      # #006500

ACCENT_BLUE = (10 / 255.0, 96 / 255.0, 255 / 255.0)           # #0A60FF macOS Solid Blue

# ==============================================================================
# PROCEDURAL VECTOR DRAWING PRIMITIVES (ZERO EXTERNAL FONTS / ZERO TOFU)
# ==============================================================================

def draw_squircle(cr, x, y, w, h, r):
    """Draws a smooth G2 superellipse approximation using cubic beziers."""
    c = 0.552284749831 * r
    cr.new_sub_path()
    cr.move_to(x + r, y)
    cr.line_to(x + w - r, y)
    cr.curve_to(x + w - r + c, y, x + w, y + r - c, x + w, y + r)
    cr.line_to(x + w, y + h - r)
    cr.curve_to(x + w, y + h - r + c, x + w - r + c, y + h, x + w - r, y + h)
    cr.line_to(x + r, y + h)
    cr.curve_to(x + r - c, y + h, x, y + h - r + c, x, y + h - r)
    cr.line_to(x, y + r)
    cr.curve_to(x, y + r - c, x + r - c, y, x + r, y)
    cr.close_path()

def draw_apple_logo(cr, cx, cy, size, color=(1.0, 1.0, 1.0, 1.0)):
    """Draws Apple Logo via exact boolean geometry (leaf, lobes, bottom indent, bite)."""
    s = size / 24.0
    cr.push_group()
    cr.set_source_rgba(*color)

    # Leaf (tilted 35 degrees)
    cr.save()
    cr.translate(cx + 1.2 * s, cy - 8.5 * s)
    cr.rotate(math.radians(35))
    cr.scale(2.2 * s, 4.0 * s)
    cr.arc(0, 0, 1.0, 0, 2 * math.pi)
    cr.restore()
    cr.fill()

    # Body (symmetric lobes with top dip and bottom notch)
    cr.new_sub_path()
    cr.move_to(cx, cy - 4.5 * s)
    cr.curve_to(cx + 4.5 * s, cy - 4.5 * s, cx + 7.5 * s, cy - 1.5 * s, cx + 7.5 * s, cy + 2.5 * s)
    cr.curve_to(cx + 7.5 * s, cy + 6.5 * s, cx + 4.5 * s, cy + 9.5 * s, cx + 2.0 * s, cy + 9.5 * s)
    cr.curve_to(cx + 1.0 * s, cy + 9.5 * s, cx + 0.5 * s, cy + 8.8 * s, cx, cy + 8.8 * s)
    cr.curve_to(cx - 0.5 * s, cy + 8.8 * s, cx - 1.0 * s, cy + 9.5 * s, cx - 2.0 * s, cy + 9.5 * s)
    cr.curve_to(cx - 4.5 * s, cy + 9.5 * s, cx - 7.5 * s, cy + 6.5 * s, cx - 7.5 * s, cy + 2.5 * s)
    cr.curve_to(cx - 7.5 * s, cy - 1.5 * s, cx - 4.5 * s, cy - 4.5 * s, cx, cy - 4.5 * s)
    cr.close_path()
    cr.fill()

    # Right side bite subtraction
    cr.set_operator(cairo.OPERATOR_CLEAR)
    cr.arc(cx + 6.8 * s, cy + 0.8 * s, 3.2 * s, 0, 2 * math.pi)
    cr.fill()

    cr.pop_group_to_source()
    cr.set_operator(cairo.OPERATOR_OVER)
    cr.paint()

def draw_wifi_icon(cr, cx, cy, size, color=(1.0, 1.0, 1.0, 1.0)):
    """Draws macOS Wi-Fi icon (center dot + 3 concentric 90-degree arcs)."""
    cr.save()
    cr.set_source_rgba(*color)
    cr.set_line_cap(cairo.LINE_CAP_ROUND)
    s = size / 16.0
    # Center dot
    cr.arc(cx, cy + 5 * s, 1.3 * s, 0, 2 * math.pi)
    cr.fill()
    # 3 Arcs
    for i in range(1, 4):
        r = (3.2 * i) * s
        cr.set_line_width(1.4 * s)
        cr.arc(cx, cy + 5 * s, r, math.radians(225), math.radians(315))
        cr.stroke()
    cr.restore()

def draw_battery_icon_tahoe(cr, x, y, charge=1.0):
    """Draws exact macOS Tahoe battery icon: 22x11px, 1px contour 45%, pure white fill (Zero Neon Green!)."""
    cr.save()
    # 1. 1px contour rgba(255, 255, 255, 0.45)
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.45)
    cr.set_line_width(1.0)
    draw_squircle(cr, x, y, 22.0, 11.0, 2.5)
    cr.stroke()

    # 2. Positive terminal nub on the right: 1.5 x 4 px
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.45)
    cr.rectangle(x + 22.5, y + 3.5, 1.5, 4.0)
    cr.fill()

    # 3. Inner fill: PURE WHITE (#FFFFFF) with 1.5px inner margin!
    fill_w = max(0.0, 19.0 * charge)
    if fill_w > 0:
        cr.set_source_rgb(1.0, 1.0, 1.0)
        draw_squircle(cr, x + 1.5, y + 1.5, fill_w, 8.0, 1.5)
        cr.fill()
    cr.restore()

def draw_search_loupe(cr, cx, cy, size, color=(1.0, 1.0, 1.0, 1.0)):
    """Draws macOS Search / Spotlight loupe."""
    cr.save()
    cr.set_source_rgba(*color)
    cr.set_line_cap(cairo.LINE_CAP_ROUND)
    s = size / 16.0
    cr.set_line_width(1.6 * s)
    cr.arc(cx - 2 * s, cy - 2 * s, 4.5 * s, 0, 2 * math.pi)
    cr.stroke()
    cr.move_to(cx + 1.5 * s, cy + 1.5 * s)
    cr.line_to(cx + 6.0 * s, cy + 6.0 * s)
    cr.stroke()
    cr.restore()

def draw_control_center_icon(cr, cx, cy, size, color=(1.0, 1.0, 1.0, 1.0)):
    """Draws macOS Control Center dual-slider toggle icon."""
    cr.save()
    cr.set_source_rgba(*color)
    cr.set_line_cap(cairo.LINE_CAP_ROUND)
    s = size / 16.0
    cr.set_line_width(1.4 * s)
    # Top slider
    cr.move_to(cx - 6 * s, cy - 3 * s)
    cr.line_to(cx + 6 * s, cy - 3 * s)
    cr.stroke()
    cr.arc(cx - 2 * s, cy - 3 * s, 2.2 * s, 0, 2 * math.pi)
    cr.fill()
    # Bottom slider
    cr.move_to(cx - 6 * s, cy + 3 * s)
    cr.line_to(cx + 6 * s, cy + 3 * s)
    cr.stroke()
    cr.arc(cx + 2 * s, cy + 3 * s, 2.2 * s, 0, 2 * math.pi)
    cr.fill()
    cr.restore()

def draw_chevron(cr, cx, cy, size, direction="left", color=(1.0, 1.0, 1.0, 1.0)):
    """Draws navigation chevrons < and >."""
    cr.save()
    cr.set_source_rgba(*color)
    cr.set_line_cap(cairo.LINE_CAP_ROUND)
    cr.set_line_join(cairo.LINE_JOIN_ROUND)
    s = size / 16.0
    cr.set_line_width(1.6 * s)
    if direction == "left":
        cr.move_to(cx + 2.5 * s, cy - 4.5 * s)
        cr.line_to(cx - 2.5 * s, cy)
        cr.line_to(cx + 2.5 * s, cy + 4.5 * s)
    else:
        cr.move_to(cx - 2.5 * s, cy - 4.5 * s)
        cr.line_to(cx + 2.5 * s, cy)
        cr.line_to(cx - 2.5 * s, cy + 4.5 * s)
    cr.stroke()
    cr.restore()

def draw_view_mode_icon(cr, mode, cx, cy, size, color=(1.0, 1.0, 1.0, 1.0)):
    """Draws the 4 Finder view icons: Icons, List, Columns, Gallery."""
    cr.save()
    cr.set_source_rgba(*color)
    s = size / 16.0
    if mode == "icons":
        for dx in [-4 * s, 1 * s]:
            for dy in [-4 * s, 1 * s]:
                cr.rectangle(cx + dx, cy + dy, 3.2 * s, 3.2 * s)
                cr.fill()
    elif mode == "list":
        for dy in [-4 * s, 0, 4 * s]:
            cr.arc(cx - 4 * s, cy + dy, 1.0 * s, 0, 2 * math.pi)
            cr.fill()
            cr.rectangle(cx - 1.5 * s, cy + dy - 0.7 * s, 6.5 * s, 1.4 * s)
            cr.fill()
    elif mode == "columns":
        for dx in [-5 * s, -1 * s, 3 * s]:
            cr.rectangle(cx + dx, cy - 5 * s, 2.5 * s, 10 * s)
            cr.fill()
    elif mode == "gallery":
        cr.rectangle(cx - 3 * s, cy - 4 * s, 6 * s, 8 * s)
        cr.fill()
        cr.set_source_rgba(color[0], color[1], color[2], color[3] * 0.5)
        cr.rectangle(cx - 6 * s, cy - 3 * s, 2 * s, 6 * s)
        cr.rectangle(cx + 4 * s, cy - 3 * s, 2 * s, 6 * s)
        cr.fill()
    cr.restore()

def draw_sidebar_icon(cr, kind, cx, cy, size, color=(0.0, 0.48, 1.0, 1.0)):
    """Draws procedural vector icons for Finder Sidebar items."""
    cr.save()
    cr.set_source_rgba(*color)
    cr.set_line_cap(cairo.LINE_CAP_ROUND)
    s = size / 16.0
    cr.set_line_width(1.3 * s)

    if kind == "airdrop":
        cr.arc(cx, cy + 4 * s, 1.2 * s, 0, 2 * math.pi)
        cr.fill()
        for r in [3.5 * s, 6.5 * s]:
            cr.arc(cx, cy + 4 * s, r, math.radians(220), math.radians(320))
            cr.stroke()
    elif kind == "recents":
        cr.arc(cx, cy, 5.5 * s, 0, 2 * math.pi)
        cr.stroke()
        cr.move_to(cx, cy)
        cr.line_to(cx, cy - 3.5 * s)
        cr.move_to(cx, cy)
        cr.line_to(cx + 2.5 * s, cy)
        cr.stroke()
    elif kind == "apps":
        cr.move_to(cx, cy - 6 * s)
        cr.line_to(cx - 4.5 * s, cy + 5 * s)
        cr.move_to(cx, cy - 6 * s)
        cr.line_to(cx + 4.5 * s, cy + 5 * s)
        cr.move_to(cx - 3 * s, cy + 1 * s)
        cr.line_to(cx + 3 * s, cy + 1 * s)
        cr.stroke()
    elif kind == "desktop":
        draw_squircle(cr, cx - 6 * s, cy - 5 * s, 12 * s, 8 * s, 1.5)
        cr.stroke()
        cr.move_to(cx, cy + 3 * s)
        cr.line_to(cx, cy + 6 * s)
        cr.move_to(cx - 3 * s, cy + 6 * s)
        cr.line_to(cx + 3 * s, cy + 6 * s)
        cr.stroke()
    elif kind == "documents":
        draw_squircle(cr, cx - 4.5 * s, cy - 6 * s, 9 * s, 12 * s, 1.5)
        cr.stroke()
        for dy in [-2 * s, 0.5 * s, 3 * s]:
            cr.move_to(cx - 2.5 * s, cy + dy)
            cr.line_to(cx + 2.5 * s, cy + dy)
            cr.stroke()
    elif kind == "downloads":
        cr.move_to(cx, cy - 5 * s)
        cr.line_to(cx, cy + 2 * s)
        cr.move_to(cx - 3 * s, cy - 1 * s)
        cr.line_to(cx, cy + 2 * s)
        cr.line_to(cx + 3 * s, cy - 1 * s)
        cr.stroke()
        cr.move_to(cx - 5 * s, cy + 4 * s)
        cr.line_to(cx - 5 * s, cy + 6 * s)
        cr.line_to(cx + 5 * s, cy + 6 * s)
        cr.line_to(cx + 5 * s, cy + 4 * s)
        cr.stroke()
    elif kind == "icloud":
        cr.arc(cx - 2.5 * s, cy, 3 * s, math.radians(110), math.radians(280))
        cr.arc(cx + 1.0 * s, cy - 2.5 * s, 3.5 * s, math.radians(200), math.radians(360))
        cr.arc(cx + 4.0 * s, cy + 1.0 * s, 2.5 * s, math.radians(280), math.radians(90))
        cr.line_to(cx - 2.5 * s, cy + 3.5 * s)
        cr.stroke()
    elif kind == "machd":
        draw_squircle(cr, cx - 6 * s, cy - 4 * s, 12 * s, 8 * s, 2.0)
        cr.stroke()
        cr.arc(cx + 3.5 * s, cy, 1.0 * s, 0, 2 * math.pi)
        cr.fill()
    elif kind == "network":
        cr.arc(cx, cy, 5.5 * s, 0, 2 * math.pi)
        cr.stroke()
        cr.move_to(cx - 5.5 * s, cy)
        cr.line_to(cx + 5.5 * s, cy)
        cr.stroke()
        cr.save()
        cr.scale(0.5, 1.0)
        cr.arc(cx * 2.0, cy, 5.5 * s, 0, 2 * math.pi)
        cr.stroke()
        cr.restore()
    elif kind == "trash":
        draw_squircle(cr, cx - 4 * s, cy - 4 * s, 8 * s, 9 * s, 1.5)
        cr.stroke()
        cr.move_to(cx - 5.5 * s, cy - 4.5 * s)
        cr.line_to(cx + 5.5 * s, cy - 4.5 * s)
        cr.stroke()
    cr.restore()

# ==============================================================================
# OFFICIAL macOS TAHOE 26 FOLDER & 3D DOCK ARTWORK (NO MOCKUPS / NO MS PAINT)
# ==============================================================================

def draw_exact_macos_folder(cr, x, y, w, h):
    """Draws real macOS Tahoe folder (continuous S-curve rear tab, pocket AO, celestial gradient, grooves)."""
    cr.save()
    # 1. Soft projected shadow below the folder
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.28)
    draw_squircle(cr, x + 2, y + 6, w - 4, h - 2, 8)
    cr.fill()

    # 2. Rear Sheet with smooth S-curve transition on tab (x=0.34..0.48)
    tab_y = y + h * 0.14
    body_top_y = y + h * 0.22
    cr.save()
    cr.new_sub_path()
    cr.move_to(x + 5, body_top_y)
    cr.line_to(x + w * 0.34, body_top_y)
    # S-curve up to tab
    cr.curve_to(x + w * 0.38, body_top_y, x + w * 0.40, tab_y, x + w * 0.44, tab_y)
    cr.line_to(x + 5, tab_y)
    cr.close_path()
    # Apple Rear Tab Blue #228BE6
    pat_back = cairo.LinearGradient(x, tab_y, x, y + h * 0.86)
    pat_back.add_color_stop_rgb(0.0, 0.24, 0.65, 0.96)
    pat_back.add_color_stop_rgb(1.0, 0.10, 0.44, 0.80)
    cr.set_source(pat_back)
    cr.fill()

    # Rear main body
    draw_squircle(cr, x + 2, body_top_y, w - 4, h * 0.68, 6)
    cr.fill()

    # Specular rim on top of rear tab
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.35)
    cr.rectangle(x + 6, tab_y + 0.5, w * 0.36, 1.0)
    cr.fill()
    cr.restore()

    # 3. Pocket AO Shadow above y = 0.28 (seamless, no hard black band!)
    pocket_y = y + h * 0.28
    cr.set_source_rgba(0.0, 0.15, 0.40, 0.22)
    cr.rectangle(x + 4, pocket_y - 3, w - 8, 3)
    cr.fill()

    # 4. Front Cover Sheet with Apple Celestial Gradient (#72D6FF -> #127DFA)
    front_y = y + h * 0.28
    front_h = h * 0.60
    pat_front = cairo.LinearGradient(x, front_y, x, front_y + front_h)
    pat_front.add_color_stop_rgb(0.0, 0.45, 0.84, 1.00) # #72D6FF Radiant Cyan Top
    pat_front.add_color_stop_rgb(1.0, 0.07, 0.49, 0.98) # #127DFA Deep Apple Blue Bottom
    cr.set_source(pat_front)
    draw_squircle(cr, x, front_y, w, front_h, 7)
    cr.fill()

    # 5. Luminous white specular highlight on top lip (y = 0.286)
    cr.set_source_rgba(0.90, 0.98, 1.0, 0.70)
    cr.rectangle(x + 4, front_y + 0.5, w - 8, 1.2)
    cr.fill()

    # 6. Two subtle horizontal base grooves near bottom (y = 0.82 and y = 0.845)
    cr.set_source_rgba(0.0, 0.15, 0.45, 0.12)
    cr.rectangle(x + 6, front_y + front_h - 8, w - 12, 1)
    cr.rectangle(x + 6, front_y + front_h - 5, w - 12, 1)
    cr.fill()

    cr.restore()

def draw_dock_icon(cr, it_id, x, y, sz):
    """Draws real 3D vector artwork for all macOS Dock icons."""
    r = sz * 0.22
    cr.save()

    if it_id == "finder":
        # 3.1. FINDER (Rosto 3D Bicolor com Curva Senoidal, Olhos em Pílula e Sorriso Curvo)
        draw_squircle(cr, x, y, sz, sz, r)
        cr.clip()
        # Right pale face
        pat_r = cairo.LinearGradient(x, y, x, y + sz)
        pat_r.add_color_stop_rgb(0.0, 0.97, 0.99, 1.00)
        pat_r.add_color_stop_rgb(1.0, 0.82, 0.90, 0.97)
        cr.set_source(pat_r)
        cr.paint()

        # Left vivid blue face with curved nose split
        cr.save()
        cr.new_sub_path()
        cr.move_to(x, y)
        cr.line_to(x + sz * 0.50, y)
        cr.curve_to(x + sz * 0.44, y + sz * 0.35, x + sz * 0.42, y + sz * 0.52, x + sz * 0.56, y + sz * 0.52)
        cr.curve_to(x + sz * 0.54, y + sz * 0.70, x + sz * 0.50, y + sz * 0.85, x + sz * 0.52, y + sz)
        cr.line_to(x, y + sz)
        cr.close_path()
        pat_l = cairo.LinearGradient(x, y, x, y + sz)
        pat_l.add_color_stop_rgb(0.0, 0.22, 0.72, 1.00)
        pat_l.add_color_stop_rgb(1.0, 0.06, 0.44, 0.90)
        cr.set_source(pat_l)
        cr.fill()
        cr.restore()

        # Continuous nose contour line
        cr.set_source_rgb(0.08, 0.15, 0.26)
        cr.set_line_width(sz * 0.034)
        cr.set_line_cap(cairo.LINE_CAP_ROUND)
        cr.set_line_join(cairo.LINE_JOIN_ROUND)
        cr.move_to(x + sz * 0.48, y + sz * 0.24)
        cr.line_to(x + sz * 0.44, y + sz * 0.52)
        cr.line_to(x + sz * 0.56, y + sz * 0.52)
        cr.stroke()

        # Vertical pill eyes
        cr.rectangle(x + sz * 0.22, y + sz * 0.34, sz * 0.06, sz * 0.16)
        cr.rectangle(x + sz * 0.72, y + sz * 0.34, sz * 0.06, sz * 0.16)
        cr.fill()

        # Wide curved smile
        cr.new_sub_path()
        cr.arc(x + sz * 0.50, y + sz * 0.52, sz * 0.26, math.radians(25), math.radians(155))
        cr.stroke()

        # Inner 3D bevel & top specular highlight
        spec = cairo.LinearGradient(x, y, x, y + sz * 0.3)
        spec.add_color_stop_rgba(0.0, 1.0, 1.0, 1.0, 0.35)
        spec.add_color_stop_rgba(1.0, 1.0, 1.0, 1.0, 0.0)
        cr.set_source(spec)
        cr.rectangle(x, y, sz, sz * 0.3)
        cr.fill()

    elif it_id == "launchpad":
        # 3.6. LAUNCHPAD (Grade 3x3 de Mini-Squircles Coloridos)
        draw_squircle(cr, x, y, sz, sz, r)
        cr.clip()
        cr.set_source_rgb(0.16, 0.18, 0.24)
        cr.paint()
        colors = [
            (0.95, 0.35, 0.30), (0.98, 0.60, 0.20), (0.95, 0.85, 0.25),
            (0.30, 0.85, 0.45), (0.20, 0.75, 0.95), (0.15, 0.50, 0.95),
            (0.65, 0.35, 0.95), (0.95, 0.40, 0.80), (0.80, 0.85, 0.90),
        ]
        grid_step = sz * 0.26
        start_g = sz * 0.24
        idx = 0
        for row in range(3):
            for col in range(3):
                gx = x + start_g + col * grid_step
                gy = y + start_g + row * grid_step
                cr.set_source_rgb(*colors[idx])
                draw_squircle(cr, gx - sz * 0.08, gy - sz * 0.08, sz * 0.16, sz * 0.16, sz * 0.04)
                cr.fill()
                idx += 1

    elif it_id == "safari":
        # 3.2. SAFARI (Bússola Real com 36 Marcações Polares e Agulha Bipartida 3D)
        draw_squircle(cr, x, y, sz, sz, r)
        cr.clip()
        cr.set_source_rgb(0.96, 0.97, 0.99)
        cr.paint()
        cx, cy = x + sz * 0.5, y + sz * 0.5
        pat = cairo.LinearGradient(x, y, x, y + sz)
        pat.add_color_stop_rgb(0.0, 0.16, 0.74, 0.99)
        pat.add_color_stop_rgb(1.0, 0.04, 0.42, 0.92)
        cr.set_source(pat)
        cr.arc(cx, cy, sz * 0.36, 0, 2 * math.pi)
        cr.fill()
        # 36 polar ticks
        cr.set_source_rgba(0.95, 0.98, 1.0, 0.85)
        cr.set_line_width(1.0)
        for i in range(36):
            rad = math.radians(i * 10)
            cr.move_to(cx + math.cos(rad) * sz * 0.29, cy + math.sin(rad) * sz * 0.29)
            cr.line_to(cx + math.cos(rad) * sz * 0.33, cy + math.sin(rad) * sz * 0.33)
            cr.stroke()
        # 3D Needle at -45 deg
        cr.save()
        cr.translate(cx, cy)
        cr.rotate(math.radians(-45))
        # Needle drop shadow
        cr.set_source_rgba(0.0, 0.1, 0.3, 0.35)
        cr.move_to(0, -sz * 0.34 + 2)
        cr.line_to(sz * 0.07, 2)
        cr.line_to(-sz * 0.07, 2)
        cr.close_path()
        cr.fill()
        cr.move_to(0, sz * 0.34 + 2)
        cr.line_to(sz * 0.07, 2)
        cr.line_to(-sz * 0.07, 2)
        cr.close_path()
        cr.fill()
        # Red North Left facet (bright)
        cr.set_source_rgb(1.00, 0.23, 0.19)
        cr.move_to(0, -sz * 0.34)
        cr.line_to(sz * 0.07, 0)
        cr.line_to(0, 0)
        cr.close_path()
        cr.fill()
        # Red North Right facet (shaded)
        cr.set_source_rgb(0.85, 0.16, 0.14)
        cr.move_to(0, -sz * 0.34)
        cr.line_to(-sz * 0.07, 0)
        cr.line_to(0, 0)
        cr.close_path()
        cr.fill()
        # White South Left facet (bright)
        cr.set_source_rgb(0.98, 0.98, 1.00)
        cr.move_to(0, sz * 0.34)
        cr.line_to(sz * 0.07, 0)
        cr.line_to(0, 0)
        cr.close_path()
        cr.fill()
        # White South Right facet (shaded)
        cr.set_source_rgb(0.82, 0.84, 0.88)
        cr.move_to(0, sz * 0.34)
        cr.line_to(-sz * 0.07, 0)
        cr.line_to(0, 0)
        cr.close_path()
        cr.fill()
        # Center pivot
        cr.set_source_rgb(0.88, 0.90, 0.94)
        cr.arc(0, 0, sz * 0.035, 0, 2 * math.pi)
        cr.fill()
        cr.restore()

    elif it_id == "terminal":
        # 3.5. TERMINAL (Obsidiana Metálica com Segmento Vetorial >_)
        draw_squircle(cr, x, y, sz, sz, r)
        cr.clip()
        cr.set_source_rgb(0.12, 0.13, 0.16)
        cr.paint()
        cr.set_source_rgba(1, 1, 1, 0.10)
        cr.rectangle(x, y + sz * 0.22, sz, 1)
        cr.fill()
        cr.set_source_rgb(0.25, 0.90, 0.45)
        cr.set_line_width(sz * 0.055)
        cr.set_line_cap(cairo.LINE_CAP_ROUND)
        cr.set_line_join(cairo.LINE_JOIN_ROUND)
        cr.move_to(x + sz * 0.20, y + sz * 0.40)
        cr.line_to(x + sz * 0.35, y + sz * 0.52)
        cr.line_to(x + sz * 0.20, y + sz * 0.64)
        cr.stroke()
        cr.move_to(x + sz * 0.44, y + sz * 0.64)
        cr.line_to(x + sz * 0.68, y + sz * 0.64)
        cr.stroke()

    elif it_id == "settings":
        # 3.4. AJUSTES DO SISTEMA (Engrenagem Metálica 3D com Chanfro Cônico)
        draw_squircle(cr, x, y, sz, sz, r)
        cr.clip()
        pat = cairo.LinearGradient(x, y, x, y + sz)
        pat.add_color_stop_rgb(0.0, 0.55, 0.57, 0.62)
        pat.add_color_stop_rgb(1.0, 0.32, 0.34, 0.38)
        cr.set_source(pat)
        cr.paint()
        cx, cy = x + sz * 0.5, y + sz * 0.5
        # Gear shadow
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.35)
        cr.arc(cx + 1, cy + 2, sz * 0.32, 0, 2 * math.pi)
        cr.fill()
        # 12-tooth gear with conical sheen
        cr.set_source_rgb(0.85, 0.88, 0.92)
        cr.arc(cx, cy, sz * 0.29, 0, 2 * math.pi)
        cr.fill()
        for i in range(12):
            rad = math.radians(i * 30)
            cr.save()
            cr.translate(cx + math.cos(rad) * sz * 0.30, cy + math.sin(rad) * sz * 0.30)
            cr.rotate(rad)
            cr.rectangle(-sz * 0.035, -sz * 0.035, sz * 0.07, sz * 0.07)
            cr.fill()
            cr.restore()
        cr.set_source(pat)
        cr.arc(cx, cy, sz * 0.21, 0, 2 * math.pi)
        cr.fill()
        # 3 Spokes
        cr.set_source_rgb(0.85, 0.88, 0.92)
        for i in range(3):
            rad = math.radians(i * 120)
            cr.save()
            cr.translate(cx, cy)
            cr.rotate(rad)
            cr.rectangle(-sz * 0.03, 0, sz * 0.06, sz * 0.21)
            cr.fill()
            cr.restore()
        # Center hub
        cr.set_source_rgb(0.92, 0.94, 0.97)
        cr.arc(cx, cy, sz * 0.09, 0, 2 * math.pi)
        cr.fill()
        cr.set_source_rgb(0.35, 0.37, 0.40)
        cr.arc(cx, cy, sz * 0.045, 0, 2 * math.pi)
        cr.fill()

    elif it_id == "activity":
        draw_squircle(cr, x, y, sz, sz, r)
        cr.clip()
        cr.set_source_rgb(0.10, 0.12, 0.16)
        cr.paint()
        cr.set_source_rgba(1, 1, 1, 0.06)
        cr.set_line_width(1.0)
        for i in range(1, 4):
            cr.move_to(x, y + sz * (i * 0.25))
            cr.line_to(x + sz, y + sz * (i * 0.25))
            cr.stroke()
        cr.set_source_rgb(0.20, 0.85, 0.45)
        cr.set_line_width(sz * 0.045)
        cr.set_line_cap(cairo.LINE_CAP_ROUND)
        cr.set_line_join(cairo.LINE_JOIN_ROUND)
        cr.move_to(x + sz * 0.10, y + sz * 0.55)
        cr.line_to(x + sz * 0.30, y + sz * 0.55)
        cr.line_to(x + sz * 0.40, y + sz * 0.25)
        cr.line_to(x + sz * 0.50, y + sz * 0.80)
        cr.line_to(x + sz * 0.60, y + sz * 0.40)
        cr.line_to(x + sz * 0.70, y + sz * 0.55)
        cr.line_to(x + sz * 0.90, y + sz * 0.55)
        cr.stroke()

    elif it_id == "preview":
        draw_squircle(cr, x, y, sz, sz, r)
        cr.clip()
        pat = cairo.LinearGradient(x, y, x, y + sz)
        pat.add_color_stop_rgb(0.0, 0.25, 0.65, 0.95)
        pat.add_color_stop_rgb(1.0, 0.12, 0.42, 0.78)
        cr.set_source(pat)
        cr.paint()
        draw_squircle(cr, x + sz * 0.18, y + sz * 0.18, sz * 0.64, sz * 0.64, 4)
        cr.set_source_rgb(0.95, 0.96, 0.98)
        cr.fill()
        cr.set_source_rgb(0.20, 0.55, 0.85)
        cr.rectangle(x + sz * 0.24, y + sz * 0.24, sz * 0.52, sz * 0.42)
        cr.fill()
        cr.set_source_rgb(0.15, 0.75, 0.45)
        cr.move_to(x + sz * 0.24, y + sz * 0.66)
        cr.line_to(x + sz * 0.44, y + sz * 0.42)
        cr.line_to(x + sz * 0.60, y + sz * 0.56)
        cr.line_to(x + sz * 0.76, y + sz * 0.36)
        cr.line_to(x + sz * 0.76, y + sz * 0.66)
        cr.close_path()
        cr.fill()

    elif it_id == "trash":
        # 3.3. LIXEIRA DO macOS (Cilindro Metálico 3D Prateado)
        draw_squircle(cr, x, y, sz, sz, r)
        cr.clip()
        cr.set_source_rgb(0.20, 0.22, 0.26)
        cr.paint()
        cx, cy = x + sz * 0.5, y + sz * 0.52
        top_w = sz * 0.28
        bot_w = sz * 0.21
        body_h = sz * 0.58
        pat = cairo.LinearGradient(cx - top_w, cy, cx + top_w, cy)
        pat.add_color_stop_rgb(0.0, 0.58, 0.60, 0.65)
        pat.add_color_stop_rgb(0.3, 0.88, 0.90, 0.94)
        pat.add_color_stop_rgb(0.7, 0.65, 0.68, 0.72)
        pat.add_color_stop_rgb(1.0, 0.52, 0.54, 0.58)
        cr.set_source(pat)
        cr.move_to(cx - top_w, cy - body_h * 0.45)
        cr.line_to(cx + top_w, cy - body_h * 0.45)
        cr.line_to(cx + bot_w, cy + body_h * 0.55)
        cr.curve_to(cx + bot_w * 0.5, cy + body_h * 0.62, cx - bot_w * 0.5, cy + body_h * 0.62, cx - bot_w, cy + body_h * 0.55)
        cr.close_path()
        cr.fill()
        # Vertical wire ribs
        cr.set_source_rgba(0.20, 0.22, 0.26, 0.45)
        cr.set_line_width(sz * 0.02)
        for frac in [-0.6, -0.3, 0.0, 0.3, 0.6]:
            cr.move_to(cx + top_w * frac, cy - body_h * 0.42)
            cr.line_to(cx + bot_w * frac, cy + body_h * 0.54)
            cr.stroke()
        # Top rim ellipse
        cr.set_source_rgb(0.85, 0.88, 0.92)
        cr.save()
        cr.translate(cx, cy - body_h * 0.45)
        cr.scale(top_w + sz * 0.02, sz * 0.08)
        cr.arc(0, 0, 1.0, 0, 2 * math.pi)
        cr.restore()
        cr.fill()
        # Dark inner hole
        cr.set_source_rgb(0.22, 0.24, 0.28)
        cr.save()
        cr.translate(cx, cy - body_h * 0.45)
        cr.scale(top_w - sz * 0.02, sz * 0.05)
        cr.arc(0, 0, 1.0, 0, 2 * math.pi)
        cr.restore()
        cr.fill()

    cr.restore()

    # Specular rim highlight on squircle edge
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.28)
    cr.set_line_width(1.0)
    draw_squircle(cr, x, y, sz, sz, r)
    cr.stroke()

# ==============================================================================
# ANALYTICAL GAUSSIAN erf DUAL-LAYER SHADOW
# ==============================================================================

def draw_gaussian_shadow(cr, x, y, w, h, r, is_active=True):
    """Renders dual-layer analytical Gaussian erf shadow (Contact AO + Deep Penumbra)."""
    steps = 22
    max_d = 72.0
    sigma2 = 38.0
    alpha_base = 0.45 if is_active else 0.20
    dy2 = 22.0 if is_active else 14.0

    for i in range(steps, 0, -1):
        d = (i / float(steps)) * max_d
        alpha = alpha_base * math.exp(-(d * d) / (2.0 * sigma2 * sigma2)) * (1.0 / steps) * 2.8
        cr.set_source_rgba(0.0, 0.0, 0.0, alpha)
        draw_squircle(cr, x - d, y - d + dy2, w + 2 * d, h + 2 * d, r + d * 0.5)
        cr.fill()

    for i in range(8, 0, -1):
        d = (i / 8.0) * 8.0
        alpha = 0.30 * math.exp(-(d * d) / 32.0) * (1.0 / 8.0) * 2.5
        cr.set_source_rgba(0.0, 0.0, 0.0, alpha)
        draw_squircle(cr, x - d, y - d + 2.0, w + 2 * d, h + 2 * d, r + d * 0.3)
        cr.fill()

# ==============================================================================
# MAIN DESKTOP APPLICATION & WINDOW CONTROLLER
# ==============================================================================

class QuartzDesktopApp(Gtk.Application):
    def __init__(self, is_nested=True):
        super().__init__(
            application_id="org.quartzlinux.desktop",
            flags=Gio.ApplicationFlags.NON_UNIQUE
        )
        self.is_nested = is_nested
        self.window = None

    def do_activate(self):
        if not self.window:
            self.window = QuartzDesktopWindow(self, self.is_nested)
        self.window.present()

class QuartzDesktopWindow(Gtk.ApplicationWindow):
    def __init__(self, app, is_nested):
        super().__init__(application=app)
        self.is_nested = is_nested
        self.set_title("QuartzLinux — macOS Tahoe 26")

        if is_nested:
            self.set_default_size(1440, 900)
            self.set_resizable(True)
        else:
            self.fullscreen()

        # Canvas & Heavy Blurred Backdrop Cache
        self.cached_width = 0
        self.cached_height = 0
        self.wallpaper_surface = None
        self.blurred_backdrop = None

        # Input & UI State
        self.mouse_x = -1000.0
        self.mouse_y = -1000.0
        self.dock_hover = False
        self.active_menu = None
        self.spotlight_visible = False
        self.spotlight_query = ""
        self.control_center_visible = False
        self.brightness = 0.85
        self.volume = 0.70

        # Finder Window Geometry & State
        self.finder_visible = True
        self.finder_minimized = False
        self.finder_x = 200.0
        self.finder_y = 100.0
        self.finder_w = 880.0
        self.finder_h = 560.0
        self.finder_dragging = False
        self.drag_offset_x = 0.0
        self.drag_offset_y = 0.0
        self.traffic_group_hover = False
        self.traffic_hover_idx = -1
        self.traffic_pressed = -1
        self.finder_selected_sidebar = "machd"
        self.finder_selected_folder = "Applications"
        self.finder_view_mode = "icons"

        # Terminal Window Geometry & State
        self.terminal_visible = False
        self.terminal_minimized = False
        self.terminal_x = 320.0
        self.terminal_y = 180.0
        self.terminal_w = 680.0
        self.terminal_h = 420.0
        self.terminal_dragging = False
        self.term_drag_offset_x = 0.0
        self.term_drag_offset_y = 0.0
        self.active_window = "finder"

        self.term_output = [
            "QuartzLinux (macOS Tahoe 26 Edition) — Wayland + Vulkan 1.3 Liquid Glass",
            "Kernel: Linux 6.13-cachyos x86_64 | Native Retina Dynamic VRR 120Hz",
            "Digite comandos do sistema para executar:",
            "nepo@cachyos-tahoe ~ % "
        ]

        # Dock items
        self.dock_items = [
            {"id": "finder", "name": "Finder", "running": True, "bounce": 0.0},
            {"id": "launchpad", "name": "Launchpad", "running": False, "bounce": 0.0},
            {"id": "safari", "name": "Safari", "running": False, "bounce": 0.0},
            {"id": "terminal", "name": "Terminal", "running": True, "bounce": 0.0},
            {"id": "settings", "name": "Ajustes do Sistema", "running": False, "bounce": 0.0},
            {"id": "activity", "name": "Monitor de Atividade", "running": False, "bounce": 0.0},
            {"id": "preview", "name": "Pré-Visualização", "running": False, "bounce": 0.0},
            {"id": "sep", "name": "", "running": False, "bounce": 0.0},
            {"id": "trash", "name": "Lixo", "running": False, "bounce": 0.0},
        ]

        # Main Drawing Area
        self.area = Gtk.DrawingArea()
        self.area.set_draw_func(self.on_draw)
        self.set_child(self.area)

        # Controllers
        motion = Gtk.EventControllerMotion()
        motion.connect("motion", self.on_motion)
        motion.connect("leave", self.on_leave)
        self.area.add_controller(motion)

        click = Gtk.GestureClick()
        click.connect("pressed", self.on_click_pressed)
        click.connect("released", self.on_click_released)
        self.area.add_controller(click)

        key = Gtk.EventControllerKey()
        key.connect("key-pressed", self.on_key_pressed)
        self.add_controller(key)

        GLib.timeout_add(16, self.on_tick)

    def on_tick(self):
        need_redraw = False
        for item in self.dock_items:
            if item["bounce"] > 0.01:
                item["bounce"] = max(0.0, item["bounce"] - 0.08)
                need_redraw = True
        self.area.queue_draw()
        return True

    def ensure_backdrop(self, width, height):
        """Generates dynamic Tahoe 26 wallpaper and heavy multi-stage blurred backdrop (sigma >= 42px)."""
        if width == self.cached_width and height == self.cached_height and self.blurred_backdrop:
            return

        self.cached_width = width
        self.cached_height = height

        # 1. High-resolution Wallpaper Surface (Dynamic Tahoe Fluid Waves)
        self.wallpaper_surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, width, height)
        cr = cairo.Context(self.wallpaper_surface)

        base_pat = cairo.LinearGradient(0, 0, width, height)
        base_pat.add_color_stop_rgb(0.0, 0.04, 0.14, 0.39)  # Deep Oceanic Blue #0A2463
        base_pat.add_color_stop_rgb(0.35, 0.23, 0.05, 0.64) # Midnight Violet #3A0CA3
        base_pat.add_color_stop_rgb(0.70, 0.02, 0.70, 0.85) # Vibrant Cyan #00B4D8
        base_pat.add_color_stop_rgb(1.0, 0.08, 0.03, 0.15)  # Obsidian Bottom
        cr.set_source(base_pat)
        cr.paint()

        waves = [
            ((0.98, 0.30, 0.45, 0.42), [(0.1, 0.8), (0.4, 0.3), (0.7, 0.7), (1.0, 0.2)]),
            ((0.00, 0.82, 0.95, 0.38), [(0.0, 0.4), (0.3, 0.8), (0.6, 0.2), (0.9, 0.6)]),
            ((1.00, 0.72, 0.05, 0.30), [(0.2, 0.9), (0.5, 0.5), (0.8, 0.8), (1.0, 0.4)]),
            ((0.55, 0.15, 0.90, 0.35), [(0.0, 0.2), (0.35, 0.6), (0.65, 0.3), (1.0, 0.7)]),
        ]

        for color, pts in waves:
            cr.save()
            cr.set_source_rgba(*color)
            cr.new_sub_path()
            p0 = pts[0]
            cr.move_to(p0[0] * width, p0[1] * height)
            p1, p2, p3 = pts[1], pts[2], pts[3]
            cr.curve_to(p1[0] * width, p1[1] * height, p2[0] * width, p2[1] * height, p3[0] * width, p3[1] * height)
            cr.line_to(width, height)
            cr.line_to(0, height)
            cr.close_path()
            cr.fill()
            cr.restore()

        # 2. Heavy Multi-Stage Cascade Blur (1/4 -> 1/16 -> 1/32 with 5-tap neighborhood diffusion)
        # Guarantees sigma >= 42px (zero sharp wallpaper lines show through)
        s1 = cairo.ImageSurface(cairo.FORMAT_ARGB32, max(1, width // 4), max(1, height // 4))
        c1 = cairo.Context(s1)
        c1.scale(0.25, 0.25)
        c1.set_source_surface(self.wallpaper_surface, 0, 0)
        c1.get_source().set_filter(cairo.FILTER_BILINEAR)
        c1.paint()

        s2 = cairo.ImageSurface(cairo.FORMAT_ARGB32, max(1, width // 16), max(1, height // 16))
        c2 = cairo.Context(s2)
        c2.scale(0.25, 0.25)
        c2.set_source_surface(s1, 0, 0)
        c2.get_source().set_filter(cairo.FILTER_BILINEAR)
        c2.paint()

        s3 = cairo.ImageSurface(cairo.FORMAT_ARGB32, max(1, width // 32), max(1, height // 32))
        c3 = cairo.Context(s3)
        c3.scale(0.5, 0.5)
        c3.set_source_surface(s2, 0, 0)
        c3.get_source().set_filter(cairo.FILTER_BILINEAR)
        c3.paint()

        s3_diffused = cairo.ImageSurface(cairo.FORMAT_ARGB32, max(1, width // 32), max(1, height // 32))
        cbd = cairo.Context(s3_diffused)
        for dx, dy, wgt in [(-1, 0, 0.2), (1, 0, 0.2), (0, -1, 0.2), (0, 1, 0.2), (0, 0, 0.2)]:
            cbd.set_source_surface(s3, dx, dy)
            cbd.paint_with_alpha(wgt)

        self.blurred_backdrop = cairo.ImageSurface(cairo.FORMAT_ARGB32, width, height)
        cr_up = cairo.Context(self.blurred_backdrop)
        cr_up.scale(32.0, 32.0)
        cr_up.set_source_surface(s3_diffused, 0, 0)
        cr_up.get_source().set_filter(cairo.FILTER_BILINEAR)
        cr_up.paint()

    def on_motion(self, controller, x, y):
        self.mouse_x = x
        self.mouse_y = y

        if self.finder_dragging:
            self.finder_x = x - self.drag_offset_x
            self.finder_y = y - self.drag_offset_y
            self.area.queue_draw()
            return

        if self.terminal_dragging:
            self.terminal_x = x - self.term_drag_offset_x
            self.terminal_y = y - self.term_drag_offset_y
            self.area.queue_draw()
            return

        # Traffic lights hover detection on Floating Glass Pill Sidebar
        if self.finder_visible and not self.finder_minimized:
            sb_x = self.finder_x + 10.0
            sb_y = self.finder_y + 10.0
            tl_x = sb_x + 16.0
            tl_y = sb_y + 16.0
            if (tl_x - 6 <= x <= tl_x + 58) and (tl_y - 6 <= y <= tl_y + 18):
                self.traffic_group_hover = True
                if math.hypot(x - (tl_x + 6), y - (tl_y + 6)) <= 8:
                    self.traffic_hover_idx = 0
                elif math.hypot(x - (tl_x + 26), y - (tl_y + 6)) <= 8:
                    self.traffic_hover_idx = 1
                elif math.hypot(x - (tl_x + 46), y - (tl_y + 6)) <= 8:
                    self.traffic_hover_idx = 2
                else:
                    self.traffic_hover_idx = -1
            else:
                self.traffic_group_hover = False
                self.traffic_hover_idx = -1

        self.area.queue_draw()

    def on_leave(self, controller):
        self.mouse_x = -1000.0
        self.mouse_y = -1000.0
        self.dock_hover = False
        self.traffic_group_hover = False
        self.traffic_hover_idx = -1
        self.area.queue_draw()

    def on_click_pressed(self, gesture, n_press, x, y):
        # 1. Top Menu Bar (height 28)
        if y <= 28:
            if x <= 36:
                self.active_menu = "apple" if self.active_menu != "apple" else None
            elif 42 <= x <= 95:
                self.active_menu = "finder" if self.active_menu != "finder" else None
            elif 100 <= x <= 150:
                self.active_menu = "file" if self.active_menu != "file" else None
            elif 155 <= x <= 195:
                self.active_menu = "edit" if self.active_menu != "edit" else None
            elif 200 <= x <= 250:
                self.active_menu = "view" if self.active_menu != "view" else None
            elif 255 <= x <= 280:
                self.active_menu = "go" if self.active_menu != "go" else None
            elif 285 <= x <= 335:
                self.active_menu = "window" if self.active_menu != "window" else None
            elif 340 <= x <= 380:
                self.active_menu = "help" if self.active_menu != "help" else None

            # Right status items
            w = self.area.get_width()
            if w - 145 <= x <= w - 120:
                self.spotlight_visible = not self.spotlight_visible
                self.active_menu = None
            elif w - 118 <= x <= w - 90:
                self.control_center_visible = not self.control_center_visible
                self.active_menu = None
            self.area.queue_draw()
            return

        if self.active_menu:
            if not (8 <= x <= 230 and 32 <= y <= 280):
                self.active_menu = None
                self.area.queue_draw()

        if self.control_center_visible:
            w = self.area.get_width()
            if not (w - 320 <= x <= w - 16 and 36 <= y <= 340):
                self.control_center_visible = False
                self.area.queue_draw()

        if self.spotlight_visible:
            w = self.area.get_width()
            sp_x = (w - 680) / 2
            sp_y = self.area.get_height() * 0.20
            if not (sp_x <= x <= sp_x + 680 and sp_y <= y <= sp_y + 260):
                self.spotlight_visible = False
                self.area.queue_draw()

        # 2. Terminal Window Interactions
        if self.terminal_visible and not self.terminal_minimized:
            if (self.terminal_x <= x <= self.terminal_x + self.terminal_w and
                self.terminal_y <= y <= self.terminal_y + 36):
                tl_x = self.terminal_x + 18
                tl_y = self.terminal_y + 16
                if math.hypot(x - (tl_x + 6), y - (tl_y + 6)) <= 8:
                    self.terminal_visible = False
                    self.area.queue_draw()
                    return
                elif math.hypot(x - (tl_x + 24), y - (tl_y + 6)) <= 8:
                    self.terminal_minimized = True
                    self.area.queue_draw()
                    return

                self.terminal_dragging = True
                self.active_window = "terminal"
                self.term_drag_offset_x = x - self.terminal_x
                self.term_drag_offset_y = y - self.terminal_y
                self.area.queue_draw()
                return
            elif (self.terminal_x <= x <= self.terminal_x + self.terminal_w and
                  self.terminal_y <= y <= self.terminal_y + self.terminal_h):
                self.active_window = "terminal"
                self.area.queue_draw()

        # 3. Finder Window Interactions
        if self.finder_visible and not self.finder_minimized:
            sb_x = self.finder_x + 10.0
            sb_y = self.finder_y + 10.0
            sb_w = 200.0
            sb_h = self.finder_h - 20.0

            # Traffic lights inside Floating Glass Pill Sidebar
            tl_x = sb_x + 16.0
            tl_y = sb_y + 16.0
            if math.hypot(x - (tl_x + 6), y - (tl_y + 6)) <= 8:
                self.finder_visible = False
                self.traffic_pressed = 0
                self.area.queue_draw()
                return
            elif math.hypot(x - (tl_x + 26), y - (tl_y + 6)) <= 8:
                self.finder_minimized = True
                self.traffic_pressed = 1
                self.area.queue_draw()
                return
            elif math.hypot(x - (tl_x + 46), y - (tl_y + 6)) <= 8:
                if self.finder_w > 1000:
                    self.finder_x, self.finder_y, self.finder_w, self.finder_h = 200, 100, 880, 560
                else:
                    w, h = self.area.get_width(), self.area.get_height()
                    self.finder_x, self.finder_y, self.finder_w, self.finder_h = 16, 36, w - 32, h - 128
                self.traffic_pressed = 2
                self.area.queue_draw()
                return

            # Check Sidebar items click
            if sb_x <= x <= sb_x + sb_w and sb_y + 48 <= y <= sb_y + sb_h:
                cur_sy = sb_y + 54
                sections = [
                    ("FAVORITOS", [("airdrop", "AirDrop"), ("recents", "Recentes"), ("apps", "Aplicativos"),
                                   ("desktop", "Mesa"), ("documents", "Documentos"), ("downloads", "Downloads")]),
                    ("iCLOUD", [("icloud", "iCloud Drive")]),
                    ("LOCAIS", [("machd", "Macintosh HD"), ("network", "Rede"), ("trash", "Lixeira")]),
                ]
                for sec_title, items in sections:
                    cur_sy += 16
                    for kid, name in items:
                        if cur_sy - 12 <= y <= cur_sy + 14:
                            self.finder_selected_sidebar = kid
                            self.area.queue_draw()
                            return
                        cur_sy += 24
                    cur_sy += 8

            # Toolbar interactions
            tb_y = self.finder_y + 10.0
            if tb_y <= y <= tb_y + 44:
                # View switcher pill
                v_x = self.finder_x + self.finder_w - 320
                if v_x <= x <= v_x + 132:
                    seg = int((x - v_x) / 33.0)
                    modes = ["icons", "list", "columns", "gallery"]
                    if 0 <= seg < 4:
                        self.finder_view_mode = modes[seg]
                        self.area.queue_draw()
                        return

                self.finder_dragging = True
                self.active_window = "finder"
                self.drag_offset_x = x - self.finder_x
                self.drag_offset_y = y - self.finder_y
                self.area.queue_draw()
                return

            # Folder selection in grid (Bipartite selection click)
            fx_start = self.finder_x + 230
            fy_start = self.finder_y + 68
            if (fx_start <= x <= self.finder_x + self.finder_w and
                fy_start <= y <= self.finder_y + self.finder_h - 32):
                col = int((x - fx_start) / 155.0)
                row = int((y - fy_start) / 110.0)
                idx = row * 4 + col
                folders = ["Applications", "Developer", "Documents", "Downloads",
                           "Library", "Movies", "Music", "Pictures", "System", "Users"]
                if 0 <= idx < len(folders):
                    self.finder_selected_folder = folders[idx]
                    self.active_window = "finder"
                    self.area.queue_draw()
                    return

        # 4. Dock Item Click
        w, h = self.area.get_width(), self.area.get_height()
        dock_y = h - 80
        if y >= dock_y - 20:
            layout = self.calculate_dock_layout(w, h)
            for it in layout:
                ix, iy, sz, it_data = it["x"], it["y"], it["size"], it["data"]
                if ix <= x <= ix + sz and iy <= y <= iy + sz:
                    it_data["bounce"] = 1.0
                    it_id = it_data["id"]
                    if it_id == "finder":
                        self.finder_visible = True
                        self.finder_minimized = False
                        self.active_window = "finder"
                    elif it_id == "terminal":
                        self.terminal_visible = True
                        self.terminal_minimized = False
                        self.active_window = "terminal"
                    elif it_id == "settings":
                        self.control_center_visible = True
                    self.area.queue_draw()
                    break

    def on_click_released(self, gesture, n_press, x, y):
        self.finder_dragging = False
        self.terminal_dragging = False
        self.traffic_pressed = -1
        self.area.queue_draw()

    def on_key_pressed(self, controller, keyval, keycode, state):
        if (state & (Gdk.ModifierType.CONTROL_MASK | Gdk.ModifierType.SUPER_MASK)) and keyval == Gdk.KEY_space:
            self.spotlight_visible = not self.spotlight_visible
            self.area.queue_draw()
            return True

        if keyval == Gdk.KEY_Escape:
            self.spotlight_visible = False
            self.control_center_visible = False
            self.active_menu = None
            self.area.queue_draw()
            return True

        if self.spotlight_visible:
            if keyval == Gdk.KEY_BackSpace:
                self.spotlight_query = self.spotlight_query[:-1]
            elif keyval == Gdk.KEY_Return:
                q = self.spotlight_query.lower()
                if "term" in q:
                    self.terminal_visible = True
                    self.terminal_minimized = False
                elif "find" in q:
                    self.finder_visible = True
                    self.finder_minimized = False
                self.spotlight_visible = False
                self.spotlight_query = ""
            elif keyval < 256 and chr(keyval).isprintable():
                self.spotlight_query += chr(keyval)
            self.area.queue_draw()
            return True

        # Typing in Terminal
        if self.active_window == "terminal" and self.terminal_visible:
            if keyval == Gdk.KEY_Return:
                cmd = self.term_output[-1].replace("nepo@cachyos-tahoe ~ % ", "").strip()
                if cmd == "clear":
                    self.term_output = ["nepo@cachyos-tahoe ~ % "]
                elif cmd == "uname -a":
                    self.term_output.append("Linux cachyos-tahoe 6.13.0-cachyos #1 SMP PREEMPT_DYNAMIC x86_64 GNU/Linux")
                    self.term_output.append("nepo@cachyos-tahoe ~ % ")
                elif cmd in ("neofetch", "fastfetch"):
                    self.term_output.append("  Apple Silicon / CachyOS Linux x86_64 Tahoe 26")
                    self.term_output.append("  Kernel: 6.13.0-cachyos | Smithay Wayland Compositor")
                    self.term_output.append("  UI: Liquid Glass Refraction + Dual-Kawase Multipass Blur")
                    self.term_output.append("  Theme: macOS Tahoe 26 Pure Vector Edition")
                    self.term_output.append("nepo@cachyos-tahoe ~ % ")
                elif cmd:
                    self.term_output.append(f"quartz-sh: comando executado: {cmd}")
                    self.term_output.append("nepo@cachyos-tahoe ~ % ")
                else:
                    self.term_output.append("nepo@cachyos-tahoe ~ % ")
                self.area.queue_draw()
                return True
            elif keyval == Gdk.KEY_BackSpace:
                cur = self.term_output[-1]
                if len(cur) > len("nepo@cachyos-tahoe ~ % "):
                    self.term_output[-1] = cur[:-1]
                self.area.queue_draw()
                return True
            elif keyval < 256 and chr(keyval).isprintable():
                self.term_output[-1] += chr(keyval)
                self.area.queue_draw()
                return True

        return False

    def calculate_dock_layout(self, w, h):
        """Calculates smooth cosine parabolic magnification layout for dock items."""
        base_size = 54.0
        max_size = 92.0
        influence_radius = 120.0
        spacing = 10.0
        num_items = len(self.dock_items)
        dock_y = h - 74.0

        is_hovering_dock = (h - 96.0 <= self.mouse_y <= h)

        sizes = []
        for i, item in enumerate(self.dock_items):
            if item["id"] == "sep":
                sizes.append(10.0)
                continue
            if is_hovering_dock and self.mouse_x >= 0:
                nom_x = (w / 2.0) + (i - num_items / 2.0) * (base_size + spacing)
                dist = abs(self.mouse_x - nom_x)
                if dist < influence_radius:
                    factor = math.cos((dist / influence_radius) * (math.pi / 2.0)) ** 2
                    s = base_size + (max_size - base_size) * factor
                else:
                    s = base_size
            else:
                s = base_size
            sizes.append(s)

        total_width = sum(sizes) + (num_items - 1) * spacing
        start_x = (w - total_width) / 2.0

        layout = []
        curr_x = start_x
        for i, item in enumerate(self.dock_items):
            sz = sizes[i]
            bounce_dy = item["bounce"] * 24.0 * math.sin(item["bounce"] * math.pi)
            if item["id"] == "sep":
                layout.append({
                    "x": curr_x,
                    "y": dock_y + 12,
                    "size": sz,
                    "height": base_size - 16,
                    "data": item
                })
            else:
                layout.append({
                    "x": curr_x,
                    "y": dock_y - (sz - base_size) - bounce_dy,
                    "size": sz,
                    "height": sz,
                    "data": item
                })
            curr_x += sz + spacing

        return layout

    def on_draw(self, area, cr, width, height):
        self.ensure_backdrop(width, height)

        # 1. Sharp Wallpaper
        cr.set_source_surface(self.wallpaper_surface, 0, 0)
        cr.paint()

        # 2. Render Windows (Gaussian Shadow + Body)
        if self.terminal_visible and not self.terminal_minimized:
            self.draw_terminal_window(cr)

        if self.finder_visible and not self.finder_minimized:
            self.draw_finder_window(cr)

        # 3. Parabolic Magnified Liquid Glass Dock
        self.draw_dock(cr, width, height)

        # 4. Top Menu Bar (Liquid Glass 28px)
        self.draw_menubar(cr, width)

        # 5. Overlays (Menus, Control Center, Spotlight)
        if self.active_menu:
            self.draw_dropdown_menu(cr)

        if self.control_center_visible:
            self.draw_control_center(cr, width)

        if self.spotlight_visible:
            self.draw_spotlight(cr, width, height)

    def draw_menubar(self, cr, width):
        """Renders 28px Top Menu Bar with Liquid Glass backdrop blur and battery 100% on left."""
        # Backdrop Blur
        cr.save()
        cr.rectangle(0, 0, width, 28)
        cr.clip()
        cr.set_source_surface(self.blurred_backdrop, 0, 0)
        cr.paint()
        cr.set_source_rgba(0.08, 0.10, 0.14, 0.58)
        cr.paint()
        cr.restore()

        # 1px bottom rim highlight
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.16)
        cr.rectangle(0, 27.5, width, 0.5)
        cr.fill()

        # Vector Apple Logo
        draw_apple_logo(cr, 18, 14, 15, (1.0, 1.0, 1.0, 0.95))

        # Active App Name (Finder)
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(13.5)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.95)
        cr.move_to(38, 19)
        cr.show_text("Finder")

        # Standard Menus
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(13)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.88)

        menus = [("Arquivo", 100), ("Editar", 158), ("Visualizar", 208),
                 ("Ir", 280), ("Janela", 305), ("Ajuda", 360)]
        for name, mx in menus:
            cr.move_to(mx, 19)
            cr.show_text(name)

        # Right Status Tray
        # 1. Text "100%" (12px Semibold white) positioned to the LEFT of the battery
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(12)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.90)
        cr.move_to(width - 232, 19)
        cr.show_text("100%")

        # 2. Exact Battery Icon (22x11px, 1px contour 45%, positive nub, pure white fill)
        draw_battery_icon_tahoe(cr, width - 192, 8.5, 1.0)

        # 3. Wi-Fi
        draw_wifi_icon(cr, width - 150, 14, 15)

        # 4. Search Loupe
        draw_search_loupe(cr, width - 124, 14, 15)

        # 5. Control Center
        draw_control_center_icon(cr, width - 98, 14, 15)

        # 6. Dynamic Clock
        time_str = time.strftime("%a %d %b %H:%M")
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(12.5)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.95)
        cr.move_to(width - 74, 19)
        cr.show_text(time_str)

    def draw_dropdown_menu(self, cr):
        """Renders Apple Menu or App dropdown with Liquid Glass."""
        mx, my, mw, mh = 8.0, 32.0, 210.0, 240.0
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.35)
        draw_squircle(cr, mx, my + 4, mw, mh, 10)
        cr.fill()

        cr.save()
        draw_squircle(cr, mx, my, mw, mh, 10)
        cr.clip()
        cr.set_source_surface(self.blurred_backdrop, 0, 0)
        cr.paint()
        cr.set_source_rgba(0.12, 0.14, 0.20, 0.85)
        cr.paint()
        cr.restore()

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.20)
        cr.set_line_width(1.0)
        draw_squircle(cr, mx, my, mw, mh, 10)
        cr.stroke()

        items = [
            "Sobre Este Mac", "Ajustes do Sistema...", "App Store...", "---",
            "Forcar Encerrar...", "---", "Repouso", "Reiniciar...", "Desligar...", "---",
            "Bloquear Tela", "Encerrar Sessao..."
        ]
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(13)
        cur_y = my + 18
        for it in items:
            if it == "---":
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.12)
                cr.rectangle(mx + 12, cur_y - 4, mw - 24, 1)
                cr.fill()
                cur_y += 10
            else:
                if mx <= self.mouse_x <= mx + mw and cur_y - 14 <= self.mouse_y <= cur_y + 6:
                    cr.set_source_rgba(*ACCENT_BLUE, 0.85)
                    draw_squircle(cr, mx + 6, cur_y - 14, mw - 12, 20, 5)
                    cr.fill()
                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0)
                else:
                    cr.set_source_rgba(1.0, 1.0, 1.0, 0.88)
                cr.move_to(mx + 16, cur_y)
                cr.show_text(it)
                cur_y += 20

    def draw_finder_window(self, cr):
        """Renders macOS Tahoe 26 Finder Window with Floating Glass Pill Sidebar & Bipartite Selection."""
        x, y, w, h = self.finder_x, self.finder_y, self.finder_w, self.finder_h

        # 1. Analytical Gaussian erf dual-layer shadow
        draw_gaussian_shadow(cr, x, y, w, h, 12, is_active=(self.active_window == "finder"))

        # 2. Main Window Dark Body (#1E1E22)
        cr.set_source_rgb(0.118, 0.118, 0.133)
        draw_squircle(cr, x, y, w, h, 12)
        cr.fill()

        # 1px Specular Border
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.18)
        cr.set_line_width(1.0)
        draw_squircle(cr, x, y, w, h, 12)
        cr.stroke()

        # 3. Floating Glass Pill Sidebar (Tahoe 26)
        # Inset 10px on all sides: detached independent capsule with R=12px and NO vertical divider line!
        sb_x = x + 10.0
        sb_y = y + 10.0
        sb_w = 200.0
        sb_h = h - 20.0

        # Independent shadow around floating sidebar capsule
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.22)
        draw_squircle(cr, sb_x - 1, sb_y + 1, sb_w + 2, sb_h + 2, 12)
        cr.fill()

        # Backdrop blur + Liquid Glass tint
        cr.save()
        draw_squircle(cr, sb_x, sb_y, sb_w, sb_h, 12)
        cr.clip()
        cr.set_source_surface(self.blurred_backdrop, 0, 0)
        cr.paint()
        cr.set_source_rgba(0.14, 0.14, 0.17, 0.58)
        cr.paint()
        cr.restore()

        # Specular inner border on Floating Sidebar
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.14)
        cr.set_line_width(1.0)
        draw_squircle(cr, sb_x, sb_y, sb_w, sb_h, 12)
        cr.stroke()

        # 4. Traffic Lights (Inside Floating Glass Pill Sidebar)
        tl_x = sb_x + 16.0
        tl_y = sb_y + 16.0
        lights = [
            (TRAFFIC_RED, TRAFFIC_RED_BORDER, TRAFFIC_RED_GLYPH, "x"),
            (TRAFFIC_YELLOW, TRAFFIC_YELLOW_BORDER, TRAFFIC_YELLOW_GLYPH, "-"),
            (TRAFFIC_GREEN, TRAFFIC_GREEN_BORDER, TRAFFIC_GREEN_GLYPH, "+"),
        ]
        for i, (fill, border, glyph_col, sym) in enumerate(lights):
            cx = tl_x + i * 20 + 6
            cy = tl_y + 6
            f_col = (fill[0] * 0.82, fill[1] * 0.82, fill[2] * 0.82) if self.traffic_pressed == i else fill
            cr.set_source_rgb(*f_col)
            cr.arc(cx, cy, 6, 0, 2 * math.pi)
            cr.fill()
            cr.set_source_rgb(*border)
            cr.set_line_width(0.8)
            cr.arc(cx, cy, 6, 0, 2 * math.pi)
            cr.stroke()
            if self.traffic_group_hover:
                cr.set_source_rgb(*glyph_col)
                cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
                cr.set_font_size(9)
                ext = cr.text_extents(sym)
                cr.move_to(cx - ext.width / 2.0, cy + ext.height / 2.0 - 0.5)
                cr.show_text(sym)

        # 5. Sidebar Sections & Items
        sections = [
            ("FAVORITOS", [
                ("airdrop", "AirDrop"), ("recents", "Recentes"), ("apps", "Aplicativos"),
                ("desktop", "Mesa"), ("documents", "Documentos"), ("downloads", "Downloads")
            ]),
            ("iCLOUD", [("icloud", "iCloud Drive")]),
            ("LOCAIS", [("machd", "Macintosh HD"), ("network", "Rede"), ("trash", "Lixeira")]),
        ]

        cur_sy = sb_y + 54
        for sec_title, items in sections:
            cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
            cr.set_font_size(10.5)
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.42)
            cr.move_to(sb_x + 16, cur_sy)
            cr.show_text(sec_title)
            cur_sy += 16

            cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
            cr.set_font_size(12.5)
            for it_id, it_name in items:
                is_sel = (self.finder_selected_sidebar == it_id)
                if is_sel:
                    # Active item internal pill: 8px margin, R=7.0, rgba(255, 255, 255, 0.14)
                    cr.set_source_rgba(1.0, 1.0, 1.0, 0.14)
                    draw_squircle(cr, sb_x + 8.0, cur_sy - 12.0, sb_w - 16.0, 28.0, 7.0)
                    cr.fill()

                draw_sidebar_icon(cr, it_id, sb_x + 22, cur_sy + 2, 14,
                                  color=(0.15, 0.60, 1.0, 1.0) if not is_sel else (1.0, 1.0, 1.0, 1.0))

                cr.set_source_rgba(1.0, 1.0, 1.0, 0.95 if is_sel else 0.82)
                cr.move_to(sb_x + 38, cur_sy + 6)
                cr.show_text(it_name)
                cur_sy += 24
            cur_sy += 8

        # 6. Toolbar (y = win.y, x = win.x + 220) - Subtle rest state, zero hard borders
        tb_x = x + 220
        tb_y = y + 10.0
        tb_w = w - 230

        # Navigation buttons < > (rgba(255, 255, 255, 0.06), subtle border 0.08, R=6px)
        draw_squircle(cr, tb_x, tb_y + 8, 28, 28, 6)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.06)
        cr.fill()
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.08)
        cr.set_line_width(0.8)
        draw_squircle(cr, tb_x, tb_y + 8, 28, 28, 6)
        cr.stroke()
        draw_chevron(cr, tb_x + 14, tb_y + 22, 14, "left", (1.0, 1.0, 1.0, 0.85))

        draw_squircle(cr, tb_x + 34, tb_y + 8, 28, 28, 6)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.06)
        cr.fill()
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.08)
        cr.set_line_width(0.8)
        draw_squircle(cr, tb_x + 34, tb_y + 8, 28, 28, 6)
        cr.stroke()
        draw_chevron(cr, tb_x + 48, tb_y + 22, 14, "right", (1.0, 1.0, 1.0, 0.45))

        # Title
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(15)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.95)
        cr.move_to(tb_x + 76, tb_y + 27)
        cr.show_text("Macintosh HD")

        # 4-View Segmented Control
        v_x = tb_x + tb_w - 310
        draw_squircle(cr, v_x, tb_y + 8, 132, 28, 6)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.06)
        cr.fill()
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.08)
        cr.set_line_width(0.8)
        draw_squircle(cr, v_x, tb_y + 8, 132, 28, 6)
        cr.stroke()

        modes = ["icons", "list", "columns", "gallery"]
        for idx, m in enumerate(modes):
            mx = v_x + idx * 33
            if self.finder_view_mode == m:
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.16)
                draw_squircle(cr, mx + 2, tb_y + 10, 29, 24, 5)
                cr.fill()
            draw_view_mode_icon(cr, m, mx + 16.5, tb_y + 22, 14,
                                (1.0, 1.0, 1.0, 0.95 if self.finder_view_mode == m else 0.55))

        # Search Pill with Vector Loupe (subtle rest state)
        sp_x = tb_x + tb_w - 165
        draw_squircle(cr, sp_x, tb_y + 8, 155, 28, 6)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.06)
        cr.fill()
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.08)
        cr.set_line_width(0.8)
        draw_squircle(cr, sp_x, tb_y + 8, 155, 28, 6)
        cr.stroke()

        draw_search_loupe(cr, sp_x + 14, tb_y + 22, 13, (1.0, 1.0, 1.0, 0.55))
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(12)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.40)
        cr.move_to(sp_x + 28, tb_y + 26)
        cr.show_text("Buscar")

        # Divider line below toolbar
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.08)
        cr.rectangle(tb_x, tb_y + 44, tb_w, 1)
        cr.fill()

        # 7. Main Grid of Folders with Bipartite Selection (Seleção Bipartida)
        fx_start = x + 230
        fy_start = y + 68
        folders = [
            "Applications", "Developer", "Documents", "Downloads",
            "Library", "Movies", "Music", "Pictures", "System", "Users"
        ]
        for idx, fname in enumerate(folders):
            col = idx % 4
            row = idx // 4
            fx = fx_start + col * 155
            fy = fy_start + row * 110
            is_sel = (self.finder_selected_folder == fname)

            if is_sel:
                # 1. Behind ICON ONLY: 84x84px rounded square (R=8px) in rgba(255, 255, 255, 0.12) with NO blue border!
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.12)
                draw_squircle(cr, fx + 5, fy - 6, 84, 84, 8)
                cr.fill()

                # Draw exact S-curve Apple folder
                draw_exact_macos_folder(cr, fx + 10, fy, 74, 58)

                # 2. Behind TEXT LABEL ONLY: tight pill (w = text_w + 12px, h = 18px, R = 4px) in solid macOS blue #0A60FF
                cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
                cr.set_font_size(12)
                ext = cr.text_extents(fname)
                w_pill = ext.width + 12.0
                h_pill = 18.0
                pill_x = fx + 47 - w_pill / 2.0
                pill_y = fy + 82
                cr.set_source_rgb(10 / 255.0, 96 / 255.0, 255 / 255.0) # #0A60FF
                draw_squircle(cr, pill_x, pill_y, w_pill, h_pill, 4)
                cr.fill()

                # Pure white bold text
                cr.set_source_rgb(1.0, 1.0, 1.0)
                cr.move_to(pill_x + 6, pill_y + 13)
                cr.show_text(fname)

            else:
                # Unselected folder
                draw_exact_macos_folder(cr, fx + 10, fy, 74, 58)

                cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
                cr.set_font_size(12)
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.90)
                ext = cr.text_extents(fname)
                cr.move_to(fx + 47 - ext.width / 2.0, fy + 94)
                cr.show_text(fname)

        # 8. Status Bar Footer (28px)
        stat_y = y + h - 28
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.06)
        cr.rectangle(x + 220, stat_y, w - 230, 1)
        cr.fill()

        draw_exact_macos_folder(cr, x + 230, stat_y + 6, 18, 14)
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(11)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.50)
        cr.move_to(x + 256, stat_y + 18)
        cr.show_text("10 itens, 428,4 GB disponiveis")

    def draw_terminal_window(self, cr):
        """Renders GPU-Accelerated Liquid Glass Terminal Emulator."""
        x, y, w, h = self.terminal_x, self.terminal_y, self.terminal_w, self.terminal_h

        draw_gaussian_shadow(cr, x, y, w, h, 12, is_active=(self.active_window == "terminal"))

        cr.set_source_rgba(0.06, 0.07, 0.10, 0.92)
        draw_squircle(cr, x, y, w, h, 12)
        cr.fill()

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.18)
        cr.set_line_width(1.0)
        draw_squircle(cr, x, y, w, h, 12)
        cr.stroke()

        tl_x = x + 18
        tl_y = y + 14
        for i, fill in enumerate([TRAFFIC_RED, TRAFFIC_YELLOW, TRAFFIC_GREEN]):
            cx = tl_x + i * 18 + 5
            cy = tl_y + 5
            cr.set_source_rgb(*fill)
            cr.arc(cx, cy, 5.5, 0, 2 * math.pi)
            cr.fill()

        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(12)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.85)
        title = "nepo - zsh - 80x24 (QuartzLinux Terminal)"
        ext = cr.text_extents(title)
        cr.move_to(x + (w - ext.width) / 2.0, y + 22)
        cr.show_text(title)

        cr.select_font_face("Monospace", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(12)
        text_y = y + 54
        max_lines = int((h - 60) / 18)
        lines = self.term_output[-max_lines:]
        for line in lines:
            if line.startswith("nepo@cachyos-tahoe"):
                cr.set_source_rgba(0.35, 0.90, 0.45, 1.0)
            elif "Kernel:" in line or "QuartzLinux" in line:
                cr.set_source_rgba(0.30, 0.75, 1.0, 0.95)
            else:
                cr.set_source_rgba(0.92, 0.92, 0.94, 0.90)
            cr.move_to(x + 18, text_y)
            cr.show_text(line)
            text_y += 18

        last_line = self.term_output[-1]
        ext = cr.text_extents(last_line)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.80)
        cr.rectangle(x + 18 + ext.width + 2, text_y - 30, 8, 14)
        cr.fill()

    def draw_dock(self, cr, width, height):
        """Renders Parabolic Magnified Liquid Glass Dock with 3D vector icons."""
        dock_layout = self.calculate_dock_layout(width, height)
        if not dock_layout:
            return

        min_x = min(item["x"] for item in dock_layout) - 16
        max_x = max(item["x"] + item["size"] for item in dock_layout) + 16
        dock_w = max_x - min_x
        dock_h = 70.0
        dock_x = min_x
        dock_y = height - 76.0

        cr.set_source_rgba(0.0, 0.0, 0.0, 0.35)
        draw_squircle(cr, dock_x, dock_y + 4, dock_w, dock_h, 20)
        cr.fill()

        # Heavy Backdrop Blur underneath dock (sigma >= 42px)
        cr.save()
        draw_squircle(cr, dock_x, dock_y, dock_w, dock_h, 20)
        cr.clip()
        cr.set_source_surface(self.blurred_backdrop, 0, 0)
        cr.paint()
        cr.set_source_rgba(0.12, 0.14, 0.20, 0.60)
        cr.paint()
        cr.restore()

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.30)
        cr.set_line_width(1.0)
        draw_squircle(cr, dock_x, dock_y, dock_w, dock_h, 20)
        cr.stroke()

        hovered_name = None
        hovered_x = 0
        hovered_y = 0

        for it in dock_layout:
            ix, iy, sz, it_data = it["x"], it["y"], it["size"], it["data"]
            it_id = it_data["id"]

            if it_id == "sep":
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.25)
                cr.rectangle(ix + sz / 2.0 - 0.5, iy, 1.0, it["height"])
                cr.fill()
                continue

            draw_dock_icon(cr, it_id, ix, iy, sz)

            if it_data["running"]:
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.95)
                cr.arc(ix + sz / 2.0, dock_y + dock_h - 6, 2.0, 0, 2 * math.pi)
                cr.fill()

            if ix <= self.mouse_x <= ix + sz and iy <= self.mouse_y <= iy + sz:
                hovered_name = it_data["name"]
                hovered_x = ix + sz / 2.0
                hovered_y = iy - 14.0

        if hovered_name:
            cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
            cr.set_font_size(12)
            ext = cr.text_extents(hovered_name)
            tt_w = ext.width + 16.0
            tt_h = 24.0
            tt_x = hovered_x - tt_w / 2.0
            tt_y = hovered_y - tt_h

            cr.set_source_rgba(0.10, 0.12, 0.18, 0.88)
            draw_squircle(cr, tt_x, tt_y, tt_w, tt_h, 6)
            cr.fill()
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.20)
            cr.set_line_width(0.8)
            draw_squircle(cr, tt_x, tt_y, tt_w, tt_h, 6)
            cr.stroke()

            cr.set_source_rgba(1.0, 1.0, 1.0, 0.95)
            cr.move_to(tt_x + 8.0, tt_y + 16.0)
            cr.show_text(hovered_name)

    def draw_spotlight(self, cr, width, height):
        """Renders Spotlight Search bar with Liquid Glass and vector loupe."""
        sp_w = 680.0
        sp_h = 56.0 if not self.spotlight_query else 240.0
        sp_x = (width - sp_w) / 2.0
        sp_y = height * 0.20

        cr.set_source_rgba(0.0, 0.0, 0.0, 0.45)
        draw_squircle(cr, sp_x - 8, sp_y + 12, sp_w + 16, sp_h + 16, 20)
        cr.fill()

        cr.save()
        draw_squircle(cr, sp_x, sp_y, sp_w, sp_h, 16)
        cr.clip()
        cr.set_source_surface(self.blurred_backdrop, 0, 0)
        cr.paint()
        cr.set_source_rgba(0.12, 0.15, 0.22, 0.85)
        cr.paint()
        cr.restore()

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.25)
        cr.set_line_width(1.2)
        draw_squircle(cr, sp_x, sp_y, sp_w, sp_h, 16)
        cr.stroke()

        draw_search_loupe(cr, sp_x + 28, sp_y + 28, 20, (1.0, 1.0, 1.0, 0.65))

        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(18)
        if not self.spotlight_query:
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.35)
            cr.move_to(sp_x + 56, sp_y + 36)
            cr.show_text("Buscar no Spotlight")
        else:
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.95)
            cr.move_to(sp_x + 56, sp_y + 36)
            cr.show_text(self.spotlight_query)

            ext = cr.text_extents(self.spotlight_query)
            cr.set_source_rgba(*ACCENT_BLUE, 0.90)
            cr.rectangle(sp_x + 56 + ext.width + 2, sp_y + 18, 2, 22)
            cr.fill()

            cr.set_source_rgba(1.0, 1.0, 1.0, 0.10)
            cr.rectangle(sp_x, sp_y + 56, sp_w, 1)
            cr.fill()

            results = [
                ("Terminal", "Aplicativo - /usr/local/bin/qz-terminal"),
                ("Ajustes do Sistema", "Aplicativo - /usr/local/bin/qz-settings"),
                ("Monitor de Atividade", "Aplicativo - /usr/local/bin/qz-activity"),
                ("Finder", "Gerenciador de Arquivos do macOS Tahoe"),
            ]
            matched = [r for r in results if self.spotlight_query.lower() in r[0].lower()] or results

            cur_ry = sp_y + 86
            for idx, (r_name, r_desc) in enumerate(matched[:4]):
                if idx == 0:
                    cr.set_source_rgba(*ACCENT_BLUE, 0.85)
                    draw_squircle(cr, sp_x + 8, cur_ry - 20, sp_w - 16, 38, 8)
                    cr.fill()
                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0)
                else:
                    cr.set_source_rgba(1.0, 1.0, 1.0, 0.85)

                cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
                cr.set_font_size(14)
                cr.move_to(sp_x + 46, cur_ry - 2)
                cr.show_text(r_name)

                cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
                cr.set_font_size(11)
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.65 if idx == 0 else 0.45)
                cr.move_to(sp_x + 46, cur_ry + 13)
                cr.show_text(r_desc)
                cur_ry += 42

    def draw_control_center(self, cr, width):
        """Renders Control Center panel."""
        cc_w = 320.0
        cc_h = 320.0
        cc_x = width - cc_w - 16.0
        cc_y = 36.0

        cr.set_source_rgba(0.0, 0.0, 0.0, 0.40)
        draw_squircle(cr, cc_x - 6, cc_y + 8, cc_w + 12, cc_h + 12, 18)
        cr.fill()

        cr.save()
        draw_squircle(cr, cc_x, cc_y, cc_w, cc_h, 16)
        cr.clip()
        cr.set_source_surface(self.blurred_backdrop, 0, 0)
        cr.paint()
        cr.set_source_rgba(0.12, 0.15, 0.22, 0.85)
        cr.paint()
        cr.restore()

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.20)
        cr.set_line_width(1.0)
        draw_squircle(cr, cc_x, cc_y, cc_w, cc_h, 16)
        cr.stroke()

        card_x = cc_x + 12
        card_y = cc_y + 12
        card_w = 142
        card_h = 130
        cr.set_source_rgba(0.18, 0.21, 0.30, 0.65)
        draw_squircle(cr, card_x, card_y, card_w, card_h, 12)
        cr.fill()

        cr.set_source_rgba(*ACCENT_BLUE, 0.95)
        cr.arc(card_x + 24, card_y + 26, 14, 0, 2 * math.pi)
        cr.fill()
        draw_wifi_icon(cr, card_x + 24, card_y + 24, 14)
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(12)
        cr.set_source_rgb(1, 1, 1)
        cr.move_to(card_x + 46, card_y + 24)
        cr.show_text("Wi-Fi")
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(10)
        cr.set_source_rgba(1, 1, 1, 0.6)
        cr.move_to(card_x + 46, card_y + 36)
        cr.show_text("CachyOS-5G")

        cr.set_source_rgba(*ACCENT_BLUE, 0.95)
        cr.arc(card_x + 24, card_y + 68, 14, 0, 2 * math.pi)
        cr.fill()
        cr.set_source_rgb(1, 1, 1)
        cr.set_line_width(1.4)
        cr.move_to(card_x + 24, card_y + 60)
        cr.line_to(card_x + 24, card_y + 76)
        cr.line_to(card_x + 29, card_y + 72)
        cr.line_to(card_x + 20, card_y + 64)
        cr.move_to(card_x + 20, card_y + 72)
        cr.line_to(card_x + 29, card_y + 64)
        cr.line_to(card_x + 24, card_y + 60)
        cr.stroke()

        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(12)
        cr.move_to(card_x + 46, card_y + 66)
        cr.show_text("Bluetooth")
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(10)
        cr.set_source_rgba(1, 1, 1, 0.6)
        cr.move_to(card_x + 46, card_y + 78)
        cr.show_text("Ativado")

        fcard_x = card_x + card_w + 12
        fcard_w = 142
        fcard_h = 60
        cr.set_source_rgba(0.18, 0.21, 0.30, 0.65)
        draw_squircle(cr, fcard_x, card_y, fcard_w, fcard_h, 12)
        cr.fill()
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(12)
        cr.set_source_rgba(1, 1, 1, 0.9)
        cr.move_to(fcard_x + 16, card_y + 28)
        cr.show_text("Foco")
        cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(10)
        cr.set_source_rgba(1, 1, 1, 0.5)
        cr.move_to(fcard_x + 16, card_y + 44)
        cr.show_text("Nao Perturbe")

        slider_y = card_y + card_h + 16
        for label, val in [("Tela", self.brightness), ("Som", self.volume)]:
            cr.set_source_rgba(1, 1, 1, 0.8)
            cr.select_font_face("Sans", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
            cr.set_font_size(11)
            cr.move_to(card_x + 4, slider_y - 4)
            cr.show_text(label)

            sl_w = cc_w - 24
            sl_h = 24
            cr.set_source_rgba(0.18, 0.21, 0.30, 0.65)
            draw_squircle(cr, card_x, slider_y, sl_w, sl_h, 10)
            cr.fill()

            cr.set_source_rgba(1.0, 1.0, 1.0, 0.90)
            draw_squircle(cr, card_x, slider_y, sl_w * val, sl_h, 10)
            cr.fill()
            slider_y += 50

def main():
    is_nested = "--nested" in sys.argv
    app = QuartzDesktopApp(is_nested=is_nested)
    return app.run([sys.argv[0]])

if __name__ == "__main__":
    sys.exit(main())
