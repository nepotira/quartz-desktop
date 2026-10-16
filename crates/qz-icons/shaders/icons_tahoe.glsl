#version 450
// ============================================================================
// qz-icons: Biblioteca Oficial de Ícones Procedurais SDF/Bézier (macOS Tahoe 26)
// Lei 1 Anti-Fraude: ZERO fontes de ícones, ZERO Unicode privado, ZERO emojis.
// ============================================================================

// ============================================================================
// 5.1. LOGOTIPO DA MAÇÃ OFICIAL (SILHUETA BOOLEANA SUAVE + MORDIDA + FOLHA 42°)
// ============================================================================
float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

float sdSegment(vec2 p, vec2 a, vec2 b, float r) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

float sdSquircleG2(vec2 p, vec2 half_size, float r) {
    vec2 q = abs(p) - (half_size - r);
    vec2 m = max(q, 0.0);
    float lp = pow(pow(m.x, 2.35) + pow(m.y, 2.35), 1.0 / 2.35);
    return lp + min(max(q.x, q.y), 0.0) - r;
}

float sdRoundBox(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
}

float draw_apple_logo_sdf(vec2 uv) {
    // uv em [0.0, 1.0]^2
    vec2 p = (uv - vec2(0.5, 0.54)) * 2.25;
    p.y = -p.y; // +Y para cima
    vec2 q = vec2(abs(p.x), p.y);

    // Lóbulos cardióides superiores e inferiores unidos com smin
    float upper_lobe = length((q - vec2(0.23, 0.04)) * vec2(1.0, 0.92)) - 0.39;
    float lower_lobe = length((q - vec2(0.18, -0.22)) * vec2(1.05, 0.85)) - 0.31;
    float body = smin(upper_lobe, lower_lobe, 0.14);

    // Cavidade suave no topo e na base entre os dois lóbulos
    float top_dip = length(p - vec2(0.0, 0.38)) - 0.15;
    float bot_dip = length(p - vec2(0.0, -0.54)) - 0.14;
    body = max(body, -top_dip);
    body = max(body, -bot_dip);

    // Subtração circular da mordida lateral direita
    float bite = length(p - vec2(0.58, 0.05)) - 0.26;
    body = max(body, -bite);

    // Folha superior inclinada a 42 graus (interseção de 2 círculos)
    vec2 lp = p - vec2(0.08, 0.54);
    float c = cos(0.733), s = sin(0.733);
    lp = mat2(c, -s, s, c) * lp;
    float leaf = max(length(lp - vec2(0.0, -0.18)) - 0.29, length(lp - vec2(0.0, 0.18)) - 0.29);

    return min(body, leaf);
}

