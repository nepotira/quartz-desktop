//! # Catálogo Oficial das 18 Animações do macOS Tahoe 26 (`qz-animation::catalog`)
//!
//! Implementa matematicamente todas as 18 animações obrigatórias do Pilar 4 a 120Hz:
//! 1. [`WindowOpenAnimation`]: Escala `0.92 -> 1.00`, Opacidade `0.0 -> 1.0` ($m=1, k=450, c=38$).
//! 2. [`WindowCloseAnimation`]: Escala `1.00 -> 0.95`, Opacidade `1.0 -> 0.0` em $140\text{ms}$ sem overshoot.
//! 3. [`QuickLookMorphAnimation`]: Morph geométrico contínuo $(x_i, y_i, w_i, h_i) \leftrightarrow \text{centro}$ ($k=420, c=36$).
//! 4. [`GenieWarpTimer`]: $380\text{ms}$ normal; $1900\text{ms}$ ($\Delta t \times 0.2$) com `Shift` pressionado.
//! 5. [`MenuConfirmationFlash`]: Destaque `#0A60FF` pisca 2x (`Off 35ms -> On 35ms -> Off 35ms -> On 35ms`) + fade $110\text{ms}$.
//! 6. [`DockEntrySpring`]: Amortecimento de entrada/saída $\alpha_{\text{entry}} \in [0, 1]$ ($k=500, c=45$).
//! 7. [`ParabolicDockBounce`]: Gravidade $g = 980\text{px/s}^2$, altura `28px`, restituição $e = 0.62$.
//! 8. [`DockStackFanAnimation`]: Mola ($k=380, c=28$) em arco curvo $\theta_k = k \cdot 4.2^\circ$, $\Delta x_k = k^2 \cdot 3.5\text{px}$.
//! 9. [`GestureSwipeTransition`]: Acoplamento 1:1 com injeção de velocidade $(\dot{x}, \dot{y})$ ($m=1, k=320, c=35.777, \zeta=1.0$).
//! 10. [`StageManager3DFlip`]: Perspectiva $\text{FOV} = 45^\circ$, rotação Y $0^\circ \to 14^\circ$ ($m=1, k=280, c=31$).
//! 11. [`ModalSheetUnfurl`]: Rotação 3D eixo X ($-8^\circ \to 0^\circ$) e sombra progressiva ($220\text{ms}$).
//! 12. [`IncorrectPasswordShake`]: $\Delta x(t) = 14.0 \cdot e^{-8t} \cdot \sin(10\pi t)$ durante $420\text{ms}$.
//! 13. [`DockPoofEffect`]: Arraste $< 100\text{px}$ retorna por mola ($k=400, c=26$); $\ge 100\text{px}$ gera 5 esferas de fumaça ($160\text{ms}$).
//! 14. [`JiggleModeOscillator`]: $\theta(t) = 1.8^\circ \cdot \sin(28t + \phi_i)$ e $\Delta y(t) = 1.0\text{px} \cdot \cos(24t + \phi_i)$.
//! 15. [`super::CursorShakeDetector`]: $\ge 3$ inversões em $400\text{ms}$ ampliam cursor $1\times \to 8\times$, retorno após $1200\text{ms}$.
//! 16. [`LinearRubberBand`]: Resistência $\Delta y_{\text{vis}} = \Delta y_{\text{raw}} \cdot 0.35$ além dos limites e retorno por mola.
//! 17. [`DownloadFlyToDock`]: Voo em Bézier quadrática até a Stack de Downloads com micro-salto da pilha.
//! 18. [`AerialWallpaperDecelerator`]: Desaceleração exponencial $v(t) = v_0 e^{-3.5t}$ a 120fps.

use crate::{Spring2D, SpringAnimation, SpringParams, SpringRect};
use std::f64::consts::PI;

///Multiplicador de câmera lenta universal quando `Shift` está pressionado ($\Delta t_{\text{eff}} = 0.2 \cdot \Delta t$).
pub const SHIFT_SLOW_MOTION_FACTOR: f64 = 0.2;

/// 1. Abertura de Janela / Popover / Spotlight (`0.92 -> 1.00`, Opacidade `0.0 -> 1.0`, $m=1, k=450, c=38$).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowOpenAnimation {
    scale_spring: SpringAnimation,
    elapsed: f64,
}

