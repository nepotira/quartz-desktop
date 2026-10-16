#version 450
// ============================================================================
// 🍏 macOS Tahoe 26 Procedural SDF UI Uber Shader (Zero External Fonts/Icons)
// Mathematical boolean geometry and vector rasterizer for all desktop icons
// ============================================================================

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec2 v_local_pos;
layout(location = 0) out vec4 frag_color;

layout(binding = 0) uniform sampler2D u_BackdropTex;

layout(push_constant) uniform UberUniforms {
    vec2 screen_size;  // Display physical resolution
    int icon_mode;     // 0: LiquidGlass, 1: Apple, 2: Folder, 3: Finder, 4: Safari, 5: Terminal, 6: Settings, 7: Launchpad, 8: Trash, 9: Wifi, 10: Battery, 11: Search
    float param_a;     // E.g. battery charge level or hover factor
    float param_b;     // Secondary parameter
    int is_dark;       // 1: Dark Mode, 0: Light Mode
} u;

// ============================================================================
// 1. LIQUID GLASS REAL À PROVA DE FALHAS (VOGEL DISK 64-TAPS, RAIO = 42px)
// Elimina 100% qualquer linha nítida do papel de parede atrás da Sidebar e Dock
// ============================================================================
vec3 sample_macos_liquid_glass(
    sampler2D backdrop_tex,
    vec2 screen_uv,
    vec2 screen_size,
    vec2 normal_2d,
    float edge_factor,
    vec4 tint
) {
    // 1. Refração ótica tipo lente curva nas bordas do Squircle (Padrão Tahoe 26)
    vec2 refracted_uv = screen_uv - normal_2d * edge_factor * (6.5 / screen_size);

    // 2. Amostragem espiral de Golden Angle (Vogel Disk de 64 amostras em raio de 42px)
    vec3 acc_color = vec3(0.0);
    float total_weight = 0.0;
    const int SAMPLES = 64;
    const float GOLDEN_ANGLE = 2.39996323;
    const float BLUR_RADIUS_PX = 42.0;

    // Rotação estocástica por pixel via Interleaved Gradient Noise (zero banding/aliasing)
    float noise = fract(52.9829189 * fract(0.06711056 * gl_FragCoord.x + 0.00583715 * gl_FragCoord.y));
    float theta_offset = noise * 6.2831853;

    for (int i = 0; i < SAMPLES; i++) {
        float r = sqrt((float(i) + 0.5) / float(SAMPLES)) * BLUR_RADIUS_PX;
        float theta = float(i) * GOLDEN_ANGLE + theta_offset;
        vec2 offset_uv = (vec2(cos(theta), sin(theta)) * r) / screen_size;

        float sigma_norm = r / (BLUR_RADIUS_PX * 0.6);
        float w = exp(-0.5 * sigma_norm * sigma_norm);
        vec2 sample_uv = clamp(refracted_uv + offset_uv, vec2(0.002), vec2(0.998));
        acc_color += texture(backdrop_tex, sample_uv).rgb * w;
        total_weight += w;
    }
    vec3 blurred = acc_color / max(total_weight, 0.0001);

    // 3. Boost de Saturação Luma Rec.709 (Apple Vibrancy = 1.90x)
    float luma = dot(blurred, vec3(0.2126, 0.7152, 0.0722));
    vec3 saturated = clamp(mix(vec3(luma), blurred, 1.90), 0.0, 1.0);

    // 4. Mistura translúcida + Dithering anti-banding
    vec3 comp = mix(saturated, tint.rgb, tint.a);
    comp += (noise - 0.5) * (2.0 / 255.0);
    return clamp(comp, 0.0, 1.0);
}

// ============================================================================
// PRIMITIVAS SDF DE ALTA PRECISÃO (SQUIRCLE G2 E SEGMENTOS)
// ============================================================================
float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

float sdSquircleG2(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - (b - r);
    vec2 m = max(q, 0.0);
    return pow(pow(m.x, 2.35) + pow(m.y, 2.35), 1.0 / 2.35) + min(max(q.x, q.y), 0.0) - r;
}

float sdSegment(vec2 p, vec2 a, vec2 b, float r) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

float sdCircle(vec2 p, float r) {
    return length(p) - r;
}

float sdRoundBox(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
}

