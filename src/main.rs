use macroquad::prelude::*;
use macroquad::ui::{self, hash, widgets};

const G: f32 = 0.1;
const RESTITUTION_COEFFICIENT: f32 = 0.3;
const ZOOM_SPEED: Vec2 = vec2(0.1, 0.1);
const MIN_ZOOM: Vec2 = vec2(0.1, 0.1);
const MAX_ZOOM: Vec2 = vec2(1.0, 1.0);

#[derive(Clone)]
struct Planet {
    position: Vec2,
    radius: f32,
    velocity: Vec2,
    mass: f32,
    history: Vec<Vec2>,
    color: Color,
}

impl Planet {
    fn new(position: Vec2, radius: f32, velocity: Vec2, mass: f32, color: Color) -> Self {
        Planet {
            position,
            radius,
            velocity,
            mass,
            history: Vec::new(),
            color,
        }
    }

    fn update(&mut self, other_planets: &mut [Planet]) {
        let mut acceleration = Vec2::ZERO;

        for i in (0..other_planets.len()).rev() {
            let other_planet = &mut other_planets[i];
            if self.position != other_planet.position {
                let direction = other_planet.position - self.position;
                let distance_squared = direction.length_squared();
                let force_magnitude = G * ((other_planet.mass * self.mass) / distance_squared);

                acceleration += direction.normalize() * force_magnitude;

                if distance_squared <= (self.radius + other_planet.radius).powi(2) {
                    let collision_normal = direction.normalize();
                    let relative_velocity = self.velocity - other_planet.velocity;
                    let impulse = (2.0 * self.mass * other_planet.mass)
                        / (self.mass + other_planet.mass)
                        * relative_velocity.dot(collision_normal);
                    let impulse = impulse * RESTITUTION_COEFFICIENT;

                    self.velocity -= impulse * collision_normal;
                    other_planet.velocity += impulse * collision_normal;
                }
            }
        }

        self.velocity += acceleration;
        self.position += self.velocity;

        self.history.push(self.position);
    }

    fn draw(&self) {
        let planet_x = self.position.x - self.radius / 2.0;
        let planet_y = self.position.y - self.radius / 2.0;

        let history_len = self.history.len();

        if history_len > 1 {
            for i in 1..history_len {
                let start = self.history[i - 1];
                let end = self.history[i];
                draw_line(
                    start.x,
                    start.y,
                    end.x,
                    end.y,
                    1.0,
                    Color::new(1.0, 1.0, 1.0, 0.1),
                );
            }
        }

        draw_circle(planet_x, planet_y, self.radius, self.color);
    }
}

struct PlanetParams {
    radius: f32,
    velocity: Vec2,
    mass: f32,
    color: Color,
}