impl Default for WindowOpenAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowOpenAnimation {
    /// Duração nominal de convergência visual ($185\text{ms}$).
    pub const NOMINAL_DURATION: f64 = 0.185;

    /// Inicia a animação de abertura em escala `0.92` e opacidade `0.0`.
    pub fn new() -> Self {
        let params = SpringParams {
            mass: 1.0,
            stiffness: 450.0,
            damping: 38.0,
        };
        Self {
            scale_spring: SpringAnimation::new(params, 0.92, 0.0, 1.00),
            elapsed: 0.0,
        }
    }

    /// Avança a animação por `dt` segundos (suporta `shift_slow_motion`) e retorna `(scale, opacity)`.
    pub fn step(&mut self, dt: f64, shift_slow_motion: bool) -> (f64, f64) {
        let eff_dt = if shift_slow_motion {
            dt.max(0.0) * SHIFT_SLOW_MOTION_FACTOR
        } else {
            dt.max(0.0)
        };
        self.elapsed += eff_dt;
        let scale = self.scale_spring.step(eff_dt).position;
        let opacity = (self.elapsed / Self::NOMINAL_DURATION).clamp(0.0, 1.0);
        (scale, opacity)
    }
}

/// 2. Fechamento de Janela (`Cmd+W`): Escala `1.00 -> 0.95`, Opacidade `1.0 -> 0.0` em $140\text{ms}$ sem overshoot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowCloseAnimation {
    elapsed: f64,
}

impl Default for WindowCloseAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowCloseAnimation {
    /// Duração total de fechamento ($140\text{ms}$).
    pub const DURATION: f64 = 0.140;

    /// Cria uma nova animação de fechamento de janela.
    pub fn new() -> Self {
        Self { elapsed: 0.0 }
    }

    /// Avança por `dt` segundos e retorna `(scale, opacity)` sem qualquer overshoot.
    pub fn step(&mut self, dt: f64) -> (f64, f64) {
        self.elapsed = (self.elapsed + dt.max(0.0)).min(Self::DURATION);
        let u = self.elapsed / Self::DURATION;
        let s = u * u * (3.0 - 2.0 * u); // Hermite monotone C1
        let scale = 1.00 - 0.05 * s;
        let opacity = 1.0 - s;
        (scale, opacity)
    }

    /// Verdadeiro quando o fechamento de $140\text{ms}$ terminou.
    pub fn is_finished(&self) -> bool {
        self.elapsed >= Self::DURATION
    }
}

/// 3. Quick Look Morphing (`Espaço`): Morph geométrico contínuo do ícone até o centro da tela ($k=420, c=36$).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuickLookMorphAnimation {
    rect_spring: SpringRect,
}

impl QuickLookMorphAnimation {
    /// Cria o morph contínuo entre o retângulo do ícone `(x, y, w, h)` e o retângulo central de preview.
    pub fn new(icon_rect: (f64, f64, f64, f64), preview_rect: (f64, f64, f64, f64)) -> Self {
        let params = SpringParams {
            mass: 1.0,
            stiffness: 420.0,
            damping: 36.0,
        };
        Self {
            rect_spring: SpringRect::new(params, icon_rect, (0.0, 0.0, 0.0, 0.0), preview_rect),
        }
    }

    /// Redireciona o morph de volta para o retângulo atual do ícone no Finder ao fechar.
    pub fn close_to_icon(&mut self, current_icon_rect: (f64, f64, f64, f64)) {
        self.rect_spring.retarget(current_icon_rect);
    }

    /// Avança por `dt` segundos e retorna o retângulo interpolado `(x, y, w, h)`.
    pub fn step(&mut self, dt: f64) -> (f64, f64, f64, f64) {
        self.rect_spring.step(dt)
    }
}

/// 4. Temporizador do Efeito Gênio ($380\text{ms}$ normal; $1900\text{ms}$ com `Shift` segurado).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenieWarpTimer {
    progress: f64,
    minimizing: bool,
}

impl GenieWarpTimer {
    /// Duração normal do Efeito Gênio ($380\text{ms}$).
    pub const DURATION_NORMAL: f64 = 0.380;
    /// Duração em câmera lenta com `Shift` ($1900\text{ms}$).
    pub const DURATION_SHIFT_SLOW: f64 = 1.900;

    /// Inicia um novo ciclo do Efeito Gênio (`minimizing = true` para minimizar, `false` para restaurar).
    pub fn new(minimizing: bool) -> Self {
        Self {
            progress: if minimizing { 0.0 } else { 1.0 },
            minimizing,
        }
    }

