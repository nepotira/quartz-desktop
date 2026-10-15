#version 450
// ============================================================================
// 🍏 macOS Tahoe 26 Procedural SDF UI Uber Shader (Zero External Fonts/Icons)
// Mathematical boolean geometry and vector rasterizer for all desktop icons
// ============================================================================

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec2 v_local_pos;
layout(location = 0) out vec4 frag_color;

layout(push_constant) uniform UberUniforms {
    int icon_mode;     // 0: Apple, 1: Folder, 2: Finder, 3: Safari, 4: Terminal, 5: Settings, 6: Launchpad, 7: Trash, 8: Wifi, 9: Battery, 10: Search
    float param_a;     // E.g. battery charge level or hover factor
    float param_b;     // Secondary parameter
    int is_dark;       // 1: Dark Mode, 0: Light Mode
} u;

// --- Primitive Distance Functions ---
float sdCircle(vec2 p, float r) { 
    return length(p) - r; 
}

float sdRoundBox(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
}

float sdSegment(vec2 p, vec2 a, vec2 b, float r) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

// 1. LOGOTIPO DA MAÇÃ () - Geometria Booleana SDF Exata
float sdAppleLogo(vec2 p) {
    p.y = -p.y; // Y apontando para cima
    p *= 1.15;
    // Corpo principal composto por dois lóbulos simétricos afunilados na base
    vec2 q = vec2(abs(p.x), p.y);
    float lobe = length((q - vec2(0.24, -0.05)) * vec2(1.0, 0.88)) - 0.44;
    float bottom_cut = -(p.y + 0.52 - 0.12 * exp(-18.0 * p.x * p.x));
    float top_indent = length(p - vec2(0.0, 0.36)) - 0.16;
    float body = max(lobe, bottom_cut);
    body = max(body, -top_indent);
    // Mordida lateral direita (subtração booleana)
    float bite = length(p - vec2(0.62, 0.06)) - 0.27;
    body = max(body, -bite);
    // Folha superior inclinada a 40 graus
    vec2 lp = p - vec2(0.10, 0.56);
    float c = cos(0.70), s = sin(0.70);
    lp = mat2(c, -s, s, c) * lp;
    float leaf = length(lp * vec2(1.8, 0.9)) - 0.19;
    return min(body, leaf);
}

// 2. ÍCONE REAL DE PASTA DO macOS TAHOE 26 (Com Aba Traseira, Gradiente e Specular)
vec4 renderMacOSFolder(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    // Aba traseira (Back Plate + Tab superior esquerda)
    float back_main = sdRoundBox(p - vec2(0.0, -0.05), vec2(0.86, 0.62), 0.12);
    float back_tab  = sdRoundBox(p - vec2(-0.46, -0.58), vec2(0.38, 0.22), 0.10);
    float d_back = min(back_main, back_tab);
    // Corpo frontal da pasta (Front Sheet)
    float d_front = sdRoundBox(p - vec2(0.0, 0.08), vec2(0.88, 0.58), 0.13);

    vec4 col = vec4(0.0);
    // Sombra projetada da pasta
    float d_shadow = sdRoundBox(p - vec2(0.0, 0.14), vec2(0.86, 0.56), 0.14);
    float shadow_a = (1.0 - smoothstep(-0.05, 0.22, d_shadow)) * 0.35;
    col = vec4(0.0, 0.0, 0.0, shadow_a);

    // Pintura da aba traseira (#2897E8 -> #1A73E8)
    if (d_back < 0.02) {
        float a = 1.0 - smoothstep(-0.015, 0.015, d_back);
        vec3 back_rgb = mix(vec3(0.20, 0.62, 0.94), vec3(0.11, 0.45, 0.82), clamp(uv.y, 0.0, 1.0));
        col = mix(col, vec4(back_rgb, 1.0), a);
    }

    // Sombra interna entre a folha frontal e a aba traseira
    float inner_ao = (1.0 - smoothstep(-0.02, 0.08, abs(d_front))) * step(d_back, 0.0);
    col.rgb *= (1.0 - inner_ao * 0.25);

    // Pintura da folha frontal com gradiente luminoso Tahoe 26 (#74D8FF -> #0084FF)
    if (d_front < 0.02) {
        float a = 1.0 - smoothstep(-0.015, 0.015, d_front);
        float gy = clamp((uv.y - 0.25) / 0.65, 0.0, 1.0);
        vec3 front_rgb = mix(vec3(0.45, 0.85, 1.00), vec3(0.05, 0.52, 0.98), gy);
        // Linha especular de 1px no topo da folha frontal
        float top_rim = exp(-pow((p.y - (-0.49)) * 45.0, 2.0)) * step(abs(p.x), 0.78);
        front_rgb += vec3(0.35, 0.40, 0.45) * top_rim;
        col = mix(col, vec4(front_rgb, 1.0), a);
    }
    return col;
}

