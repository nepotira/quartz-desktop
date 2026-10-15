#!/usr/bin/env python3
# ==============================================================================
# 🍏 QuartzLinux (qz-desktop) — macOS Tahoe 26 Graphical Desktop Shell
# Interactive GTK4 + Cairo desktop environment with Liquid Glass,
# Parabolic Magnified Dock, Top Menu Bar, Traffic Lights, and Spotlight.
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

# --- Canonical macOS Tahoe 26 Design Constants ---
TRAFFIC_RED = (255 / 255.0, 95 / 255.0, 86 / 255.0)       # #FF5F56
TRAFFIC_RED_BORDER = (224 / 255.0, 68 / 255.0, 62 / 255.0) # #E0443E
TRAFFIC_RED_GLYPH = (77 / 255.0, 0 / 255.0, 0 / 255.0)     # #4D0000

TRAFFIC_YELLOW = (255 / 255.0, 189 / 255.0, 46 / 255.0)     # #FFBD2E
TRAFFIC_YELLOW_BORDER = (222 / 255.0, 161 / 255.0, 35 / 255.0) # #DEA123
TRAFFIC_YELLOW_GLYPH = (153 / 255.0, 87 / 255.0, 0 / 255.0) # #995700

TRAFFIC_GREEN = (39 / 255.0, 201 / 255.0, 63 / 255.0)       # #27C93F
TRAFFIC_GREEN_BORDER = (26 / 255.0, 171 / 255.0, 41 / 255.0) # #1AAB29
TRAFFIC_GREEN_GLYPH = (0 / 255.0, 101 / 255.0, 0 / 255.0)   # #006500

ACCENT_BLUE = (0 / 255.0, 122 / 255.0, 255 / 255.0)        # #007AFF