    /// Avança o progresso $t \in [0, 1]$ multiplicando `dt` por `0.2` se `shift_held` for verdadeiro.
    pub fn step(&mut self, dt: f64, shift_held: bool) -> f64 {
        let eff_dt = if shift_held {
            dt.max(0.0) * SHIFT_SLOW_MOTION_FACTOR
        } else {
            dt.max(0.0)
        };
        let delta = eff_dt / Self::DURATION_NORMAL;
        if self.minimizing {
            self.progress = (self.progress + delta).clamp(0.0, 1.0);
        } else {
            self.progress = (self.progress - delta).clamp(0.0, 1.0);
        }
        self.progress
    }
}

/// 5. Menu Item Confirmation Flash: Destaque `#0A60FF` pisca 2 vezes (`Off 35ms -> On 35ms -> Off 35ms -> On 35ms`) + fade-out ($110\text{ms}$).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuConfirmationFlash {
    elapsed_ms: f64,
}

impl Default for MenuConfirmationFlash {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuConfirmationFlash {
    /// Duração de cada pulso de piscada ($35\text{ms}$).
    pub const BLINK_STEP_MS: f64 = 35.0;
    /// Duração do fade-out final ($110\text{ms}$).
    pub const FADEOUT_MS: f64 = 110.0;
    /// Duração total ($4 \times 35 + 110 = 250\text{ms}$).
    pub const TOTAL_MS: f64 = 250.0;

    /// Inicia o flash de confirmação de item de menu.
    pub fn new() -> Self {
        Self { elapsed_ms: 0.0 }
    }

    /// Avança por `dt` segundos e retorna `(highlight_visible, menu_opacity)`.
    pub fn step(&mut self, dt: f64) -> (bool, f64) {
        self.elapsed_ms = (self.elapsed_ms + dt.max(0.0) * 1000.0).min(Self::TOTAL_MS);
        let t = self.elapsed_ms;
        if t < 35.0 {
            (false, 1.0)
        } else if t < 70.0 {
            (true, 1.0)
        } else if t < 105.0 {
            (false, 1.0)
        } else if t < 140.0 {
            (true, 1.0)
        } else {
            let fade = 1.0 - ((t - 140.0) / Self::FADEOUT_MS).clamp(0.0, 1.0);
            (true, fade)
        }
    }

    /// Verdadeiro quando o ciclo de $250\text{ms}$ terminou.
    pub fn is_completed(&self) -> bool {
        self.elapsed_ms >= Self::TOTAL_MS
    }
}

/// 6. Amortecimento de Entrada/Saída do Dock ($\alpha_{\text{entry}} \in [0, 1]$ com mola $k = 500, c = 45$).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockEntrySpring {
    spring: SpringAnimation,
}

impl Default for DockEntrySpring {
    fn default() -> Self {
        Self::new()
    }
}

impl DockEntrySpring {
    /// Cria o amortecedor de entrada do Dock em repouso ($\alpha_{\text{entry}} = 0.0$).
    pub fn new() -> Self {
        let params = SpringParams {
            mass: 1.0,
            stiffness: 500.0,
            damping: 45.0,
        };
        Self {
            spring: SpringAnimation::new(params, 0.0, 0.0, 0.0),
        }
    }

    /// Define se o cursor está dentro do Dock (`inside = true` $\to 1.0$, `false` $\to 0.0$).
    pub fn set_cursor_inside(&mut self, inside: bool) {
        self.spring.retarget(if inside { 1.0 } else { 0.0 });
    }

    /// Avança por `dt` segundos e retorna $\alpha_{\text{entry}} \in [0.0, 1.0]$.
    pub fn step(&mut self, dt: f64) -> f64 {
        self.spring.step(dt).position.clamp(0.0, 1.0)
    }
}

/// 7. Salto Parabólico do Dock (App Launch Bounce): Gravidade $g = 980\text{px/s}^2$, altura `28px`, restituição $e = 0.62$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParabolicDockBounce {
    y: f64,
    vy: f64,
    active: bool,
}

impl Default for ParabolicDockBounce {
    fn default() -> Self {
        Self::new()
    }
}