// 3. ÍCONE DO FINDER NO DOCK (Rosto Bicolor Sorridente)
vec4 renderFinderDockIcon(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    vec2 q = abs(p) - vec2(0.62);
    float d_squircle = pow(pow(max(q.x, 0.0), 4.0) + pow(max(q.y, 0.0), 4.0), 0.25) + min(max(q.x, q.y), 0.0) - 0.26;
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_squircle);
    if (mask <= 0.001) return vec4(0.0);

    // Curva divisória central do nariz/rosto
    float split_x = -0.06 * sin(p.y * 2.2) + ((p.y > 0.05) ? 0.10 : -0.04);
    float side = smoothstep(split_x - 0.015, split_x + 0.015, p.x);

    vec3 left_blue  = mix(vec3(0.18, 0.65, 0.98), vec3(0.08, 0.45, 0.88), uv.y);
    vec3 right_pale = mix(vec3(0.95, 0.98, 1.00), vec3(0.80, 0.90, 0.98), uv.y);
    vec3 rgb = mix(left_blue, right_pale, side);

    // Olhos verticais em pílula
    float eye_l = sdSegment(p, vec2(-0.34, -0.26), vec2(-0.34, -0.10), 0.048);
    float eye_r = sdSegment(p, vec2( 0.34, -0.26), vec2( 0.34, -0.10), 0.048);
    // Linha do nariz
    float nose_v = sdSegment(p, vec2(-0.04, -0.45), vec2(-0.08, 0.08), 0.028);
    float nose_h = sdSegment(p, vec2(-0.08,  0.08), vec2( 0.08, 0.06), 0.028);
    // Sorriso em arco parabólico
    float smile_y = 0.32 - 0.38 * p.x * p.x;
    float smile = abs(p.y - smile_y) - 0.032;
    smile = max(smile, abs(p.x) - 0.46);

    float features = min(min(eye_l, eye_r), min(min(nose_v, nose_h), smile));
    float feat_mask = 1.0 - smoothstep(-0.015, 0.015, features);
    rgb = mix(rgb, vec3(0.08, 0.14, 0.24), feat_mask);

    // Borda especular do Squircle
    float rim = 1.0 - smoothstep(0.0, 0.04, abs(d_squircle + 0.02));
    rgb += vec3(0.25) * rim * (1.0 - uv.y);
    return vec4(rgb, mask);
}