// ============================================================================
// 2. PASTA AZUL OFICIAL DO macOS TAHOE 26 (COM ABA EM CURVA "S" CONTÍNUA)
// uv em [0.0, 1.0]^2
// ============================================================================
vec4 draw_exact_macos_folder(vec2 uv) {
    vec2 p = uv;
    float px = 1.0 / 80.0;
    vec4 final_col = vec4(0.0);

    // Sombra suave projetada abaixo da pasta
    vec2 sh_q = abs(p - vec2(0.50, 0.59)) - vec2(0.43, 0.28) + 0.08;
    float d_sh = min(max(sh_q.x, sh_q.y), 0.0) + length(max(sh_q, 0.0)) - 0.08;
    float sh_alpha = (1.0 - smoothstep(-0.02, 0.11, d_sh)) * 0.28;
    final_col = vec4(0.0, 0.0, 0.0, sh_alpha);

    // 1. Parte traseira com transição suave em curva S da aba esquerda (x=0.34..0.48)
    float tab_curve = smoothstep(0.34, 0.48, p.x);
    float top_back_y = mix(0.14, 0.22, tab_curve);

    vec2 back_min = vec2(0.06, top_back_y);
    vec2 back_max = vec2(0.94, 0.86);
    vec2 back_center = (back_min + back_max) * 0.5;
    vec2 back_half = (back_max - back_min) * 0.5;
    vec2 qb = abs(p - back_center) - back_half + 0.07;
    float d_back = min(max(qb.x, qb.y), 0.0) + length(max(qb, 0.0)) - 0.07;

    if (d_back < px) {
        float a_back = 1.0 - smoothstep(-px, px, d_back);
        vec3 back_rgb = mix(vec3(0.24, 0.65, 0.96), vec3(0.10, 0.44, 0.80), clamp((p.y - 0.14) / 0.22, 0.0, 1.0));
        // Borda especular fininha no topo da aba traseira
        float tab_rim = exp(-pow((p.y - top_back_y) / 0.010, 2.0)) * 0.35;
        back_rgb += vec3(tab_rim);
        final_col = mix(final_col, vec4(back_rgb, 1.0), a_back);
    }

    // 2. Capa frontal principal da pasta (y = 0.28 até y = 0.88)
    vec2 front_min = vec2(0.05, 0.28);
    vec2 front_max = vec2(0.95, 0.88);
    vec2 front_center = (front_min + front_max) * 0.5;
    vec2 front_half = (front_max - front_min) * 0.5;
    vec2 qf = abs(p - front_center) - front_half + 0.075;
    float d_front = min(max(qf.x, qf.y), 0.0) + length(max(qf, 0.0)) - 0.075;

    // Sombra de oclusão suave logo acima de y = 0.28 (sem faixa preta dura!)
    float pocket_ao = exp(-pow((p.y - 0.28) / 0.025, 2.0)) * step(p.y, 0.28) * step(d_back, 0.0);
    final_col.rgb *= (1.0 - pocket_ao * 0.22);

    if (d_front < px) {
        float a_front = 1.0 - smoothstep(-px, px, d_front);
        float t_y = clamp((p.y - 0.28) / 0.60, 0.0, 1.0);
        // Gradiente Azul Celeste Oficial Apple (#72D6FF -> #127DFA)
        vec3 front_rgb = mix(vec3(0.45, 0.84, 1.00), vec3(0.07, 0.49, 0.98), t_y);

        // Chanfro especular branco luminoso no lábio superior da capa frontal (y = 0.285)
        float top_highlight = exp(-pow((p.y - 0.286) / 0.011, 2.0));
        front_rgb = mix(front_rgb, vec3(0.90, 0.98, 1.00), top_highlight * 0.70);

        // Dois vincos horizontais sutis na base da pasta
        float groove1 = exp(-pow((p.y - 0.82) / 0.007, 2.0)) * 0.10;
        float groove2 = exp(-pow((p.y - 0.845) / 0.007, 2.0)) * 0.08;
        front_rgb -= vec3(groove1 + groove2);

        final_col = mix(final_col, vec4(front_rgb, 1.0), a_front);
    }
    return final_col;
}

// ============================================================================
// 3. ÍCONES 3D DE ALTA DEFINIÇÃO DO DOCK (PROIBIDO DESENHO "MS PAINT")
// ============================================================================