impl ParabolicDockBounce {
    /// Aceleração gravitacional ($980.0\text{ px/s}^2$).
    pub const GRAVITY: f64 = 980.0;
    /// Altura do salto inicial ($28.0\text{px}$).
    pub const INITIAL_HEIGHT: f64 = 28.0;
    /// Coeficiente de restituição no impacto com a base do Dock ($e = 0.62$).
    pub const RESTITUTION: f64 = 0.62;

    /// Cria e dispara um salto parabólico com velocidade inicial $v_0 = \sqrt{2 g h}$.
    pub fn new() -> Self {
        let v0 = (2.0 * Self::GRAVITY * Self::INITIAL_HEIGHT).sqrt();
        Self {
            y: 0.0,
            vy: v0,
            active: true,
        }
    }

    /// Avança a cinemática balística por `dt` segundos e retorna a altura $y \ge 0$.
    pub fn step(&mut self, dt: f64) -> f64 {
        if !self.active {
            return 0.0;
        }
        let dt = dt.max(0.0);
        self.y += self.vy * dt - 0.5 * Self::GRAVITY * dt * dt;
        self.vy -= Self::GRAVITY * dt;

        if self.y <= 0.0 {
            self.y = 0.0;
            self.vy = -self.vy * Self::RESTITUTION;
            if self.vy.abs() < 18.0 {
                self.vy = 0.0;
                self.active = false;
            }
        }
        self.y
    }

    /// Verdadeiro enquanto o ícone ainda está quicando.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// 8. Abertura de Stacks do Dock (Modo Leque / Fan): Mola ($k = 380, c = 28$), $\theta_k = k \cdot 4.2^\circ$, $\Delta x_k = k^2 \cdot 3.5\text{px}$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockStackFanAnimation {
    spring: SpringAnimation,
}

impl Default for DockStackFanAnimation {
    fn default() -> Self {
        Self::new()
    }
}

impl DockStackFanAnimation {
    /// Incremento angular por item no modo Leque ($4.2^\circ$).
    pub const ANGLE_PER_ITEM_DEG: f64 = 4.2;
    /// Coeficiente quadrático de deslocamento lateral ($3.5\text{px}$).
    pub const OFFSET_X_COEFF: f64 = 3.5;

    /// Cria a mola de abertura de Stack ($m = 1.0, k = 380.0, c = 28.0$).
    pub fn new() -> Self {
        let params = SpringParams {
            mass: 1.0,
            stiffness: 380.0,
            damping: 28.0,
        };
        Self {
            spring: SpringAnimation::new(params, 0.0, 0.0, 1.0),
        }
    }

    /// Avança por `dt` segundos (respeitando `Shift` slow-motion) e retorna o fator de abertura.
    pub fn step(&mut self, dt: f64, shift_held: bool) -> f64 {
        let eff_dt = if shift_held {
            dt.max(0.0) * SHIFT_SLOW_MOTION_FACTOR
        } else {
            dt.max(0.0)
        };
        self.spring.step(eff_dt).position
    }

    /// Calcula `(dx_px, dy_px, theta_deg)` para o $k$-ésimo ícone ($k \ge 1$) na pilha em Leque.
    pub fn item_transform(&self, k: usize, vertical_step: f64) -> (f64, f64, f64) {
        let factor = self.spring.sample_at(self.spring.elapsed_time()).position;
        let kf = k as f64;
        let dx = (kf * kf * Self::OFFSET_X_COEFF) * factor;
        let dy = -(kf * vertical_step) * factor;
        let theta_deg = (kf * Self::ANGLE_PER_ITEM_DEG) * factor;
        (dx, dy, theta_deg)
    }
}

/// 9. Mission Control, App Exposé, Show Desktop & Spaces Swipe ($m = 1.0, k = 320.0, c = 35.777, \zeta = 1.0$).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GestureSwipeTransition {
    spring_2d: Spring2D,
}

impl GestureSwipeTransition {
    /// Inicia a transição na coordenada atual com velocidade injetada $(\dot{x}, \dot{y})$.
    pub fn release_with_velocity(
        current_pos: (f64, f64),
        velocity: (f64, f64),
        target_pos: (f64, f64),
    ) -> Self {
        Self {
            spring_2d: Spring2D::new(
                SpringParams::mission_control(),
                current_pos,
                velocity,
                target_pos,
            ),
        }
    }

