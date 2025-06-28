use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32, FontId, Frame, Margin, RichText},
    EguiContextPass, EguiContexts, EguiPlugin,
};

use crate::{
    game_state::GameState,
    health::Health,
    input::{Input, MousePos},
    physics::{Grounded, Velocity},
    player::{ChargingDash, Dashing, Player},
    score::Score,
};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_systems(
            EguiContextPass,
            (score_ui_system, debug_ui_system).run_if(in_state(GameState::GameRunning)),
        )
        .add_systems(
            EguiContextPass,
            (gameover_ui_system).run_if(in_state(GameState::GameOver)),
        );
    }
}

fn debug_ui_system(
    mut contexts: EguiContexts,
    input: Res<Input>,
    mouse_pos: Res<MousePos>,
    player: Query<
        (
            Entity,
            &Transform,
            &Velocity,
            &Health,
            Option<&Grounded>,
            Option<&ChargingDash>,
            Option<&Dashing>,
        ),
        With<Player>,
    >,
) {
    if let Some(ctx) = contexts.try_ctx_mut() {
        let input_dir = input.dir();
        let mouse_pos = **mouse_pos;

        egui::Area::new("debug input".into())
            .anchor(Align2::LEFT_TOP, (16., 16.))
            .show(ctx, |ui| {
                ui.set_min_width(1000.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(format!("Input:\n{:.2} {:.2}", input_dir.x, input_dir.y))
                            .color(Color32::WHITE)
                            .font(FontId::proportional(16.0)),
                    );
                    ui.label(
                        RichText::new(format!("Mouse:\n{:.0} {:.0}", mouse_pos.x, mouse_pos.y))
                            .color(Color32::WHITE)
                            .font(FontId::proportional(16.0)),
                    );
                });
            });

        if let Ok((entity, transform, vel, health, grounded_opt, charging_opt, dashing_opt)) =
            player.single()
        {
            egui::Area::new("debug player".into())
                .anchor(Align2::LEFT_TOP, (16., 96.))
                .show(ctx, |ui| {
                    ui.set_min_width(1000.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(format!("entity-idx: {}", entity.index()))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!(
                                "position:\n{:.2},{:.2}",
                                transform.translation.x, transform.translation.y
                            ))
                            .color(Color32::WHITE)
                            .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!(
                                "velocity (current):\n{:.2} {:.2}",
                                vel.current.x, vel.current.y
                            ))
                            .color(Color32::WHITE)
                            .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!(
                                "velocity (target):\n{:.2} {:.2}",
                                vel.target.x, vel.target.y
                            ))
                            .color(Color32::WHITE)
                            .font(FontId::proportional(16.0)),
                        );

                        ui.label(
                            RichText::new(format!("hp: {}/{}", health.current, health.max))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );

                        ui.label(
                            RichText::new(format!("grounded: {}", grounded_opt.is_some()))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!("charging: {}", charging_opt.is_some()))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!("dashing: {}", dashing_opt.is_some()))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );
                    });
                });
        }
    }
}

fn score_ui_system(mut contexts: EguiContexts, score: Res<Score>) {
    if let Some(ctx) = contexts.try_ctx_mut() {
        egui::Area::new("score".into())
            .anchor(Align2::CENTER_TOP, (0., 16.))
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("Score: {}", score.0))
                        .color(Color32::WHITE)
                        .font(FontId::proportional(16.0)),
                );
            });
    }
}

fn gameover_ui_system(mut commands: Commands, mut contexts: EguiContexts, score: Res<Score>) {
    if let Some(ctx) = contexts.try_ctx_mut() {
        egui::CentralPanel::default()
            .frame(
                Frame::NONE
                    .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 220))
                    .inner_margin(Margin::symmetric(24, 16)),
            )
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);

                    ui.label(
                        RichText::new("GAME OVER")
                            .color(Color32::WHITE)
                            .font(FontId::proportional(32.0)),
                    );

                    ui.add_space(4.0);

                    ui.label(
                        RichText::new(format!("Final Score: {}", score.0))
                            .color(Color32::WHITE)
                            .font(FontId::proportional(16.0)),
                    );

                    ui.add_space(16.0);

                    if ui
                        .button(
                            RichText::new("Play Again")
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        )
                        .clicked()
                    {
                        commands.set_state(GameState::GameRunning);
                    }
                });
            });
    }
}
