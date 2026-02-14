use bevy::{
    camera_controller::free_camera::FreeCameraState,
    feathers::{
        self,
        controls::checkbox,
        theme::{ThemeBackgroundColor, ThemedText},
    },
    pbr::wireframe::WireframeConfig,
    prelude::*,
    ui_widgets::{ValueChange, checkbox_self_update, observe},
};

#[derive(Resource)]
pub struct Settings {
    pub move_cars: bool,
    pub wireframe_enabled: bool,
    pub shadow_maps_enabled: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for Settings {
    fn default() -> Self {
        Self {
            move_cars: false,
            wireframe_enabled: false,
            shadow_maps_enabled: false,
        }
    }
}

pub struct SettingsUiPlugin;
impl Plugin for SettingsUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::default())
            .add_systems(Startup, setup_settings_ui);
    }
}

fn setup_settings_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        ThemeBackgroundColor(feathers::tokens::WINDOW_BG),
        observe(
            |_: On<Pointer<Over>>, mut free_camera_state: Single<&mut FreeCameraState>| {
                free_camera_state.enabled = false;
            },
        ),
        observe(
            |_: On<Pointer<Out>>, mut free_camera_state: Single<&mut FreeCameraState>| {
                free_camera_state.enabled = true;
            },
        ),
        children![(
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Start,
                row_gap: px(8),
                ..default()
            },
            children![
                (Text("Settings".to_owned())),
                (
                    checkbox((), Spawn((Text::new("Move Cars"), ThemedText))),
                    observe(checkbox_self_update),
                    observe(
                        |change: On<ValueChange<bool>>, mut settings: ResMut<Settings>| {
                            settings.move_cars = change.value;
                        }
                    )
                ),
                (
                    checkbox((), Spawn((Text::new("Wireframe Enabled"), ThemedText))),
                    observe(checkbox_self_update),
                    observe(
                        |change: On<ValueChange<bool>>,
                         mut settings: ResMut<Settings>,
                         mut wireframe_config: ResMut<WireframeConfig>| {
                            settings.wireframe_enabled = change.value;
                            wireframe_config.global = change.value;
                        }
                    )
                ),
                (
                    checkbox(
                        (),
                        Spawn((Text::new("Shadow maps enabled"), ThemedText))
                    ),
                    observe(checkbox_self_update),
                    observe(
                        |change: On<ValueChange<bool>>,
                         mut settings: ResMut<Settings>,
                         mut directional_lights: Query<&mut DirectionalLight>| {
                            settings.shadow_maps_enabled = change.value;
                            for mut light in &mut directional_lights {
                                light.shadow_maps_enabled = change.value;

                            }
                        }
                    )
                ),
            ]
        )],
    ));
}