    /// Avança por `dt` segundos e retorna `(pos_xy, vel_xy)`.
    pub fn step(&mut self, dt: f64) -> ((f64, f64), (f64, f64)) {
        self.spring_2d.step(dt)
    }
}

/// 10. Stage Manager 3D Flip: Perspectiva $\text{FOV} = 45^\circ$, rotação Y de $0^\circ \to 14^\circ$ ($m=1, k=280, c=31$).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StageManager3DFlip {
    angle_spring: SpringAnimation,
}

impl Default for StageManager3DFlip {
    fn default() -> Self {
        Self::new()
    }
}

impl StageManager3DFlip {
    /// Campo de visão em graus ($\text{FOV} = 45^\circ$).
    pub const FOV_DEG: f64 = 45.0;
    /// Ângulo lateral em repouso na pilha 3D ($14.0^\circ$).
    pub const LATERAL_TILT_DEG: f64 = 14.0;

    /// Cria a animação de rotação Y de $0^\circ \to 14^\circ$.
    pub fn new() -> Self {
        Self {
            angle_spring: SpringAnimation::new(
                SpringParams::stage_manager(),
                0.0,
                0.0,
                Self::LATERAL_TILT_DEG,
            ),
        }
    }

    /// Alterna entre pilha lateral ($14^\circ$) e palco central ($0^\circ$).
    pub fn set_in_lateral_strip(&mut self, in_strip: bool) {
        self.angle_spring
            .retarget(if in_strip { Self::LATERAL_TILT_DEG } else { 0.0 });
    }

    /// Avança por `dt` segundos e retorna o ângulo de rotação Y em graus.
    pub fn step(&mut self, dt: f64) -> f64 {
        self.angle_spring.step(dt).position
    }
}

/// 11. Sheets Modais (Diálogos Presos à Barra de Título): Rotação 3D no eixo X ($-8^\circ \to 0^\circ$) em $220\text{ms}$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModalSheetUnfurl {
    elapsed: f64,
}

impl Default for ModalSheetUnfurl {
    fn default() -> Self {
        Self::new()
    }
}

impl ModalSheetUnfurl {
    /// Duração de abertura do Sheet modal ($220\text{ms}$).
    pub const DURATION: f64 = 0.220;
    /// Ângulo inicial de inclinação 3D no eixo X ($-8.0^\circ$).
    pub const INITIAL_PITCH_DEG: f64 = -8.0;

    /// Inicia o desenrolar do Sheet modal preso à barra de título.
    pub fn new() -> Self {
        Self { elapsed: 0.0 }
    }

    /// Avança por `dt` segundos e retorna `(pitch_deg, slide_progress, shadow_alpha)`.
    pub fn step(&mut self, dt: f64) -> (f64, f64, f64) {
        self.elapsed = (self.elapsed + dt.max(0.0)).min(Self::DURATION);
        let u = self.elapsed / Self::DURATION;
        let s = u * u * (3.0 - 2.0 * u);
        let pitch_deg = Self::INITIAL_PITCH_DEG * (1.0 - s);
        let shadow_alpha = 0.42 * s;
        (pitch_deg, s, shadow_alpha)
    }
}

/// 12. Shake de Senha Incorreta (Lock Screen): $\Delta x(t) = 14.0 \cdot e^{-8t} \cdot \sin(10\pi t)$ durante $420\text{ms}$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IncorrectPasswordShake {
    elapsed: f64,
}

impl Default for IncorrectPasswordShake {
    fn default() -> Self {
        Self::new()
    }
}

impl IncorrectPasswordShake {
    /// Duração total do shake de senha incorreta ($420\text{ms}$).
    pub const DURATION: f64 = 0.420;
    /// Amplitude inicial ($14.0\text{px}$).
    pub const AMPLITUDE: f64 = 14.0;
    /// Taxa de decaimento exponencial ($8.0\text{ s}^{-1}$).
    pub const DECAY_RATE: f64 = 8.0;

    /// Cria um novo shake de senha incorreta.
    pub fn new() -> Self {
        Self { elapsed: 0.0 }
    }

    /// Avalia analiticamente $\Delta x(t) = 14.0 \cdot e^{-8t} \cdot \sin(10\pi t)$ no instante $t$.
    #[inline]
    pub fn sample_at(t: f64) -> f64 {
        if !(0.0..=Self::DURATION).contains(&t) {
            return 0.0;
        }
        Self::AMPLITUDE * (-Self::DECAY_RATE * t).exp() * (10.0 * PI * t).sin()
    }

