/// Aceleração horizontal do jogador ao se mover para os lados.
pub const PLAYER_ACCELERATION: f32 = 1000.0;
/// Velocidade horizontal máxima que o jogador pode atingir.
pub const PLAYER_SPEED: f32 = 250.0;
/// A velocidade do impulso (dash) do jogador ativado com o botão esquerdo do mouse.
pub const PLAYER_DASH_SPEED: f32 = 1000.0;
/// Duração em segundos do estado de 'dashing'.
pub const PLAYER_DASH_DURATION: f32 = 0.15;
/// Força de atrito aplicada ao jogador quando está no ar.
pub const FRICTION: f32 = 5.0;
/// Força de atrito aplicada quando o jogador está no chão (maior para parar mais rápido).
pub const GROUND_FRICTION: f32 = 10.0;
/// Força de atrito aplicada enquanto o jogador carrega o dash, para mantê-lo no lugar.
pub const CHARGING_FRICTION: f32 = 10.0;
/// Força da gravidade que puxa o jogador para baixo.
pub const GRAVITY: f32 = 750.0;
/// A coordenada Y que define a posição do chão.
pub const FLOOR_Y: f32 = -250.0;
/// Força do impulso vertical (burst) ativado com a barra de espaço.
pub const BURST_FORCE: f32 = 500.0;
/// Força do pequeno salto (hop) ao se mover no chão.
pub const GROUND_HOP_FORCE: f32 = 100.0;
/// Força contínua para cima (bater de asas) ao segurar a tecla W.
pub const AERIAL_VERTICAL_THRUST: f32 = 1200.0;
/// Velocidade vertical máxima atingível através do impulso contínuo (W).
pub const MAX_AERIAL_VERTICAL_SPEED: f32 = 550.0;
/// Força contínua para baixo (mergulho) ao segurar a tecla S no ar.
pub const AERIAL_DOWNWARD_FORCE: f32 = 1500.0;
/// Tempo de recarga em segundos para as habilidades de burst e dash.
pub const ABILITY_COOLDOWN: f32 = 1.0;
/// Multiplicador da gravidade quando o jogador está se movendo lateralmente.
pub const LATERAL_GRAVITY_MULTIPLIER: f32 = 0.5;
/// Multiplicador da gravidade quando o jogador está mirando o dash.
pub const CHARGING_GRAVITY_MULTIPLIER: f32 = 0.1;
