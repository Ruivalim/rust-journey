use crate::common::despawn_screen;
use crate::common::GameState;
use bevy::prelude::*;

#[derive(Component)]
struct OnGameOverScreen;

#[derive(Component)]
enum GameOverActionButtons {
    PlayAgain,
    Quit,
}

pub fn gameover_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::GameOver), setup)
        .add_systems(
            OnExit(GameState::GameOver),
            despawn_screen::<OnGameOverScreen>,
        )
        .add_systems(
            Update,
            gameover_action.run_if(in_state(GameState::GameOver)),
        );
}

fn setup(mut commands: Commands) {
    let button_node = Node {
        width: Val::Px(300.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_font = TextFont {
        font_size: 33.0,
        ..default()
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnGameOverScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    // Display the game name
                    parent.spawn((
                        Text::new("YOU LOST!!!!!!!"),
                        TextFont {
                            font_size: 67.0,
                            ..default()
                        },
                        Node {
                            margin: UiRect::all(Val::Px(50.0)),
                            ..default()
                        },
                    ));
                    parent
                        .spawn((
                            Button,
                            button_node.clone(),
                            GameOverActionButtons::PlayAgain,
                        ))
                        .with_children(|parent| {
                            parent.spawn((Text::new("Try Again"), button_text_font.clone()));
                        });
                    parent
                        .spawn((Button, button_node, GameOverActionButtons::Quit))
                        .with_children(|parent| {
                            parent.spawn((Text::new("Quit"), button_text_font));
                        });
                });
        });
}

fn gameover_action(
    interaction_query: Query<
        (&Interaction, &GameOverActionButtons),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, gameover_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match gameover_button_action {
                GameOverActionButtons::Quit => {
                    app_exit_events.send(AppExit::Success);
                }
                GameOverActionButtons::PlayAgain => {
                    game_state.set(GameState::Game);
                }
            }
        }
    }
}