    /// Avança por `dt` segundos e retorna o deslocamento horizontal $\Delta x$.
    pub fn step(&mut self, dt: f64) -> f64 {
        self.elapsed += dt.max(0.0);
        Self::sample_at(self.elapsed)
    }
}

/// Esfera procedural de fumaça branca do efeito "Poof" do Dock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoofSmokeSphere {
    /// Centro atual $(x, y)$ em relação ao ponto de soltura.
    pub offset: (f64, f64),
    /// Raio atual da esfera em pixels.
    pub radius: f64,
    /// Opacidade atual $\in [0.0, 1.0]$.
    pub opacity: f64,
}

/// 13. Efeito "Poof" (5 Esferas de Fumaça em $160\text{ms}$) & Snap-Back Elástico ($< 100\text{px}$, $k=400, c=26$).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DockDragReleaseOutcome {
    /// Arraste $< 100\text{px}$: retorna elasticamente à posição original no Dock.
    SnapBack(Spring2D),
    /// Arraste $\ge 100\text{px}$: explode em 5 esferas procedurais de fumaça branca durante $160\text{ms}$.
    Poof {
        /// Tempo decorrido em segundos.
        elapsed: f64,
    },
}

impl DockDragReleaseOutcome {
    /// Limiar de distância para remoção com efeito Poof ($100.0\text{px}$).
    pub const POOF_DISTANCE_THRESHOLD: f64 = 100.0;
    /// Duração da dissipação das 5 esferas de fumaça ($160\text{ms}$).
    pub const POOF_DURATION: f64 = 0.160;

    /// Decide entre Snap-Back elástico ($k=400, c=26$) ou Poof de 5 esferas conforme a distância arrastada.
    pub fn from_release(drag_pos: (f64, f64), dock_origin: (f64, f64)) -> Self {
        let dx = drag_pos.0 - dock_origin.0;
        let dy = drag_pos.1 - dock_origin.1;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < Self::POOF_DISTANCE_THRESHOLD {
            let params = SpringParams {
                mass: 1.0,
                stiffness: 400.0,
                damping: 26.0,
            };
            Self::SnapBack(Spring2D::new(params, drag_pos, (0.0, 0.0), dock_origin))
        } else {
            Self::Poof { elapsed: 0.0 }
        }
    }

    /// Calcula o estado instantâneo das 5 esferas procedurais de fumaça branca no instante `elapsed`.
    pub fn evaluate_poof_spheres(elapsed: f64) -> [PoofSmokeSphere; 5] {
        let u = (elapsed / Self::POOF_DURATION).clamp(0.0, 1.0);
        let angles: [f64; 5] = [0.0, 1.256637, 2.513274, 3.769911, 5.026548]; // 5 direções pentagonais
        let mut spheres = [PoofSmokeSphere {
            offset: (0.0, 0.0),
            radius: 0.0,
            opacity: 0.0,
        }; 5];
        for (i, &ang) in angles.iter().enumerate() {
            let dist = 6.0 + 26.0 * u;
            let radius = 10.0 + 14.0 * u;
            let opacity = (1.0 - u * u).clamp(0.0, 1.0);
            spheres[i] = PoofSmokeSphere {
                offset: (ang.cos() * dist, ang.sin() * dist),
                radius,
                opacity,
            };
        }
        spheres
    }
}

/// 14. Jiggle Mode (Launchpad & Customização da Toolbar do Finder):
/// $\theta(t) = 1.8^\circ \cdot \sin(28t + \phi_i)$ e $\Delta y(t) = 1.0\text{px} \cdot \cos(24t + \phi_i)$.
pub struct JiggleModeOscillator;

impl JiggleModeOscillator {
    /// Amplitude angular ($1.8^\circ$).
    pub const AMPLITUDE_DEG: f64 = 1.8;
    /// Amplitude vertical ($1.0\text{px}$).
    pub const AMPLITUDE_Y_PX: f64 = 1.0;

    /// Avalia `(theta_deg, delta_y_px)` para o ícone de índice `item_index` no instante `t`.
    #[inline]
    pub fn sample(t: f64, item_index: usize) -> (f64, f64) {
        let phi = (item_index as f64) * 0.85;
        let theta_deg = Self::AMPLITUDE_DEG * (28.0 * t + phi).sin();
        let dy_px = Self::AMPLITUDE_Y_PX * (24.0 * t + phi).cos();
        (theta_deg, dy_px)
    }
}