impl PlanetParams {
    fn new() -> Self {
        PlanetParams {
            radius: 10.0,
            mass: 10.0,
            velocity: vec2(0.0, 0.0),
            color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

fn update_planets(planets: &mut Vec<Planet>) {
    let mut planets_clone = planets.clone();

    for planet in planets.iter_mut() {
        planet.update(&mut planets_clone);
    }
}

fn draw_planets(planets: &[Planet]) {
    for planet in planets {
        planet.draw();
    }
}

fn handle_input(
    mut camera: Camera2D,
    planet_params: &PlanetParams,
    planets: &mut Vec<Planet>,
    target: &mut usize,
    spawn_on_click: bool,
    ui_enabled: &mut bool,
) {
    if spawn_on_click && is_mouse_button_pressed(MouseButton::Right) {
        let position = camera.screen_to_world(mouse_position().into());

        let planet = Planet::new(
            position,
            planet_params.radius,
            planet_params.velocity,
            planet_params.mass,
            planet_params.color,
        );

        planets.push(planet);
    }

    if is_key_pressed(KeyCode::Z) {
        *target = (*target + 1) % planets.len();
    }

    if is_key_pressed(KeyCode::X) {
        *target = (*target + planets.len() - 1) % planets.len();
    }

    if is_key_pressed(KeyCode::U) {
        *ui_enabled = !*ui_enabled;
    }

    let zoom_delta: Vec2 = vec2(mouse_wheel().0, mouse_wheel().1) * ZOOM_SPEED;
    camera.zoom = (camera.zoom + zoom_delta).max(MIN_ZOOM).min(MAX_ZOOM);
}

fn draw_ui(planet_params: &mut PlanetParams, spawn_on_click: &mut bool, planets: &mut Vec<Planet>) {
    widgets::Window::new(hash!(), vec2(470., 50.), vec2(300., 300.))
        .label("Planet Creator")
        .ui(&mut *ui::root_ui(), |ui| {
            ui.tree_node(hash!(), "Settings", |ui| {
                ui.slider(
                    hash!("radius"),
                    "Radius",
                    1.0..100.0,
                    &mut planet_params.radius,
                );
                ui.separator();
                ui.slider(
                    hash!("velocity_x"),
                    "Velocity X",
                    -100.0..100.0,
                    &mut planet_params.velocity.x,
                );
                ui.separator();
                ui.slider(
                    hash!("velocity_y"),
                    "Velocity Y",
                    -100.0..100.0,
                    &mut planet_params.velocity.y,
                );
                ui.separator();
                ui.slider(hash!("mass"), "Mass", 1.0..1000.0, &mut planet_params.mass);
                ui.separator();
                ui.slider(
                    hash!("color_red"),
                    "Red",
                    0.0..1.0,
                    &mut planet_params.color.r,
                );
                ui.separator();
                ui.slider(
                    hash!("color_green"),
                    "Green",
                    0.0..1.0,
                    &mut planet_params.color.g,
                );
                ui.separator();
                ui.slider(
                    hash!("color_blue"),
                    "Blue",
                    0.0..1.0,
                    &mut planet_params.color.b,
                );
                ui.separator();
                ui.checkbox(hash!("spawn_on_click"), "Spawn on click", spawn_on_click);
            });
            ui.tree_node(hash!(), "Planets", |ui| {
                let mut remove_planet_index: Option<usize> = None;

                for i in (0..planets.len()).rev() {
                    ui.tree_node(hash!(), &format!("Planet {}", i + 1), |ui| {
                        let planet = &mut planets[i];
                        ui.label(None, &format!("Radius: {}", planet.radius));
                        ui.separator();
                        ui.label(None, &format!("Mass: {}", planet.mass));
                        ui.separator();
                        ui.label(None, &format!("Velocity: {}", planet.velocity));
                        ui.separator();
                        if ui.button(None, "Remove") {
                            remove_planet_index = Some(i);
                        }
                    });

                    if let Some(index) = remove_planet_index {
                        planets.remove(index);
                        break;
                    }
                }
            });
        });
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Planets".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut planet_params = PlanetParams::new();
    let mut camera =
        Camera2D::from_display_rect(Rect::new(0.0, 0.0, screen_width(), screen_height()));
    let mut target = 0;
    let mut planets = vec![
        Planet::new(
            vec2(screen_width() / 2.0, screen_height() / 2.0),
            5.0,
            vec2(-0.1, -0.1),
            5.0,
            Color::new(1.0, 0.0, 0.0, 1.0),
        ),
        Planet::new(
            vec2(screen_width() / 2.0 + 100.0, screen_height() / 2.0),
            10.0,
            vec2(0.1, 0.1),
            10.0,
            Color::new(1.0, 1.0, 1.0, 1.0),
        ),
    ];

    let mut spawn_on_click = false;
    let mut ui_enabled = false;

    loop {
        clear_background(BLACK);

        if !planets.is_empty() {
            camera.target = planets[target].position;
        } else {
            target = 0;
        }

        update_planets(&mut planets);
        handle_input(
            camera,
            &planet_params,
            &mut planets,
            &mut target,
            spawn_on_click,
            &mut ui_enabled,
        );

        set_camera(&camera);
        draw_planets(&planets);
        set_default_camera();

        if ui_enabled {
            draw_ui(&mut planet_params, &mut spawn_on_click, &mut planets);
        }

        next_frame().await
    }
}