// 4. ÍCONE DO SAFARI (Bússola Azul Royal com Ponteiro Bicolor)
vec4 renderSafariDockIcon(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    vec2 q = abs(p) - vec2(0.62);
    float d_squircle = pow(pow(max(q.x, 0.0), 4.0) + pow(max(q.y, 0.0), 4.0), 0.25) + min(max(q.x, q.y), 0.0) - 0.26;
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_squircle);
    if (mask <= 0.001) return vec4(0.0);

    // Fundo azul safira profundo com radial gradient
    float dist = length(p);
    vec3 bg = mix(vec3(0.12, 0.58, 0.98), vec3(0.04, 0.28, 0.72), smoothstep(0.1, 0.85, dist));

    // Dial circular da bússola com marcas graduadas
    float dial_ring = abs(length(p) - 0.62) - 0.02;
    float dial_a = 1.0 - smoothstep(-0.01, 0.01, dial_ring);
    bg = mix(bg, vec3(1.0), dial_a * 0.4);

    // Agulha da bússola rotacionada a 45 graus (pi/4)
    float c = 0.70710678, s = 0.70710678;
    vec2 rp = mat2(c, -s, s, c) * p;

    // Losango da agulha (Norte Vermelho, Sul Branco)
    float needle_shape = abs(rp.x) * 3.8 + abs(rp.y) * 1.1 - 0.75;
    float needle_mask = 1.0 - smoothstep(-0.015, 0.015, needle_shape);

    if (needle_mask > 0.001) {
        vec3 needle_col = (rp.y < 0.0) ? vec3(0.95, 0.20, 0.18) : vec3(0.96, 0.97, 0.99);
        // Sombra suave no meio da agulha
        needle_col *= (rp.x > 0.0) ? 0.88 : 1.0;
        bg = mix(bg, needle_col, needle_mask);
    }

    // Pino central
    float pin = length(p) - 0.08;
    bg = mix(bg, vec3(0.95), 1.0 - smoothstep(-0.01, 0.01, pin));

    // Specular Rim
    float rim = 1.0 - smoothstep(0.0, 0.04, abs(d_squircle + 0.02));
    bg += vec3(0.25) * rim * (1.0 - uv.y);
    return vec4(bg, mask);
}

// 5. ÍCONE DO TERMINAL (Obsidiana Metálica com Segmento Vetorial >_)
vec4 renderTerminalDockIcon(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    vec2 q = abs(p) - vec2(0.62);
    float d_squircle = pow(pow(max(q.x, 0.0), 4.0) + pow(max(q.y, 0.0), 4.0), 0.25) + min(max(q.x, q.y), 0.0) - 0.26;
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_squircle);
    if (mask <= 0.001) return vec4(0.0);

    vec3 bg = mix(vec3(0.14, 0.15, 0.18), vec3(0.06, 0.07, 0.09), uv.y);

    // Barra de título sutil no topo do squircle
    float titlebar = sdRoundBox(p - vec2(0.0, -0.54), vec2(0.75, 0.14), 0.08);
    if (titlebar < 0.0) {
        bg = mix(bg, vec3(0.20, 0.22, 0.26), 0.4);
    }

    // Segmentos do prompt ">"
    float p1 = sdSegment(p, vec2(-0.48, -0.18), vec2(-0.24, 0.0), 0.038);
    float p2 = sdSegment(p, vec2(-0.24,  0.00), vec2(-0.48, 0.18), 0.038);
    float d_chevron = min(p1, p2);

    // Segmento do cursor "_"
    float d_cursor = sdSegment(p, vec2(-0.12, 0.18), vec2(0.18, 0.18), 0.038);
    float d_prompt = min(d_chevron, d_cursor);

    float prompt_mask = 1.0 - smoothstep(-0.015, 0.015, d_prompt);
    bg = mix(bg, vec3(0.25, 0.92, 0.45), prompt_mask);

    // Specular Rim
    float rim = 1.0 - smoothstep(0.0, 0.04, abs(d_squircle + 0.02));
    bg += vec3(0.3) * rim * (1.0 - uv.y);
    return vec4(bg, mask);
}