// 3.1. FINDER (Rosto 3D Bicolor com Curva Senoidal, Olhos em Pílula e Sorriso Curvo)
vec4 draw_dock_finder(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    float d_sq = sdSquircleG2(p, vec2(0.84), 0.36);
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_sq);
    if (mask <= 0.001) return vec4(0.0);

    // Divisão curva do nariz que vira no centro e segue até a base
    float nose_bend = smoothstep(-0.08, 0.08, p.y);
    float split_x = mix(-0.05 - 0.06 * p.y, 0.12 - 0.08 * (p.y - 0.08), nose_bend);
    float side = smoothstep(split_x - 0.02, split_x + 0.02, p.x);

    vec3 left_col  = mix(vec3(0.22, 0.72, 1.00), vec3(0.06, 0.44, 0.90), uv.y);
    vec3 right_col = mix(vec3(0.97, 0.99, 1.00), vec3(0.82, 0.90, 0.97), uv.y);
    vec3 rgb = mix(left_col, right_col, side);

    // Traço contínuo do nariz e divisão central
    float d_nose_top = sdSegment(p, vec2(-0.02, -0.84), vec2(-0.08, 0.04), 0.032);
    float d_nose_mid = sdSegment(p, vec2(-0.08,  0.04), vec2( 0.12, 0.01), 0.032);
    float d_nose_bot = sdSegment(p, vec2( 0.09,  0.34), vec2( 0.05, 0.84), 0.030);
    float d_nose = min(min(d_nose_top, d_nose_mid), d_nose_bot);

    // Olhos verticais em pílula
    float d_eye_l = sdSegment(p, vec2(-0.36, -0.28), vec2(-0.36, -0.11), 0.052);
    float d_eye_r = sdSegment(p, vec2( 0.34, -0.28), vec2( 0.34, -0.11), 0.052);

    // Sorriso curvo largo com cantos afunilados
    float smile_arc_y = 0.24 + 0.18 * (1.0 - p.x * p.x * 1.9);
    float d_smile = abs(p.y - smile_arc_y) - 0.034 * (1.0 - 0.4 * p.x * p.x);
    d_smile = max(d_smile, abs(p.x) - 0.48);

    float d_features = min(min(d_eye_l, d_eye_r), min(d_nose, d_smile));
    rgb = mix(rgb, vec3(0.08, 0.15, 0.26), 1.0 - smoothstep(-0.015, 0.015, d_features));

    // Sombra interna 3D na base e borda especular no topo do Squircle
    float inner_bevel = smoothstep(-0.12, 0.0, d_sq);
    rgb = mix(rgb, rgb * 0.78, inner_bevel * uv.y);
    rgb += vec3(0.35) * (1.0 - smoothstep(0.0, 0.04, abs(d_sq + 0.02))) * (1.0 - uv.y);
    return vec4(rgb, mask);
}

// 3.2. SAFARI (Bússola Real com 36 Marcações Polares e Agulha Bipartida 3D)
vec4 draw_dock_safari(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    float d_sq = sdSquircleG2(p, vec2(0.84), 0.36);
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_sq);
    if (mask <= 0.001) return vec4(0.0);

    vec3 bg = vec3(0.96, 0.97, 0.99);
    float r = length(p);
    float d_dial = r - 0.72;
    vec3 dial_col = mix(vec3(0.16, 0.74, 0.99), vec3(0.04, 0.42, 0.92), uv.y);

    // 36 Marcações polares de graus ao redor da bússola
    float angle = atan(p.y, p.x);
    float ticks = abs(sin(angle * 18.0));
    float tick_ring = step(0.58, r) * step(r, 0.67) * smoothstep(0.18, 0.05, ticks);
    dial_col = mix(dial_col, vec3(0.95, 0.98, 1.0), tick_ring * 0.85);

    // Agulha inclinada a -45 graus (Vermelha NE / Branca SW) com facetamento 3D
    float c = 0.70710678, s = 0.70710678;
    vec2 np = mat2(c, -s, s, c) * p;
    float needle_shape = abs(np.y) + abs(np.x) * 0.16 - 0.095;
    float needle_shadow = abs(np.y - 0.03) + abs(np.x) * 0.16 - 0.095;

    // Sombra da agulha sobre o mostrador azul
    dial_col *= (1.0 - (1.0 - smoothstep(-0.02, 0.04, needle_shadow)) * 0.35);

    if (needle_shape < 0.02) {
        float nm = 1.0 - smoothstep(-0.015, 0.015, needle_shape);
        vec3 n_col = (np.x > 0.0) ? vec3(1.00, 0.23, 0.19) : vec3(0.98, 0.98, 1.00);
        // Sombreamento longitudinal de metade da agulha (efeito 3D piramidal)
        if (np.y > 0.0) n_col *= 0.84;
        dial_col = mix(dial_col, n_col, nm);
    }

    vec3 rgb = mix(bg, dial_col, 1.0 - smoothstep(-0.015, 0.015, d_dial));
    return vec4(rgb, mask);
}

