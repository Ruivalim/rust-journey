use crate::common::despawn_screen;
use crate::common::GameState;
use bevy::prelude::*;

#[derive(Component)]
struct OnMenuScreen;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Quit,
}

pub fn menu_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Menu), setup)
        .add_systems(OnExit(GameState::Menu), despawn_screen::<OnMenuScreen>)
        .add_systems(Update, menu_action.run_if(in_state(GameState::Menu)));
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
            OnMenuScreen,
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
                        Text::new("Bevy Bird"),
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
                        .spawn((Button, button_node.clone(), MenuButtonAction::Play))
                        .with_children(|parent| {
                            parent.spawn((Text::new("New Game"), button_text_font.clone()));
                        });
                    parent
                        .spawn((Button, button_node, MenuButtonAction::Quit))
                        .with_children(|parent| {
                            parent.spawn((Text::new("Quit"), button_text_font));
                        });
                });
        });
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.send(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    game_state.set(GameState::Game);
                }
            }
        }
    }
}