/// 16. Rubber-Banding Linear + Retorno por Mola ($\Delta y_{\text{vis}} = \Delta y_{\text{raw}} \cdot 0.35$).
pub struct LinearRubberBand;

impl LinearRubberBand {
    /// Fator de resistência elástica além dos limites ($0.35$).
    pub const RESISTANCE_FACTOR: f64 = 0.35;

    /// Converte o excesso de rolagem bruto $\Delta y_{\text{raw}}$ em deslocamento visual $\Delta y_{\text{vis}}$.
    #[inline]
    pub fn attenuate(raw_overscroll: f64) -> f64 {
        raw_overscroll * Self::RESISTANCE_FACTOR
    }
}

/// 17. Animação de Download Concluído (Fly-to-Dock em Curva de Bézier Quadrática + Micro-Salto da Pilha).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DownloadFlyToDock {
    start: (f64, f64),
    control: (f64, f64),
    end: (f64, f64),
    elapsed: f64,
    stack_bounce: SpringAnimation,
}

impl DownloadFlyToDock {
    /// Duração do voo parabólico até a Stack de Downloads ($480\text{ms}$).
    pub const FLIGHT_DURATION: f64 = 0.480;

    /// Cria o voo em curva de Bézier quadrática de `start` até `end`.
    pub fn new(start: (f64, f64), end: (f64, f64)) -> Self {
        let control = ((start.0 + end.0) * 0.5, start.1.min(end.1) - 140.0);
        Self {
            start,
            control,
            end,
            elapsed: 0.0,
            stack_bounce: SpringAnimation::new(SpringParams::dock_bounce(), 0.0, 0.0, 0.0),
        }
    }

    /// Avança por `dt` segundos e retorna `((x, y), scale, stack_bounce_offset)`.
    pub fn step(&mut self, dt: f64) -> ((f64, f64), f64, f64) {
        let prev = self.elapsed;
        self.elapsed += dt.max(0.0);
        let u = (self.elapsed / Self::FLIGHT_DURATION).clamp(0.0, 1.0);
        let om = 1.0 - u;
        let x = om * om * self.start.0 + 2.0 * om * u * self.control.0 + u * u * self.end.0;
        let y = om * om * self.start.1 + 2.0 * om * u * self.control.1 + u * u * self.end.1;
        let scale = 1.0 - 0.45 * u;

        if prev < Self::FLIGHT_DURATION && self.elapsed >= Self::FLIGHT_DURATION {
            self.stack_bounce = SpringAnimation::new(SpringParams::dock_bounce(), 10.0, 120.0, 0.0);
        }
        let bounce_y = if self.elapsed >= Self::FLIGHT_DURATION {
            self.stack_bounce.step(dt).position
        } else {
            0.0
        };
        ((x, y), scale, bounce_y)
    }
}

/// 18. Desaceleração Contínua do Papel de Parede Aerial (Lock Screen $\to$ Desktop): $v(t) = v_0 e^{-3.5t}$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AerialWallpaperDecelerator {
    initial_velocity: f64,
    elapsed: f64,
    position: f64,
}

impl AerialWallpaperDecelerator {
    /// Constante de decaimento exponencial ($\lambda = 3.5\text{ s}^{-1}$).
    pub const DECAY_LAMBDA: f64 = 3.5;

    /// Inicia o desacelerador a 120fps com velocidade inicial $v_0$.
    pub fn new(initial_velocity: f64) -> Self {
        Self {
            initial_velocity,
            elapsed: 0.0,
            position: 0.0,
        }
    }