// 6. ÍCONE DE AJUSTES DO SISTEMA (3 Engrenagens Metálicas Concentricas)
vec4 renderSettingsDockIcon(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    vec2 q = abs(p) - vec2(0.62);
    float d_squircle = pow(pow(max(q.x, 0.0), 4.0) + pow(max(q.y, 0.0), 4.0), 0.25) + min(max(q.x, q.y), 0.0) - 0.26;
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_squircle);
    if (mask <= 0.001) return vec4(0.0);

    vec3 bg = mix(vec3(0.68, 0.70, 0.74), vec3(0.42, 0.44, 0.48), uv.y);

    // Engrenagem procedural com 8 dentes radiais
    float angle = atan(p.y, p.x);
    float radius = length(p);
    float teeth = sin(angle * 8.0) * 0.08;
    float gear_outer = radius - (0.50 + teeth);
    float gear_inner = radius - 0.22;
    float gear = max(gear_outer, -gear_inner);

    float gear_mask = 1.0 - smoothstep(-0.015, 0.015, gear);
    bg = mix(bg, vec3(0.85, 0.88, 0.92), gear_mask);

    // Centro da engrenagem
    float center_hole = length(p) - 0.10;
    bg = mix(bg, vec3(0.45, 0.48, 0.52), 1.0 - smoothstep(-0.01, 0.01, center_hole));

    // Specular Rim
    float rim = 1.0 - smoothstep(0.0, 0.04, abs(d_squircle + 0.02));
    bg += vec3(0.3) * rim * (1.0 - uv.y);
    return vec4(bg, mask);
}

// 7. ÍCONE DO LAUNCHPAD (Grade 3x3 de Mini-Squircles Coloridos)
vec4 renderLaunchpadDockIcon(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    vec2 q = abs(p) - vec2(0.62);
    float d_squircle = pow(pow(max(q.x, 0.0), 4.0) + pow(max(q.y, 0.0), 4.0), 0.25) + min(max(q.x, q.y), 0.0) - 0.26;
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_squircle);
    if (mask <= 0.001) return vec4(0.0);

    vec3 bg = mix(vec3(0.24, 0.26, 0.32), vec3(0.12, 0.14, 0.18), uv.y);

    // 9 mini-squircles
    for (int ix = -1; ix <= 1; ix++) {
        for (int iy = -1; iy <= 1; iy++) {
            vec2 center = vec2(float(ix) * 0.34, float(iy) * 0.34);
            float d_item = sdRoundBox(p - center, vec2(0.11), 0.04);
            float item_mask = 1.0 - smoothstep(-0.015, 0.015, d_item);
            // Cores vibrantes do arco-íris
            vec3 col = mix(vec3(0.95, 0.35, 0.25), vec3(0.20, 0.70, 0.95), float(ix + 1) / 2.0);
            col = mix(col, vec3(0.35, 0.85, 0.40), float(iy + 1) / 2.0);
            bg = mix(bg, col, item_mask);
        }
    }

    // Specular Rim
    float rim = 1.0 - smoothstep(0.0, 0.04, abs(d_squircle + 0.02));
    bg += vec3(0.3) * rim * (1.0 - uv.y);
    return vec4(bg, mask);
}

// 8. ÍCONE DA LIXEIRA (Cesta Metálica / Vidro com Nervuras)
vec4 renderTrashDockIcon(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    vec2 q = abs(p) - vec2(0.62);
    float d_squircle = pow(pow(max(q.x, 0.0), 4.0) + pow(max(q.y, 0.0), 4.0), 0.25) + min(max(q.x, q.y), 0.0) - 0.26;
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_squircle);
    if (mask <= 0.001) return vec4(0.0);

    vec3 bg = mix(vec3(0.28, 0.30, 0.36), vec3(0.16, 0.18, 0.22), uv.y);

    // Borda superior da cesta
    float rim_bar = sdRoundBox(p - vec2(0.0, -0.46), vec2(0.48, 0.06), 0.04);
    // Corpo trapezoidal da lixeira
    float body_y = clamp((p.y - (-0.40)) / 0.88, 0.0, 1.0);
    float half_w = mix(0.44, 0.34, body_y);
    float d_body = max(abs(p.x) - half_w, abs(p.y - 0.06) - 0.44);

    float basket = min(rim_bar, d_body);
    float b_mask = 1.0 - smoothstep(-0.015, 0.015, basket);
    bg = mix(bg, vec3(0.72, 0.75, 0.80), b_mask * 0.75);

    // Linhas verticais (nervuras metálicas)
    for (float lx = -0.28; lx <= 0.28; lx += 0.14) {
        float line_d = sdSegment(p, vec2(lx * 0.95, -0.38), vec2(lx * 0.75, 0.48), 0.014);
        bg = mix(bg, vec3(0.20, 0.22, 0.26), (1.0 - smoothstep(-0.01, 0.01, line_d)) * b_mask);
    }

    // Specular Rim
    float rim = 1.0 - smoothstep(0.0, 0.04, abs(d_squircle + 0.02));
    bg += vec3(0.3) * rim * (1.0 - uv.y);
    return vec4(bg, mask);
}