// 3.3. LIXEIRA DO macOS (Cilindro Metálico 3D Prateado — Proibido Quadrado Cinza!)
vec4 draw_dock_trash(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    // Corpo trapezoidal com base curva e boca elíptica no topo
    float top_w = 0.56, bot_w = 0.42;
    float y_norm = clamp((p.y - (-0.52)) / 1.18, 0.0, 1.0);
    float half_w = mix(top_w, bot_w, y_norm);

    float d_side = abs(p.x) - half_w;
    float bottom_curve = 0.08 * (1.0 - pow(clamp(p.x / bot_w, -1.0, 1.0), 2.0));
    float d_vert = max(-p.y - 0.52, p.y - (0.64 + bottom_curve));
    float d_body = max(d_side, d_vert);

    // Aro elíptico superior da lixeira
    vec2 rp = (p - vec2(0.0, -0.52)) / vec2(top_w + 0.03, 0.11);
    float d_rim = length(rp) - 1.0;

    float d_all = min(d_body, d_rim);
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_all);
    if (mask <= 0.001) return vec4(0.0);

    // Iluminação Cilíndrica Anisotrópica (Alumínio Anodizado Apple)
    float nx = clamp(p.x / max(half_w, 0.01), -1.0, 1.0);
    float specular = exp(-pow((nx + 0.28) * 2.6, 2.0)) * 0.35;
    vec3 metal = mix(vec3(0.58, 0.60, 0.65), vec3(0.86, 0.88, 0.92), 0.5 - 0.4 * abs(nx - 0.1));
    metal += vec3(specular);
    metal *= mix(1.0, 0.82, y_norm);

    // Caneluras/Fendas verticais sutis na malha metálica
    float grooves = sin(nx * 22.0);
    float in_groove = step(0.15, y_norm) * step(y_norm, 0.88) * step(abs(nx), 0.82);
    metal *= (1.0 - 0.12 * smoothstep(0.4, 0.9, grooves) * in_groove);

    // Interior escuro da boca elíptica no topo
    vec2 ip = (p - vec2(0.0, -0.52)) / vec2(top_w - 0.04, 0.075);
    float d_hole = length(ip) - 1.0;
    if (d_hole < 0.02) {
        vec3 inner_col = mix(vec3(0.18, 0.20, 0.23), vec3(0.38, 0.40, 0.45), (p.y + 0.60) * 6.0);
        metal = mix(metal, inner_col, 1.0 - smoothstep(-0.02, 0.02, d_hole));
    }
    return vec4(metal, mask);
}

// 3.4. AJUSTES DO SISTEMA (Engrenagem Metálica 3D com Chanfro Cônico)
vec4 draw_dock_settings(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    float d_sq = sdSquircleG2(p, vec2(0.84), 0.36);
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_sq);
    if (mask <= 0.001) return vec4(0.0);

    // Fundo metálico escovado escuro
    vec3 bg = mix(vec3(0.55, 0.57, 0.62), vec3(0.32, 0.34, 0.38), uv.y);
    float r = length(p);
    float a = atan(p.y, p.x);

    // Engrenagem externa de 12 dentes suavizados + 3 raios internos
    float outer_gear = r - (0.54 + 0.065 * smoothstep(-0.4, 0.4, cos(a * 12.0)));
    float inner_ring = abs(r - 0.34) - 0.065;
    float spokes = min(abs(p.x), min(abs(p.x * 0.5 + p.y * 0.866), abs(p.x * 0.5 - p.y * 0.866))) - 0.055;
    spokes = max(spokes, r - 0.34);
    float center_hub = abs(r - 0.12) - 0.045;

    float d_gear = max(outer_gear, -(r - 0.42));
    d_gear = min(d_gear, min(inner_ring, min(spokes, center_hub)));

    // Sombra projetada da engrenagem
    bg *= (1.0 - (1.0 - smoothstep(-0.01, 0.06, d_gear)) * 0.40);

    // Brilho metálico cônico na engrenagem
    vec3 gear_col = mix(vec3(0.74, 0.76, 0.80), vec3(0.95, 0.96, 0.98), 0.5 + 0.5 * cos(a * 2.0 - 1.2));
    vec3 rgb = mix(bg, gear_col, 1.0 - smoothstep(-0.015, 0.015, d_gear));
    return vec4(rgb, mask);
}