def draw_squircle(cr, x, y, w, h, r):
    """Draws a smooth squircle (continuous G2 superellipse approximation)."""
    # Using cubic bezier segments tuned to Apple squircle curvature
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

        # State tracking
        self.mouse_x = -1000.0
        self.mouse_y = -1000.0
        self.dock_hover = False
        self.active_menu = None
        self.spotlight_visible = False
        self.spotlight_query = ""
        self.control_center_visible = False
        self.brightness = 0.85
        self.volume = 0.70
        self.dark_mode = True

        # Window state (Finder)
        self.finder_visible = True
        self.finder_minimized = False
        self.finder_x = 220.0
        self.finder_y = 120.0
        self.finder_w = 780.0
        self.finder_h = 480.0
        self.finder_dragging = False
        self.drag_offset_x = 0.0
        self.drag_offset_y = 0.0
        self.traffic_group_hover = False
        self.traffic_hover_idx = -1  # 0: red, 1: yellow, 2: green
        self.traffic_dirty = False
        self.traffic_pressed = -1

        # Window state (Terminal)
        self.terminal_visible = False
        self.terminal_minimized = False
        self.terminal_x = 340.0
        self.terminal_y = 180.0
        self.terminal_w = 680.0
        self.terminal_h = 420.0
        self.terminal_dragging = False
        self.term_drag_offset_x = 0.0
        self.term_drag_offset_y = 0.0
        self.term_output = [
            "QuartzLinux (macOS Tahoe 26 Architecture) — Smithay + Vulkan 1.3",
            "Apple M-Series / CachyOS Linux x86_64 Kernel 6.13+",
            "Digite comandos ou clique no Dock para interagir.",
            "nepo@cachyos-tahoe ~ % "
        ]

        # Active focused window: 'finder' or 'terminal'
        self.active_window = 'finder'

        # Dock items
        self.dock_items = [
            {"id": "finder", "name": "Finder", "icon": "finder", "running": True, "bounce": 0.0},
            {"id": "safari", "name": "Safari", "icon": "safari", "running": False, "bounce": 0.0},
            {"id": "terminal", "name": "Terminal", "icon": "terminal", "running": True, "bounce": 0.0},
            {"id": "settings", "name": "Ajustes do Sistema", "icon": "settings", "running": False, "bounce": 0.0},
            {"id": "activity", "name": "Monitor de Atividade", "icon": "activity", "running": False, "bounce": 0.0},
            {"id": "preview", "name": "Pré-Visualização", "icon": "preview", "running": False, "bounce": 0.0},
            {"id": "sep", "name": "", "icon": "sep", "running": False, "bounce": 0.0},
            {"id": "trash", "name": "Lixo", "icon": "trash", "running": False, "bounce": 0.0},
        ]

        # Main Drawing Area
        self.area = Gtk.DrawingArea()
        self.area.set_draw_func(self.on_draw)
        self.set_child(self.area)

        # Event controllers
        motion_ctrl = Gtk.EventControllerMotion()
        motion_ctrl.connect("motion", self.on_motion)
        motion_ctrl.connect("leave", self.on_leave)
        self.area.add_controller(motion_ctrl)

        click_ctrl = Gtk.GestureClick()
        click_ctrl.connect("pressed", self.on_click_pressed)
        click_ctrl.connect("released", self.on_click_released)
        self.area.add_controller(click_ctrl)

        key_ctrl = Gtk.EventControllerKey()
        key_ctrl.connect("key-pressed", self.on_key_pressed)
        self.add_controller(key_ctrl)

        # 60Hz/120Hz Animation tick timer
        GLib.timeout_add(16, self.on_tick)

    def on_tick(self):
        # Update bounce physics
        need_redraw = False
        for item in self.dock_items:
            if item["bounce"] > 0.01:
                item["bounce"] = max(0.0, item["bounce"] - 0.08)
                need_redraw = True

        self.area.queue_draw()
        return True

    def on_motion(self, controller, x, y):
        self.mouse_x = x
        self.mouse_y = y

        # Dragging Finder
        if self.finder_dragging:
            self.finder_x = x - self.drag_offset_x
            self.finder_y = y - self.drag_offset_y
            self.area.queue_draw()
            return

        # Dragging Terminal
        if self.terminal_dragging:
            self.terminal_x = x - self.term_drag_offset_x
            self.terminal_y = y - self.term_drag_offset_y
            self.area.queue_draw()
            return

        # Check traffic lights hover on Finder
        if self.finder_visible and not self.finder_minimized:
            tl_x = self.finder_x + 20
            tl_y = self.finder_y + 18
            # Traffic lights bounding box: 3 circles of diameter 12, spacing 8 -> total width 52, height 20
            if (tl_x - 6 <= x <= tl_x + 58) and (tl_y - 6 <= y <= tl_y + 18):
                self.traffic_group_hover = True
                # Check specific light
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
            if x <= 40: # Apple menu
                self.active_menu = "apple" if self.active_menu != "apple" else None
            elif 45 <= x <= 100: # Finder menu
                self.active_menu = "finder" if self.active_menu != "finder" else None
            elif 105 <= x <= 145: # File
                self.active_menu = "file" if self.active_menu != "file" else None
            elif 150 <= x <= 190: # Edit
                self.active_menu = "edit" if self.active_menu != "edit" else None
            elif 195 <= x <= 240: # View
                self.active_menu = "view" if self.active_menu != "view" else None
            elif 245 <= x <= 280: # Go
                self.active_menu = "go" if self.active_menu != "go" else None
            elif 285 <= x <= 345: # Window
                self.active_menu = "window" if self.active_menu != "window" else None
            elif 350 <= x <= 390: # Help
                self.active_menu = "help" if self.active_menu != "help" else None

            # Right status items
            w = self.area.get_width()
            if w - 190 <= x <= w - 165: # Search / Spotlight
                self.spotlight_visible = not self.spotlight_visible
                self.active_menu = None
            elif w - 160 <= x <= w - 130: # Control Center
                self.control_center_visible = not self.control_center_visible
                self.active_menu = None
            self.area.queue_draw()
            return

        # If a menu was open and clicked outside, close it
        if self.active_menu:
            # Check if clicked inside menu dropdown (x: 8 to 220, y: 32 to 240)
            if not (8 <= x <= 220 and 32 <= y <= 260):
                self.active_menu = None
                self.area.queue_draw()

        # If Control Center was open and clicked outside, close it
        if self.control_center_visible:
            w = self.area.get_width()
            if not (w - 320 <= x <= w - 16 and 36 <= y <= 340):
                self.control_center_visible = False
                self.area.queue_draw()

        # If Spotlight was open and clicked outside, close it
        if self.spotlight_visible:
            w = self.area.get_width()
            sp_x = (w - 680) / 2
            sp_y = self.area.get_height() * 0.20
            if not (sp_x <= x <= sp_x + 680 and sp_y <= y <= sp_y + 260):
                self.spotlight_visible = False
                self.area.queue_draw()

        # 2. Check Terminal window header
        if self.terminal_visible and not self.terminal_minimized:
            if (self.terminal_x <= x <= self.terminal_x + self.terminal_w and
                self.terminal_y <= y <= self.terminal_y + 36):
                # Traffic lights on Terminal
                tl_x = self.terminal_x + 20
                tl_y = self.terminal_y + 18
                if math.hypot(x - (tl_x + 6), y - (tl_y + 6)) <= 8:
                    self.terminal_visible = False
                    self.area.queue_draw()
                    return
                elif math.hypot(x - (tl_x + 26), y - (tl_y + 6)) <= 8:
                    self.terminal_minimized = True
                    self.area.queue_draw()
                    return

                # Start drag
                self.terminal_dragging = True
                self.active_window = 'terminal'
                self.term_drag_offset_x = x - self.terminal_x
                self.term_drag_offset_y = y - self.terminal_y
                self.area.queue_draw()
                return
            elif (self.terminal_x <= x <= self.terminal_x + self.terminal_w and
                  self.terminal_y <= y <= self.terminal_y + self.terminal_h):
                self.active_window = 'terminal'
                self.area.queue_draw()

        # 3. Check Finder window header & traffic lights
        if self.finder_visible and not self.finder_minimized:
            if (self.finder_x <= x <= self.finder_x + self.finder_w and
                self.finder_y <= y <= self.finder_y + 48):
                tl_x = self.finder_x + 20
                tl_y = self.finder_y + 18
                if math.hypot(x - (tl_x + 6), y - (tl_y + 6)) <= 8: # Close
                    self.finder_visible = False
                    self.traffic_pressed = 0
                    self.area.queue_draw()
                    return
                elif math.hypot(x - (tl_x + 26), y - (tl_y + 6)) <= 8: # Minimize
                    self.finder_minimized = True
                    self.traffic_pressed = 1
                    self.area.queue_draw()
                    return
                elif math.hypot(x - (tl_x + 46), y - (tl_y + 6)) <= 8: # Zoom
                    if self.finder_w > 1000:
                        self.finder_x, self.finder_y, self.finder_w, self.finder_h = 220, 120, 780, 480
                    else:
                        w, h = self.area.get_width(), self.area.get_height()
                        self.finder_x, self.finder_y, self.finder_w, self.finder_h = 20, 40, w - 40, h - 140
                    self.traffic_pressed = 2
                    self.area.queue_draw()
                    return

                # Start drag
                self.finder_dragging = True
                self.active_window = 'finder'
                self.drag_offset_x = x - self.finder_x
                self.drag_offset_y = y - self.finder_y
                self.area.queue_draw()
                return
            elif (self.finder_x <= x <= self.finder_x + self.finder_w and
                  self.finder_y <= y <= self.finder_y + self.finder_h):
                self.active_window = 'finder'
                self.area.queue_draw()

        # 4. Check Dock clicks
        w, h = self.area.get_width(), self.area.get_height()
        dock_y = h - 84
        if y >= dock_y - 20:
            dock_info = self.calculate_dock_layout(w, h)
            for item in dock_info:
                ix, iy, sz, it_data = item["x"], item["y"], item["size"], item["data"]
                if ix <= x <= ix + sz and iy <= y <= iy + sz:
                    it_data["bounce"] = 1.0
                    it_id = it_data["id"]
                    if it_id == "finder":
                        self.finder_visible = True
                        self.finder_minimized = False
                        self.active_window = 'finder'
                    elif it_id == "terminal":
                        self.terminal_visible = True
                        self.terminal_minimized = False
                        self.active_window = 'terminal'
                    elif it_id == "settings":
                        self.spotlight_visible = False
                        self.control_center_visible = True
                    self.area.queue_draw()
                    break

    def on_click_released(self, gesture, n_press, x, y):
        self.finder_dragging = False
        self.terminal_dragging = False
        self.traffic_pressed = -1
        self.area.queue_draw()

    def on_key_pressed(self, controller, keyval, keycode, state):
        # Cmd+Space or Ctrl+Space to toggle Spotlight
        if (state & (Gdk.ModifierType.CONTROL_MASK | Gdk.ModifierType.SUPER_MASK)) and keyval == Gdk.KEY_space:
            self.spotlight_visible = not self.spotlight_visible
            self.area.queue_draw()
            return True
        # Escape closes Spotlight and Control Center
        if keyval == Gdk.KEY_Escape:
            self.spotlight_visible = False
            self.control_center_visible = False
            self.active_menu = None
            self.area.queue_draw()
            return True
        # Typing in Spotlight
        if self.spotlight_visible:
            if keyval == Gdk.KEY_BackSpace:
                self.spotlight_query = self.spotlight_query[:-1]
            elif keyval == Gdk.KEY_Return:
                # Launch first match
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
        if self.active_window == 'terminal' and self.terminal_visible:
            if keyval == Gdk.KEY_Return:
                cmd = self.term_output[-1].replace("nepo@cachyos-tahoe ~ % ", "").strip()
                if cmd == "clear":
                    self.term_output = ["nepo@cachyos-tahoe ~ % "]
                elif cmd == "uname -a":
                    self.term_output.append("Linux cachyos-tahoe 6.13.0-cachyos #1 SMP PREEMPT_DYNAMIC x86_64 GNU/Linux")
                    self.term_output.append("nepo@cachyos-tahoe ~ % ")
                elif cmd in ("neofetch", "fastfetch"):
                    self.term_output.append("             OS: QuartzLinux (macOS Tahoe 26 on CachyOS)")
                    self.term_output.append("    #####     Host: Apple Silicon / CachyOS Linux x86_64")
                    self.term_output.append("   #######    Kernel: 6.13.0-cachyos")
                    self.term_output.append("  #########   Compositor: Smithay + Vulkan 1.3 Liquid Glass")
                    self.term_output.append("  #########   Theme: macOS Tahoe 26 Aqua Glass [Dark]")
                    self.term_output.append("   #######    Icons: SF Pro Symbols + Tahoe 26 Vector")
                    self.term_output.append("    #####     Memory: 4120MiB / 32140MiB")
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
        """Calculates parabolic magnification layout for dock items."""
        base_size = 54.0
        max_size = 88.0
        influence_radius = 120.0
        spacing = 10.0

        num_items = len(self.dock_items)
        dock_y = h - 76.0

        # Calculate sizes based on mouse X position
        sizes = []
        is_hovering_dock = (h - 96.0 <= self.mouse_y <= h)

        for i, item in enumerate(self.dock_items):
            if item["id"] == "sep":
                sizes.append(12.0)
                continue
            if is_hovering_dock and self.mouse_x >= 0:
                # Nominal center
                nom_x = (w / 2.0) + (i - num_items / 2.0) * (base_size + spacing)
                dist = abs(self.mouse_x - nom_x)
                if dist < influence_radius:
                    # Parabolic cosine magnification
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
            # Bounce offset
            bounce_dy = item["bounce"] * 24.0 * math.sin(item["bounce"] * math.pi)
            if item["id"] == "sep":
                layout.append({
                    "x": curr_x,
                    "y": dock_y + 10,
                    "size": sz,
                    "height": base_size - 12,
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
        # 1. Background Wallpaper (macOS Tahoe 26 Dynamic Liquid Mesh Gradient)
        pattern = cairo.RadialGradient(width * 0.7, height * 0.3, 50, width * 0.5, height * 0.5, max(width, height))
        pattern.add_color_stop_rgba(0.0, 0.12, 0.22, 0.45, 1.0) # Deep Tahoe Royal Blue
        pattern.add_color_stop_rgba(0.4, 0.08, 0.12, 0.32, 1.0) # Deep Midnight Purple
        pattern.add_color_stop_rgba(0.8, 0.05, 0.07, 0.18, 1.0) # Deep Oceanic Dark
        pattern.add_color_stop_rgba(1.0, 0.02, 0.04, 0.10, 1.0) # Bottom Obsidian
        cr.set_source(pattern)
        cr.rectangle(0, 0, width, height)
        cr.fill()

        # Dynamic glowing chromatic arcs on wallpaper
        cr.save()
        arc_pat = cairo.LinearGradient(width * 0.2, 0, width * 0.8, height)
        arc_pat.add_color_stop_rgba(0.0, 0.25, 0.55, 0.95, 0.22)
        arc_pat.add_color_stop_rgba(0.5, 0.65, 0.25, 0.85, 0.18)
        arc_pat.add_color_stop_rgba(1.0, 0.15, 0.85, 0.80, 0.15)
        cr.set_source(arc_pat)
        draw_squircle(cr, width * 0.15, height * 0.15, width * 0.70, height * 0.65, 180)
        cr.fill()
        cr.restore()

        # 2. Render Windows
        # Terminal Window (if behind or active)
        if self.terminal_visible and not self.terminal_minimized:
            self.draw_terminal_window(cr)

        # Finder Window
        if self.finder_visible and not self.finder_minimized:
            self.draw_finder_window(cr)

        # 3. Parabolic Magnified Liquid Glass Dock
        self.draw_dock(cr, width, height)

        # 4. Top Menu Bar (Liquid Glass 28px)
        self.draw_menubar(cr, width)

        # 5. Dropdown Menu (if open)
        if self.active_menu:
            self.draw_dropdown_menu(cr)

        # 6. Control Center (if open)
        if self.control_center_visible:
            self.draw_control_center(cr, width)

        # 7. Spotlight Search (if open)
        if self.spotlight_visible:
            self.draw_spotlight(cr, width, height)

    def draw_menubar(self, cr, width):
        """Renders 28px Liquid Glass Top Menu Bar."""
        # Frosted glass background
        cr.set_source_rgba(0.08, 0.10, 0.15, 0.75)
        cr.rectangle(0, 0, width, 28)
        cr.fill()

        # 1px bottom rim highlight
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.14)
        cr.rectangle(0, 27.5, width, 0.5)
        cr.fill()

        # Left side items
        cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(14)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.95)

        # Apple Logo
        cr.move_to(14, 19)
        cr.show_text("")

        # Active App Name (Finder)
        cr.move_to(42, 19)
        cr.show_text("Finder")

        # Menus
        cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(13)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.88)

        menus = [("Arquivo", 108), ("Editar", 168), ("Visualizar", 218), ("Ir", 294), ("Janela", 324), ("Ajuda", 380)]
        for name, x in menus:
            cr.move_to(x, 19)
            cr.show_text(name)

        # Right side status tray
        cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(12)

        # Wi-Fi & Battery & Control Center & Spotlight
        cr.move_to(width - 240, 19)
        cr.show_text("100% 􀛨") # Battery

        cr.move_to(width - 195, 19)
        cr.show_text("􀙇") # Wi-Fi

        cr.move_to(width - 170, 19)
        cr.show_text("􀊫") # Search / Spotlight

        cr.move_to(width - 145, 19)
        cr.show_text("􀜊") # Control Center

        # Dynamic Clock (Qua 15 out 09:35)
        time_str = time.strftime("%a %d %b %H:%M")
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.95)
        cr.move_to(width - 115, 19)
        cr.show_text(time_str)

    def draw_dropdown_menu(self, cr):
        """Draws Apple Menu or App dropdown with Tahoe 26 styling."""
        menu_x = 8.0
        menu_y = 32.0
        menu_w = 210.0
        menu_h = 240.0

        # Shadow
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.35)
        draw_squircle(cr, menu_x, menu_y + 4, menu_w, menu_h, 10)
        cr.fill()

        # Frosted Glass body
        cr.set_source_rgba(0.12, 0.14, 0.20, 0.90)
        draw_squircle(cr, menu_x, menu_y, menu_w, menu_h, 10)
        cr.fill()

        # 1px border highlight
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.18)
        draw_squircle(cr, menu_x, menu_y, menu_w, menu_h, 10)
        cr.set_line_width(1.0)
        cr.stroke()

        # Items
        items = [
            "Sobre Este Mac",
            "Ajustes do Sistema...",
            "App Store...",
            "---",
            "Forçar Encerrar...",
            "---",
            "Repouso",
            "Reiniciar...",
            "Desligar...",
            "---",
            "Bloquear Tela",
            "Encerrar Sessão..."
        ]
        cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(13)

        cur_y = menu_y + 18
        for it in items:
            if it == "---":
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.12)
                cr.rectangle(menu_x + 12, cur_y - 4, menu_w - 24, 1)
                cr.fill()
                cur_y += 10
            else:
                # Hover highlight
                if menu_x <= self.mouse_x <= menu_x + menu_w and cur_y - 14 <= self.mouse_y <= cur_y + 6:
                    cr.set_source_rgba(*ACCENT_BLUE, 0.85)
                    draw_squircle(cr, menu_x + 6, cur_y - 14, menu_w - 12, 20, 5)
                    cr.fill()
                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0)
                else:
                    cr.set_source_rgba(1.0, 1.0, 1.0, 0.88)

                cr.move_to(menu_x + 16, cur_y)
                cr.show_text(it)
                cur_y += 20

    def draw_finder_window(self, cr):
        """Draws the main interactive Finder Window with Traffic Lights and Floating Sidebar."""
        x, y, w, h = self.finder_x, self.finder_y, self.finder_w, self.finder_h

        # Dual-Layer Analytical Shadow
        # Layer 1: Ambient Occlusion (sigma=4, dy=2, alpha=0.28)
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.28)
        draw_squircle(cr, x - 2, y + 2, w + 4, h + 4, 14)
        cr.fill()
        # Layer 2: Penumbra (sigma=38, dy=22, alpha=0.42)
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.42)
        draw_squircle(cr, x - 12, y + 16, w + 24, h + 24, 20)
        cr.fill()

        # Window body (Tahoe 26 Frosted Liquid Glass)
        cr.set_source_rgba(0.13, 0.15, 0.22, 0.88)
        draw_squircle(cr, x, y, w, h, 12)
        cr.fill()

        # 1px specular rim highlight
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.24)
        draw_squircle(cr, x, y, w, h, 12)
        cr.set_line_width(1.0)
        cr.stroke()

        # Title bar divider
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.10)
        cr.rectangle(x, y + 46, w, 1)
        cr.fill()

        # Window Title
        cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(13)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.90)
        title = "Finder — Macintosh HD"
        extents = cr.text_extents(title)
        cr.move_to(x + (w - extents.width) / 2.0, y + 28)
        cr.show_text(title)

        # Traffic Lights (Close, Minimize, Zoom)
        tl_x = x + 20
        tl_y = y + 18
        lights = [
            (TRAFFIC_RED, TRAFFIC_RED_BORDER, TRAFFIC_RED_GLYPH, "×"),
            (TRAFFIC_YELLOW, TRAFFIC_YELLOW_BORDER, TRAFFIC_YELLOW_GLYPH, "−"),
            (TRAFFIC_GREEN, TRAFFIC_GREEN_BORDER, TRAFFIC_GREEN_GLYPH, "⤢"),
        ]

        for i, (fill, border, glyph_color, symbol) in enumerate(lights):
            cx = tl_x + i * 20 + 6
            cy = tl_y + 6

            # Apply 18% luminance darkening if active pressed
            if self.traffic_pressed == i:
                f_color = (fill[0] * 0.82, fill[1] * 0.82, fill[2] * 0.82)
            else:
                f_color = fill

            # Fill circle
            cr.set_source_rgb(*f_color)
            cr.arc(cx, cy, 6, 0, 2 * math.pi)
            cr.fill()

            # 0.5px border
            cr.set_source_rgb(*border)
            cr.arc(cx, cy, 6, 0, 2 * math.pi)
            cr.set_line_width(0.8)
            cr.stroke()

            # Inner glyph (shows on group hover)
            if self.traffic_group_hover:
                cr.set_source_rgb(*glyph_color)
                cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
                cr.set_font_size(9)
                g_ext = cr.text_extents(symbol)
                cr.move_to(cx - g_ext.width / 2.0, cy + g_ext.height / 2.0 - 0.5)
                cr.show_text(symbol)

        # Tahoe 26 Floating Glass Pill Sidebar (inset_margin = 8px)
        sb_x = x + 8
        sb_y = y + 54
        sb_w = 175
        sb_h = h - 62
        cr.set_source_rgba(0.18, 0.20, 0.28, 0.60)
        draw_squircle(cr, sb_x, sb_y, sb_w, sb_h, 10)
        cr.fill()
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.08)
        draw_squircle(cr, sb_x, sb_y, sb_w, sb_h, 10)
        cr.set_line_width(0.8)
        cr.stroke()

        # Sidebar content
        cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(11)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.45)
        cr.move_to(sb_x + 14, sb_y + 22)
        cr.show_text("FAVORITOS")

        favorites = [
            ("􀈄 AirDrop", False),
            ("􀣋 Aplicativos", True), # Selected
            ("􀈕 Mesa (Desktop)", False),
            ("􀈷 Documentos", False),
            ("􀈄 Downloads", False),
            ("􀈀 iCloud Drive", False),
        ]
        cr.set_font_size(12)
        cur_sy = sb_y + 44
        for name, selected in favorites:
            if selected:
                cr.set_source_rgba(*ACCENT_BLUE, 0.85)
                draw_squircle(cr, sb_x + 6, cur_sy - 13, sb_w - 12, 22, 6)
                cr.fill()
                cr.set_source_rgba(1.0, 1.0, 1.0, 1.0)
            else:
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.82)
            cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
            cr.move_to(sb_x + 14, cur_sy + 2)
            cr.show_text(name)
            cur_sy += 26

        # Main file view content (grid of folders)
        content_x = sb_x + sb_w + 14
        content_y = y + 64
        folders = [
            ("Aplicativos", "􀣋"),
            ("Documentos", "􀈷"),
            ("Downloads", "􀈄"),
            ("Desenvolvedor", "􀌚"),
            ("Fotos", "􀅼"),
            ("Música", "􀑪")
        ]

        col_w = 110
        row_h = 100
        for idx, (fname, ficon) in enumerate(folders):
            row = idx // 4
            col = idx % 4
            fx = content_x + col * col_w
            fy = content_y + row * row_h

            # Folder squircle background
            cr.set_source_rgba(0.20, 0.45, 0.85, 0.25)
            draw_squircle(cr, fx + 16, fy + 8, 54, 46, 12)
            cr.fill()

            cr.set_source_rgba(0.35, 0.65, 1.0, 0.90)
            cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
            cr.set_font_size(24)
            cr.move_to(fx + 30, fy + 40)
            cr.show_text(ficon)

            cr.set_source_rgba(1.0, 1.0, 1.0, 0.90)
            cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
            cr.set_font_size(11)
            ext = cr.text_extents(fname)
            cr.move_to(fx + 43 - ext.width / 2.0, fy + 72)
            cr.show_text(fname)

    def draw_terminal_window(self, cr):
        """Draws GPU-accelerated Liquid Glass Terminal Emulator."""
        x, y, w, h = self.terminal_x, self.terminal_y, self.terminal_w, self.terminal_h

        # Shadow
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.40)
        draw_squircle(cr, x - 8, y + 12, w + 16, h + 16, 16)
        cr.fill()

        # Terminal body (Dark Frosted Glass 88% opacity)
        cr.set_source_rgba(0.06, 0.08, 0.12, 0.92)
        draw_squircle(cr, x, y, w, h, 12)
        cr.fill()

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.18)
        draw_squircle(cr, x, y, w, h, 12)
        cr.set_line_width(1.0)
        cr.stroke()

        # Title bar
        cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(12)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.85)
        title = "nepo — zsh — 80×24 (QuartzLinux Terminal)"
        ext = cr.text_extents(title)
        cr.move_to(x + (w - ext.width) / 2.0, y + 22)
        cr.show_text(title)

        # Traffic lights
        tl_x = x + 18
        tl_y = y + 14
        for i, fill in enumerate([TRAFFIC_RED, TRAFFIC_YELLOW, TRAFFIC_GREEN]):
            cx = tl_x + i * 18 + 5
            cy = tl_y + 5
            cr.set_source_rgb(*fill)
            cr.arc(cx, cy, 5.5, 0, 2 * math.pi)
            cr.fill()

        # Terminal text output
        cr.select_font_face("JetBrains Mono, Menlo, monospace", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(12)
        text_y = y + 54
        for line in self.terminal_output_view():
            if line.startswith("nepo@cachyos-tahoe"):
                cr.set_source_rgba(0.4, 0.85, 0.45, 1.0) # Green prompt
            elif "OS:" in line or "Kernel:" in line or "Apple" in line:
                cr.set_source_rgba(0.35, 0.75, 1.0, 0.95) # Cyan info
            else:
                cr.set_source_rgba(0.92, 0.92, 0.94, 0.90)
            cr.move_to(x + 18, text_y)
            cr.show_text(line)
            text_y += 18

        # Cursor
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.80)
        last_line = self.term_output[-1]
        ext = cr.text_extents(last_line)
        cr.rectangle(x + 18 + ext.width + 2, text_y - 30, 8, 14)
        cr.fill()

    def terminal_output_view(self):
        max_lines = int((self.terminal_h - 60) / 18)
        return self.term_output[-max_lines:]

    def draw_dock(self, cr, width, height):
        """Renders Parabolic Magnified Liquid Glass Dock."""
        dock_layout = self.calculate_dock_layout(width, height)
        if not dock_layout:
            return

        min_x = min(item["x"] for item in dock_layout) - 16
        max_x = max(item["x"] + item["size"] for item in dock_layout) + 16
        dock_w = max_x - min_x
        dock_h = 68.0
        dock_x = min_x
        dock_y = height - 76.0

        # Dock Shadow
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.35)
        draw_squircle(cr, dock_x, dock_y + 4, dock_w, dock_h, 20)
        cr.fill()

        # Dock Liquid Glass Body
        cr.set_source_rgba(0.12, 0.14, 0.20, 0.65)
        draw_squircle(cr, dock_x, dock_y, dock_w, dock_h, 20)
        cr.fill()

        # 1px border highlight
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.28)
        draw_squircle(cr, dock_x, dock_y, dock_w, dock_h, 20)
        cr.set_line_width(1.0)
        cr.stroke()

        # Render Dock Items
        hovered_item_name = None
        hovered_x = 0
        hovered_y = 0

        for it in dock_layout:
            ix, iy, sz, it_data = it["x"], it["y"], it["size"], it["data"]
            it_id = it_data["id"]

            if it_id == "sep":
                # Dock vertical separator
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.25)
                cr.rectangle(ix + sz / 2.0 - 0.5, iy, 1.0, it["height"])
                cr.fill()
                continue

            # Icon Squircle Background
            icon_r = sz * 0.22
            cr.save()
            draw_squircle(cr, ix, iy, sz, sz, icon_r)
            cr.clip()

            # Beautiful iconic gradients
            if it_id == "finder":
                pat = cairo.LinearGradient(ix, iy, ix, iy + sz)
                pat.add_color_stop_rgb(0.0, 0.40, 0.75, 1.0)
                pat.add_color_stop_rgb(1.0, 0.10, 0.45, 0.85)
                cr.set_source(pat)
                cr.paint()
                cr.set_source_rgb(1.0, 1.0, 1.0)
                cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
                cr.set_font_size(sz * 0.46)
                cr.move_to(ix + sz * 0.28, iy + sz * 0.68)
                cr.show_text("")

            elif it_id == "safari":
                pat = cairo.LinearGradient(ix, iy, ix, iy + sz)
                pat.add_color_stop_rgb(0.0, 0.25, 0.65, 1.0)
                pat.add_color_stop_rgb(1.0, 0.05, 0.30, 0.80)
                cr.set_source(pat)
                cr.paint()
                cr.set_source_rgb(1.0, 0.2, 0.2)
                cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
                cr.set_font_size(sz * 0.5)
                cr.move_to(ix + sz * 0.26, iy + sz * 0.70)
                cr.show_text("🧭")

            elif it_id == "terminal":
                cr.set_source_rgb(0.12, 0.14, 0.18)
                cr.paint()
                cr.set_source_rgb(0.40, 0.85, 0.50)
                cr.select_font_face("JetBrains Mono", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
                cr.set_font_size(sz * 0.45)
                cr.move_to(ix + sz * 0.22, iy + sz * 0.65)
                cr.show_text(">_")

            elif it_id == "settings":
                pat = cairo.LinearGradient(ix, iy, ix, iy + sz)
                pat.add_color_stop_rgb(0.0, 0.60, 0.62, 0.66)
                pat.add_color_stop_rgb(1.0, 0.35, 0.38, 0.42)
                cr.set_source(pat)
                cr.paint()
                cr.set_source_rgb(1.0, 1.0, 1.0)
                cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
                cr.set_font_size(sz * 0.48)
                cr.move_to(ix + sz * 0.25, iy + sz * 0.70)
                cr.show_text("⚙")

            elif it_id == "activity":
                cr.set_source_rgb(0.10, 0.12, 0.16)
                cr.paint()
                cr.set_source_rgb(0.20, 0.80, 0.40)
                cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
                cr.set_font_size(sz * 0.45)
                cr.move_to(ix + sz * 0.22, iy + sz * 0.68)
                cr.show_text("📈")

            elif it_id == "preview":
                pat = cairo.LinearGradient(ix, iy, ix, iy + sz)
                pat.add_color_stop_rgb(0.0, 0.30, 0.65, 0.95)
                pat.add_color_stop_rgb(1.0, 0.15, 0.45, 0.75)
                cr.set_source(pat)
                cr.paint()
                cr.set_source_rgb(1.0, 1.0, 1.0)
                cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
                cr.set_font_size(sz * 0.45)
                cr.move_to(ix + sz * 0.25, iy + sz * 0.68)
                cr.show_text("🌄")

            elif it_id == "trash":
                cr.set_source_rgba(0.25, 0.28, 0.35, 0.8)
                cr.paint()
                cr.set_source_rgb(0.9, 0.9, 0.9)
                cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
                cr.set_font_size(sz * 0.45)
                cr.move_to(ix + sz * 0.25, iy + sz * 0.68)
                cr.show_text("🗑")

            cr.restore()

            # Specular icon rim
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.22)
            draw_squircle(cr, ix, iy, sz, sz, icon_r)
            cr.set_line_width(0.8)
            cr.stroke()

            # Running Indicator Dot (4px white dot 2px below)
            if it_data["running"]:
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.95)
                cr.arc(ix + sz / 2.0, dock_y + dock_h - 6, 2.0, 0, 2 * math.pi)
                cr.fill()

            # Check hover for tooltip
            if ix <= self.mouse_x <= ix + sz and iy <= self.mouse_y <= iy + sz:
                hovered_item_name = it_data["name"]
                hovered_x = ix + sz / 2.0
                hovered_y = iy - 14.0

        # Draw Tooltip above hovered item
        if hovered_item_name:
            cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
            cr.set_font_size(12)
            ext = cr.text_extents(hovered_item_name)
            tt_w = ext.width + 16.0
            tt_h = 24.0
            tt_x = hovered_x - tt_w / 2.0
            tt_y = hovered_y - tt_h

            cr.set_source_rgba(0.10, 0.12, 0.18, 0.85)
            draw_squircle(cr, tt_x, tt_y, tt_w, tt_h, 6)
            cr.fill()

            cr.set_source_rgba(1.0, 1.0, 1.0, 0.20)
            draw_squircle(cr, tt_x, tt_y, tt_w, tt_h, 6)
            cr.set_line_width(0.8)
            cr.stroke()

            cr.set_source_rgba(1.0, 1.0, 1.0, 0.95)
            cr.move_to(tt_x + 8.0, tt_y + 16.0)
            cr.show_text(hovered_item_name)

    def draw_spotlight(self, cr, width, height):
        """Renders macOS Spotlight Search overlay."""
        sp_w = 680.0
        sp_h = 56.0 if not self.spotlight_query else 240.0
        sp_x = (width - sp_w) / 2.0
        sp_y = height * 0.20

        # Shadow
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.45)
        draw_squircle(cr, sp_x - 8, sp_y + 12, sp_w + 16, sp_h + 16, 20)
        cr.fill()

        # Frosted glass body
        cr.set_source_rgba(0.12, 0.15, 0.22, 0.92)
        draw_squircle(cr, sp_x, sp_y, sp_w, sp_h, 16)
        cr.fill()

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.28)
        draw_squircle(cr, sp_x, sp_y, sp_w, sp_h, 16)
        cr.set_line_width(1.2)
        cr.stroke()

        # Search icon
        cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(20)
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.60)
        cr.move_to(sp_x + 20, sp_y + 36)
        cr.show_text("􀊫")

        # Text input / placeholder
        cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(18)
        if not self.spotlight_query:
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.35)
            cr.move_to(sp_x + 56, sp_y + 36)
            cr.show_text("Buscar no Spotlight")
        else:
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.95)
            cr.move_to(sp_x + 56, sp_y + 36)
            cr.show_text(self.spotlight_query)

            # Cursor
            ext = cr.text_extents(self.spotlight_query)
            cr.set_source_rgba(*ACCENT_BLUE, 0.90)
            cr.rectangle(sp_x + 56 + ext.width + 2, sp_y + 18, 2, 22)
            cr.fill()

            # Search Results List
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.10)
            cr.rectangle(sp_x, sp_y + 56, sp_w, 1)
            cr.fill()

            results = [
                ("Terminal", "Aplicativo — /usr/local/bin/qz-terminal", "terminal"),
                ("Ajustes do Sistema", "Aplicativo — /usr/local/bin/qz-settings", "settings"),
                ("Monitor de Atividade", "Aplicativo — /usr/local/bin/qz-activity", "activity"),
                ("Finder", "Gerenciador de Arquivos do macOS Tahoe", "finder"),
            ]
            matched = [r for r in results if self.spotlight_query.lower() in r[0].lower()]
            if not matched:
                matched = results

            cur_ry = sp_y + 86
            for idx, (r_name, r_desc, r_icon) in enumerate(matched[:4]):
                if idx == 0:
                    cr.set_source_rgba(*ACCENT_BLUE, 0.85)
                    draw_squircle(cr, sp_x + 8, cur_ry - 20, sp_w - 16, 38, 8)
                    cr.fill()
                    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0)
                else:
                    cr.set_source_rgba(1.0, 1.0, 1.0, 0.85)

                cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
                cr.set_font_size(14)
                cr.move_to(sp_x + 46, cur_ry - 2)
                cr.show_text(r_name)

                cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
                cr.set_font_size(11)
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.65 if idx == 0 else 0.45)
                cr.move_to(sp_x + 46, cur_ry + 13)
                cr.show_text(r_desc)
                cur_ry += 42

    def draw_control_center(self, cr, width):
        """Renders macOS Control Center panel."""
        cc_w = 320.0
        cc_h = 320.0
        cc_x = width - cc_w - 16.0
        cc_y = 36.0

        # Shadow
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.40)
        draw_squircle(cr, cc_x - 6, cc_y + 8, cc_w + 12, cc_h + 12, 18)
        cr.fill()

        # Frosted glass body
        cr.set_source_rgba(0.12, 0.15, 0.22, 0.90)
        draw_squircle(cr, cc_x, cc_y, cc_w, cc_h, 16)
        cr.fill()

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.20)
        draw_squircle(cr, cc_x, cc_y, cc_w, cc_h, 16)
        cr.set_line_width(1.0)
        cr.stroke()

        # Connectivity Card
        card_x = cc_x + 12
        card_y = cc_y + 12
        card_w = 142
        card_h = 130
        cr.set_source_rgba(0.18, 0.21, 0.30, 0.65)
        draw_squircle(cr, card_x, card_y, card_w, card_h, 12)
        cr.fill()

        # Wi-Fi button
        cr.set_source_rgba(*ACCENT_BLUE, 0.95)
        cr.arc(card_x + 24, card_y + 26, 14, 0, 2 * math.pi)
        cr.fill()
        cr.set_source_rgb(1, 1, 1)
        cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(12)
        cr.move_to(card_x + 46, card_y + 24)
        cr.show_text("Wi-Fi")
        cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(10)
        cr.set_source_rgba(1, 1, 1, 0.6)
        cr.move_to(card_x + 46, card_y + 36)
        cr.show_text("CachyOS-5G")

        # Bluetooth button
        cr.set_source_rgba(*ACCENT_BLUE, 0.95)
        cr.arc(card_x + 24, card_y + 68, 14, 0, 2 * math.pi)
        cr.fill()
        cr.set_source_rgb(1, 1, 1)
        cr.select_font_face("SF Pro Display", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(12)
        cr.move_to(card_x + 46, card_y + 66)
        cr.show_text("Bluetooth")
        cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(10)
        cr.set_source_rgba(1, 1, 1, 0.6)
        cr.move_to(card_x + 46, card_y + 78)
        cr.show_text("Ativado")

        # Focus Mode Card
        fcard_x = card_x + card_w + 12
        fcard_w = 142
        fcard_h = 60
        cr.set_source_rgba(0.18, 0.21, 0.30, 0.65)
        draw_squircle(cr, fcard_x, card_y, fcard_w, fcard_h, 12)
        cr.fill()
        cr.set_source_rgba(1, 1, 1, 0.9)
        cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
        cr.set_font_size(12)
        cr.move_to(fcard_x + 16, card_y + 28)
        cr.show_text("􀆹 Foco")
        cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_NORMAL)
        cr.set_font_size(10)
        cr.set_source_rgba(1, 1, 1, 0.5)
        cr.move_to(fcard_x + 16, card_y + 44)
        cr.show_text("Não Perturbe")

        # Sliders (Brightness & Volume)
        slider_y = card_y + card_h + 16
        for label, icon, val in [("Tela", "􀆮", self.brightness), ("Som", "􀊩", self.volume)]:
            cr.set_source_rgba(1, 1, 1, 0.8)
            cr.select_font_face("SF Pro Text", cairo.FONT_SLANT_NORMAL, cairo.FONT_WEIGHT_BOLD)
            cr.set_font_size(11)
            cr.move_to(card_x + 4, slider_y - 4)
            cr.show_text(label)

            # Slider track
            sl_w = cc_w - 24
            sl_h = 24
            cr.set_source_rgba(0.18, 0.21, 0.30, 0.65)
            draw_squircle(cr, card_x, slider_y, sl_w, sl_h, 10)
            cr.fill()

            # Slider fill
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