// ============================================================================
// 5.2. PASTA AZUL OFICIAL DO macOS TAHOE 26 (CURVA S + 3-STOP GRADIENT + INSÍGNIA)
// folder_glyph_id: 0=Normal, 1=Applications, 2=Documents, 3=Downloads, 4=Desktop, 5=Developer
// ============================================================================
vec4 draw_macos_folder_tahoe(vec2 uv, int folder_glyph_id) {
    vec2 p = uv;
    float px = 1.0 / 84.0;
    vec4 col = vec4(0.0);

    // Sombra suave da pasta
    vec2 sq = abs(p - vec2(0.50, 0.60)) - vec2(0.42, 0.27) + 0.08;
    float d_sh = min(max(sq.x, sq.y), 0.0) + length(max(sq, 0.0)) - 0.08;
    col = vec4(0.0, 0.0, 0.0, (1.0 - smoothstep(-0.01, 0.10, d_sh)) * 0.28);

    // 1. Aba traseira com transição em curva S contínua (smoothstep(0.32, 0.46, uv.x))
    float s_curve = smoothstep(0.32, 0.46, p.x);
    float top_y = mix(0.14, 0.22, s_curve);
    vec2 b_min = vec2(0.06, top_y), b_max = vec2(0.94, 0.86);
    vec2 qb = abs(p - (b_min + b_max) * 0.5) - (b_max - b_min) * 0.5 + 0.075;
    float d_back = min(max(qb.x, qb.y), 0.0) + length(max(qb, 0.0)) - 0.075;

    if (d_back < px) {
        float a_b = 1.0 - smoothstep(-px, px, d_back);
        // Gradiente #2CA2F8 -> #1472D4
        vec3 b_rgb = mix(vec3(0.173, 0.635, 0.973), vec3(0.078, 0.447, 0.831), clamp((p.y - 0.14) / 0.24, 0.0, 1.0));
        b_rgb += vec3(0.30) * exp(-pow((p.y - top_y) / 0.010, 2.0));
        col = mix(col, vec4(b_rgb, 1.0), a_b);
    }

    // Sombra interna suave projetada na junção entre aba traseira e capa frontal
    float junction_ao = exp(-pow((p.y - 0.27) / 0.024, 2.0)) * step(p.y, 0.27) * step(d_back, 0.0);
    col.rgb *= (1.0 - junction_ao * 0.24);

    // 2. Capa frontal (y = 0.27..0.88, R = 0.085)
    vec2 f_min = vec2(0.05, 0.27), f_max = vec2(0.95, 0.88);
    vec2 qf = abs(p - (f_min + f_max) * 0.5) - (f_max - f_min) * 0.5 + 0.085;
    float d_front = min(max(qf.x, qf.y), 0.0) + length(max(qf, 0.0)) - 0.085;

    if (d_front < px) {
        float a_f = 1.0 - smoothstep(-px, px, d_front);
        float ty = clamp((p.y - 0.27) / 0.61, 0.0, 1.0);
        // Gradiente vertical de 3 paradas: #78DAFF -> #36A8FF -> #0B7CFF
        vec3 c1 = vec3(0.471, 0.855, 1.000);
        vec3 c2 = vec3(0.212, 0.659, 1.000);
        vec3 c3 = vec3(0.043, 0.486, 1.000);
        vec3 f_rgb = (ty < 0.5) ? mix(c1, c2, ty * 2.0) : mix(c2, c3, (ty - 0.5) * 2.0);

        // Crista especular branca/ciano de 1.0px em y = 0.275
        float crest = exp(-pow((p.y - 0.275) / 0.011, 2.0));
        f_rgb = mix(f_rgb, vec3(0.92, 0.98, 1.00), crest * 0.72);

        // Dois vincos horizontais sutis na base (y = 0.82 e y = 0.85)
        float g1 = exp(-pow((p.y - 0.82) / 0.006, 2.0)) * 0.11;
        float g2 = exp(-pow((p.y - 0.85) / 0.006, 2.0)) * 0.09;
        f_rgb -= vec3(g1 + g2);

        // Insígnia central em baixo-relevo (Embossed Glyph)
        if (folder_glyph_id > 0) {
            vec2 gp = (p - vec2(0.50, 0.575)) * 4.2;
            float d_glyph = 1.0;
            if (folder_glyph_id == 1) { // Applications ('A' estilizado)
                d_glyph = min(sdSegment(gp, vec2(0.0, -0.45), vec2(-0.38, 0.42), 0.07),
                          min(sdSegment(gp, vec2(0.0, -0.45), vec2( 0.38, 0.42), 0.07),
                              sdSegment(gp, vec2(-0.28, 0.18), vec2( 0.28, 0.18), 0.07)));
            } else if (folder_glyph_id == 2) { // Documents (Página com linhas)
                float doc_box = abs(sdRoundBox(gp, vec2(0.32, 0.42), 0.06)) - 0.04;
                float l1 = sdSegment(gp, vec2(-0.18, -0.14), vec2(0.18, -0.14), 0.04);
                float l2 = sdSegment(gp, vec2(-0.18,  0.04), vec2(0.18,  0.04), 0.04);
                float l3 = sdSegment(gp, vec2(-0.18,  0.22), vec2(0.10,  0.22), 0.04);
                d_glyph = min(doc_box, min(l1, min(l2, l3)));
            } else if (folder_glyph_id == 3) { // Downloads (Seta em círculo)
                float ring = abs(length(gp) - 0.48) - 0.06;
                float arr  = min(sdSegment(gp, vec2(0.0, -0.25), vec2(0.0, 0.22), 0.07),
                             min(sdSegment(gp, vec2(-0.18, 0.05), vec2(0.0, 0.24), 0.07),
                                 sdSegment(gp, vec2( 0.18, 0.05), vec2(0.0, 0.24), 0.07)));
                d_glyph = min(ring, arr);
            } else if (folder_glyph_id == 4) { // Desktop (Janela com barra superior e dock)
                float desk_box = abs(sdRoundBox(gp, vec2(0.44, 0.32), 0.07)) - 0.045;
                float dock_bar = sdRoundBox(gp - vec2(0.0, 0.18), vec2(0.24, 0.04), 0.02);
                d_glyph = min(desk_box, dock_bar);
            } else if (folder_glyph_id == 5) { // Developer (Martelo/Chave '< / >')
                d_glyph = min(sdSegment(gp, vec2(-0.22, -0.25), vec2(-0.45, 0.0), 0.065),
                          min(sdSegment(gp, vec2(-0.45,  0.00), vec2(-0.22, 0.25), 0.065),
                          min(sdSegment(gp, vec2( 0.22, -0.25), vec2( 0.45, 0.0), 0.065),
                          min(sdSegment(gp, vec2( 0.45,  0.00), vec2( 0.22, 0.25), 0.065),
                              sdSegment(gp, vec2( 0.08, -0.32), vec2(-0.08, 0.32), 0.065)))));
            }
            float gm = 1.0 - smoothstep(-0.04, 0.04, d_glyph);
            f_rgb = mix(f_rgb, f_rgb * vec3(0.55, 0.72, 0.92), gm * 0.65);
        }

        col = mix(col, vec4(f_rgb, 1.0), a_f);
    }
    return col;
}