// 3.5. TERMINAL (Obsidiana Metálica com Segmento Vetorial >_)
vec4 draw_dock_terminal(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    float d_sq = sdSquircleG2(p, vec2(0.84), 0.36);
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_sq);
    if (mask <= 0.001) return vec4(0.0);

    vec3 bg = mix(vec3(0.14, 0.15, 0.18), vec3(0.06, 0.07, 0.09), uv.y);

    float p1 = sdSegment(p, vec2(-0.48, -0.18), vec2(-0.24, 0.0), 0.038);
    float p2 = sdSegment(p, vec2(-0.24,  0.00), vec2(-0.48, 0.18), 0.038);
    float d_chevron = min(p1, p2);
    float d_cursor = sdSegment(p, vec2(-0.12, 0.18), vec2(0.18, 0.18), 0.038);
    float d_prompt = min(d_chevron, d_cursor);

    float prompt_mask = 1.0 - smoothstep(-0.015, 0.015, d_prompt);
    bg = mix(bg, vec3(0.25, 0.92, 0.45), prompt_mask);

    float rim = 1.0 - smoothstep(0.0, 0.04, abs(d_sq + 0.02));
    bg += vec3(0.3) * rim * (1.0 - uv.y);
    return vec4(bg, mask);
}

// 3.6. LAUNCHPAD (Grade 3x3 de Mini-Squircles Coloridos)
vec4 draw_dock_launchpad(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    float d_sq = sdSquircleG2(p, vec2(0.84), 0.36);
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_sq);
    if (mask <= 0.001) return vec4(0.0);

    vec3 bg = mix(vec3(0.24, 0.26, 0.32), vec3(0.12, 0.14, 0.18), uv.y);

    for (int ix = -1; ix <= 1; ix++) {
        for (int iy = -1; iy <= 1; iy++) {
            vec2 center = vec2(float(ix) * 0.34, float(iy) * 0.34);
            float d_item = sdRoundBox(p - center, vec2(0.11), 0.04);
            float item_mask = 1.0 - smoothstep(-0.015, 0.015, d_item);
            vec3 col = mix(vec3(0.95, 0.35, 0.25), vec3(0.20, 0.70, 0.95), float(ix + 1) / 2.0);
            col = mix(col, vec3(0.35, 0.85, 0.40), float(iy + 1) / 2.0);
            bg = mix(bg, col, item_mask);
        }
    }

    float rim = 1.0 - smoothstep(0.0, 0.04, abs(d_sq + 0.02));
    bg += vec3(0.3) * rim * (1.0 - uv.y);
    return vec4(bg, mask);
}

// 3.7. ACTIVITY MONITOR (Tela Obsidiana com Pulso ECG Luminoso #30D158)
vec4 draw_dock_activity_monitor(vec2 uv) {
    vec2 p = uv * 2.0 - 1.0;
    float d_sq = sdSquircleG2(p, vec2(0.84), 0.36);
    float mask = 1.0 - smoothstep(-0.02, 0.02, d_sq);
    if (mask <= 0.001) return vec4(0.0);

    vec3 bg = mix(vec3(0.11, 0.13, 0.17), vec3(0.05, 0.06, 0.09), uv.y);
    float d_ecg = min(sdSegment(p, vec2(-0.68, 0.10), vec2(-0.28, 0.10), 0.036),
                  min(sdSegment(p, vec2(-0.28, 0.10), vec2(-0.10, -0.46), 0.036),
                  min(sdSegment(p, vec2(-0.10, -0.46), vec2(0.08, 0.48), 0.036),
                  min(sdSegment(p, vec2(0.08, 0.48), vec2(0.24, -0.18), 0.036),
                  min(sdSegment(p, vec2(0.24, -0.18), vec2(0.38, 0.10), 0.036),
                      sdSegment(p, vec2(0.38, 0.10), vec2(0.68, 0.10), 0.036))))));
    float glow = exp(-pow(max(d_ecg, 0.0) / 0.12, 2.0)) * 0.35;
    bg += vec3(0.10, 0.85, 0.45) * glow;
    bg = mix(bg, vec3(0.22, 0.96, 0.52), 1.0 - smoothstep(-0.015, 0.015, d_ecg));
    return vec4(clamp(bg, 0.0, 1.0), mask);
}