// 9. ÍCONES DE STATUS (Wi-Fi, Bateria, Lupa de Busca)
float sdWifiIcon(vec2 p) {
    p.y -= 0.35;
    float d = sdCircle(p, 0.12);
    for (int i = 1; i <= 3; i++) {
        float r = float(i) * 0.28;
        float ring = abs(length(p) - r) - 0.055;
        float cone = max(abs(p.x) * 1.15 + p.y, -p.y - 0.05);
        d = min(d, max(ring, cone));
    }
    return d;
}

float sdSearchLoupe(vec2 p) {
    float ring = abs(sdCircle(p - vec2(-0.12, -0.12), 0.42)) - 0.075;
    float handle = sdSegment(p, vec2(0.18, 0.18), vec2(0.65, 0.65), 0.085);
    return min(ring, handle);
}

float sdBatteryIcon(vec2 p, float charge_level, out float fill_mask) {
    float body = abs(sdRoundBox(p - vec2(-0.08, 0.0), vec2(0.68, 0.36), 0.12)) - 0.045;
    float nub  = sdRoundBox(p - vec2(0.70, 0.0), vec2(0.06, 0.16), 0.04);
    float w = 0.56 * clamp(charge_level, 0.05, 1.0);
    float inner = sdRoundBox(p - vec2(-0.08 - (0.56 - w), 0.0), vec2(w, 0.24), 0.06);
    fill_mask = 1.0 - smoothstep(-0.02, 0.02, inner);
    return min(body, nub);
}

void main() {
    vec2 p = v_local_pos;
    vec2 uv = v_uv;

    switch (u.icon_mode) {
        case 0: { // Apple Logo
            float d = sdAppleLogo(p);
            float a = 1.0 - smoothstep(-0.015, 0.015, d);
            vec3 col = (u.is_dark == 1) ? vec3(1.0) : vec3(0.1);
            frag_color = vec4(col, a);
            break;
        }
        case 1: { // Folder
            frag_color = renderMacOSFolder(uv);
            break;
        }
        case 2: { // Finder Dock Icon
            frag_color = renderFinderDockIcon(uv);
            break;
        }
        case 3: { // Safari Dock Icon
            frag_color = renderSafariDockIcon(uv);
            break;
        }
        case 4: { // Terminal Dock Icon
            frag_color = renderTerminalDockIcon(uv);
            break;
        }
        case 5: { // Settings Dock Icon
            frag_color = renderSettingsDockIcon(uv);
            break;
        }
        case 6: { // Launchpad Dock Icon
            frag_color = renderLaunchpadDockIcon(uv);
            break;
        }
        case 7: { // Trash Dock Icon
            frag_color = renderTrashDockIcon(uv);
            break;
        }
        case 8: { // Wi-Fi Icon
            float d = sdWifiIcon(p);
            float a = 1.0 - smoothstep(-0.02, 0.02, d);
            frag_color = vec4(vec3(1.0), a);
            break;
        }
        case 9: { // Battery Icon
            float fill_m = 0.0;
            float d = sdBatteryIcon(p, u.param_a, fill_m);
            float border_a = 1.0 - smoothstep(-0.02, 0.02, d);
            vec3 col = mix(vec3(1.0), vec3(0.28, 0.85, 0.45), fill_m);
            frag_color = vec4(col, max(border_a, fill_m));
            break;
        }
        case 10: { // Search Loupe
            float d = sdSearchLoupe(p);
            float a = 1.0 - smoothstep(-0.02, 0.02, d);
            frag_color = vec4(vec3(1.0), a);
            break;
        }
        default:
            frag_color = vec4(0.0);
    }
}