    /// Avalia a velocidade instantânea $v(t) = v_0 e^{-3.5t}$ e integra a posição.
    pub fn step(&mut self, dt: f64) -> (f64, f64) {
        self.elapsed += dt.max(0.0);
        let v = self.initial_velocity * (-Self::DECAY_LAMBDA * self.elapsed).exp();
        self.position = (self.initial_velocity / Self::DECAY_LAMBDA)
            * (1.0 - (-Self::DECAY_LAMBDA * self.elapsed).exp());
        (self.position, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_18_animations_catalog() {
        // 1. Window Open
        let mut w_open = WindowOpenAnimation::new();
        let (sc, op) = w_open.step(0.20, false);
        assert!(sc > 0.98 && op >= 1.0);

        // 2. Window Close
        let mut w_close = WindowCloseAnimation::new();
        let (sc_c, op_c) = w_close.step(0.14);
        assert!((sc_c - 0.95).abs() < 1e-5 && op_c.abs() < 1e-5);
        assert!(w_close.is_finished());

        // 3. Quick Look Morph
        let mut ql = QuickLookMorphAnimation::new((100.0, 100.0, 64.0, 64.0), (400.0, 200.0, 800.0, 600.0));
        let r = ql.step(0.5);
        assert!(r.2 > 600.0);

        // 4. Genie Shift Slow-Motion (0.2x speed -> 1900ms)
        let mut genie_slow = GenieWarpTimer::new(true);
        let p_slow = genie_slow.step(0.380, true);
        assert!((p_slow - 0.2).abs() < 1e-4);

        // 5. Menu Confirmation Flash (35ms steps + 110ms fade)
        let mut flash = MenuConfirmationFlash::new();
        assert_eq!(flash.step(0.010).0, false); // 10ms: Off
        assert_eq!(flash.step(0.040).0, true);  // 50ms: On
        assert_eq!(flash.step(0.035).0, false); // 85ms: Off
        assert_eq!(flash.step(0.035).0, true);  // 120ms: On

        // 6. Dock Entry Spring (k=500, c=45)
        let mut dock_entry = DockEntrySpring::new();
        dock_entry.set_cursor_inside(true);
        assert!(dock_entry.step(0.2) > 0.8);

        // 7. Parabolic Dock Bounce (g=980, h=28, e=0.62)
        let mut bounce = ParabolicDockBounce::new();
        let y_mid = bounce.step(0.05);
        assert!(y_mid > 0.0 && bounce.is_active());

        // 8. Stack Fan (theta = k * 4.2 deg, dx = k^2 * 3.5px)
        let mut fan = DockStackFanAnimation::new();
        fan.step(1.0, false);
        let (dx, _dy, theta) = fan.item_transform(2, 56.0);
        assert!((dx - 14.0).abs() < 0.5);
        assert!((theta - 8.4).abs() < 0.5);

        // 9 & 10. Gesture Swipe & Stage Manager 3D Flip
        let mut flip = StageManager3DFlip::new();
        assert!(flip.step(0.5) > 13.0);

        // 11. Modal Sheet Unfurl (-8 deg -> 0 deg in 220ms)
        let mut sheet = ModalSheetUnfurl::new();
        let (pitch, prog, _) = sheet.step(0.220);
        assert!(pitch.abs() < 1e-5 && (prog - 1.0).abs() < 1e-5);

        // 12. Password Shake (14 * e^(-8t) * sin(10*pi*t))
        let mut shake = IncorrectPasswordShake::new();
        assert!(shake.step(0.05).abs() > 1.0);
        assert_eq!(IncorrectPasswordShake::sample_at(0.50), 0.0);

        // 13. Poof Effect (<100px vs >=100px)
        let snap = DockDragReleaseOutcome::from_release((100.0, 50.0), (100.0, 0.0));
        assert!(matches!(snap, DockDragReleaseOutcome::SnapBack(_)));
        let poof = DockDragReleaseOutcome::from_release((100.0, 150.0), (100.0, 0.0));
        assert!(matches!(poof, DockDragReleaseOutcome::Poof { .. }));
        let spheres = DockDragReleaseOutcome::evaluate_poof_spheres(0.08);
        assert_eq!(spheres.len(), 5);

        // 14. Jiggle Mode
        let (j_theta, j_dy) = JiggleModeOscillator::sample(0.1, 3);
        assert!(j_theta.abs() <= 1.8 && j_dy.abs() <= 1.0);

        // 16. Linear Rubber-Band (0.35)
        assert!((LinearRubberBand::attenuate(100.0) - 35.0).abs() < 1e-6);

        // 17. Download Fly-to-Dock
        let mut fly = DownloadFlyToDock::new((200.0, 200.0), (800.0, 900.0));
        let (pos, _sc, _b) = fly.step(0.480);
        assert!((pos.0 - 800.0).abs() < 1e-4);

        // 18. Aerial Wallpaper Decelerator
        let mut aerial = AerialWallpaperDecelerator::new(100.0);
        let (_pos_a, v_a) = aerial.step(1.0);
        assert!(v_a < 10.0);
    }
}