// 4. ÍCONES DE STATUS (Apple Logo 5.1, Wi-Fi, Bateria, Lupa)
float draw_apple_logo_sdf(vec2 uv) {
    vec2 p = (uv - vec2(0.5, 0.54)) * 2.25;
    p.y = -p.y;
    vec2 q = vec2(abs(p.x), p.y);
    float upper_lobe = length((q - vec2(0.23, 0.04)) * vec2(1.0, 0.92)) - 0.39;
    float lower_lobe = length((q - vec2(0.18, -0.22)) * vec2(1.05, 0.85)) - 0.31;
    float body = smin(upper_lobe, lower_lobe, 0.14);
    float top_dip = length(p - vec2(0.0, 0.38)) - 0.15;
    float bot_dip = length(p - vec2(0.0, -0.54)) - 0.14;
    body = max(body, -top_dip);
    body = max(body, -bot_dip);
    float bite = length(p - vec2(0.58, 0.05)) - 0.26;
    body = max(body, -bite);
    vec2 lp = p - vec2(0.08, 0.54);
    float c = cos(0.733), s = sin(0.733);
    lp = mat2(c, -s, s, c) * lp;
    float leaf = max(length(lp - vec2(0.0, -0.18)) - 0.29, length(lp - vec2(0.0, 0.18)) - 0.29);
    return min(body, leaf);
}

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
        case 0: { // Liquid Glass sample
            vec3 glass = sample_macos_liquid_glass(u_BackdropTex, uv, u.screen_size, vec2(0.0, 1.0), 0.5, vec4(0.14, 0.14, 0.17, 0.58));
            frag_color = vec4(glass, 1.0);
            break;
        }
        case 1: { // Apple Logo (Pillar 5.1)
            float d = draw_apple_logo_sdf(uv);
            float a = 1.0 - smoothstep(-0.015, 0.015, d);
            vec3 col = (u.is_dark == 1) ? vec3(1.0) : vec3(0.1);
            frag_color = vec4(col, a);
            break;
        }
        case 2: { // Exact Apple Folder
            frag_color = draw_exact_macos_folder(uv);
            break;
        }
        case 3: { // Finder 3D Dock Icon
            frag_color = draw_dock_finder(uv);
            break;
        }
        case 4: { // Safari 3D Dock Icon
            frag_color = draw_dock_safari(uv);
            break;
        }
        case 5: { // Terminal 3D Dock Icon
            frag_color = draw_dock_terminal(uv);
            break;
        }
        case 6: { // Settings 3D Dock Icon
            frag_color = draw_dock_settings(uv);
            break;
        }
        case 7: { // Launchpad 3D Dock Icon
            frag_color = draw_dock_launchpad(uv);
            break;
        }
        case 8: { // Trash 3D Dock Icon
            frag_color = draw_dock_trash(uv);
            break;
        }
        case 9: { // Wi-Fi Icon
            float d = sdWifiIcon(p);
            float a = 1.0 - smoothstep(-0.02, 0.02, d);
            frag_color = vec4(vec3(1.0), a);
            break;
        }
        case 10: { // Battery Icon (Pure White #FFFFFF fill, no neon green)
            float fill_m = 0.0;
            float d = sdBatteryIcon(p, u.param_a, fill_m);
            float border_a = 1.0 - smoothstep(-0.02, 0.02, d);
            vec3 col = vec3(1.0);
            frag_color = vec4(col, max(border_a * 0.45, fill_m));
            break;
        }
        case 11: { // Search Loupe
            float d = sdSearchLoupe(p);
            float a = 1.0 - smoothstep(-0.02, 0.02, d);
            frag_color = vec4(vec3(1.0), a);
            break;
        }
        case 12: { // Activity Monitor 3D Dock Icon
            frag_color = draw_dock_activity_monitor(uv);
            break;
        }
        default:
            frag_color = vec4(0.0);
    }
}
