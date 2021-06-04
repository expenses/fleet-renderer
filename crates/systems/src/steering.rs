use bevy_ecs::prelude::*;
use components_and_resources::components::*;
use components_and_resources::resources::*;
use ultraviolet::Vec3;

mod primitives;

#[profiling::function]
pub fn run_persuit(
    mut query: Query<(
        Entity,
        &Position,
        &Velocity,
        &MaxSpeed,
        Option<&mut CommandQueue>,
        &mut StagingPersuitForce,
    )>,
    on_board: Query<&mut OnBoard>,
    boids: Query<(&Position, &Velocity, &MaxSpeed)>,
    mut commands: Commands,
    mut carrying: Query<&mut Carrying>,
) {
    query.for_each_mut(|(entity, pos, vel, max_speed, queue, mut staging_persuit_force)| {
        let boid = to_boid(pos, vel, max_speed);
        let max_force = max_speed.0 / 10.0;

        if let Some(mut queue) = queue {
            match queue.0.front().cloned() {
                Some(Command::Interact { target, ty, range_sq }) => {
                    if let Ok(target_boid) = boids.get(target).map(|(p, v, ms)| to_boid(p, v, ms)) {
                        // Because ships are constantly turning, the predicted
                        // point of contact for a ship far away varies a lot, resulting
                        // in an annoying visual wobble. So we disable leading here.
                        // We should fix this someother how though.
                        let lead_factor = 0.0;

                        staging_persuit_force.0 = boid.persue(target_boid, lead_factor);

                        let within_range = (boid.pos - target_boid.pos).mag_sq() < range_sq + max_force;

                        if within_range {
                            match ty {
                                InteractionType::BeCarriedBy => {
                                    match carrying.get_mut(target) {
                                        Ok(mut carrying) => {
                                            carrying.0.push(entity);
                                            queue.0.clear();
                                            commands.entity(entity)
                                                .remove::<Position>();

                                            let ship_on_board = unsafe {
                                                on_board.get_unchecked(entity)
                                            };

                                            let carrying_on_board = unsafe {
                                                on_board.get_unchecked(target)
                                            };

                                            if let (Ok(mut ship_on_board), Ok(mut carrying_on_board)) = (ship_on_board, carrying_on_board) {
                                                carrying_on_board.0.append(&mut ship_on_board.0);
                                            }
                                        },
                                        Err(err) => {
                                            log::error!(
                                                "Entity {:?} tried to be carried by {:?} but {:?} cannot carry ships: {}",
                                                entity, target, target, err
                                            );
                                            queue.0.pop_front();
                                        }
                                    }
                                },
                                InteractionType::Mine => {}
                                InteractionType::Attack => {}
                            }
                        }
                    } else {
                        queue.0.pop_front();
                    }
                }
                Some(Command::MoveTo { point, .. }) => {
                    staging_persuit_force.0 = boid.seek(point);

                    if (boid.pos - point).mag_sq() < max_force {
                        queue.0.pop_front();
                    }
                }
                None => {
                    staging_persuit_force.0 = Vec3::zero();
                }
            }
        } else {
            staging_persuit_force.0 = Vec3::zero();
        }
    })
}

#[profiling::function]
pub fn run_evasion(
    mut query: Query<(
        Entity,
        &Position,
        &Velocity,
        &MaxSpeed,
        Option<&Evading>,
        &mut StagingEvasionForce,
    )>,
    boids: Query<(&Position, &Velocity, &MaxSpeed)>,
    mut commands: Commands,
) {
    query.for_each_mut(
        |(entity, pos, vel, max_speed, evading, mut staging_evasion_force)| {
            if let Some(&Evading(entity_to_avoid)) = evading {
                let boid = to_boid(pos, vel, max_speed);

                if let Ok(evading_boid) = boids
                    .get(entity_to_avoid)
                    .map(|(p, v, ms)| to_boid(p, v, ms))
                {
                    staging_evasion_force.0 = boid.evade(evading_boid) * 0.5;
                } else {
                    staging_evasion_force.0 = Vec3::zero();
                    commands.entity(entity).remove::<Evading>();
                }
            } else {
                staging_evasion_force.0 = Vec3::zero();
            }
        },
    )
}

#[profiling::function]
pub fn run_avoidance(
    mut query: Query<(
        Entity,
        &Position,
        &Velocity,
        &MaxSpeed,
        Option<&CommandQueue>,
        &mut StagingAvoidanceForce,
    )>,
    boids: Query<(
        Entity,
        Option<&CommandQueue>,
        &Position,
        &Velocity,
        &MaxSpeed,
    )>,
    task_pool: Res<bevy_tasks::TaskPool>,
) {
    query.par_for_each_mut(
        &task_pool,
        8,
        |(entity, pos, vel, max_speed, queue, mut steering_avoidance_force)| {
            let boid = to_boid(pos, vel, max_speed);

            let get_be_carried_by_entity =
                |queue: Option<&CommandQueue>| match queue.and_then(|queue| queue.0.front()) {
                    Some(Command::Interact {
                        target,
                        ty: InteractionType::BeCarriedBy,
                        ..
                    }) => Some(*target),
                    _ => None,
                };

            let be_carried_by_entity = get_be_carried_by_entity(queue);
            let iter = boids
                .iter()
                .filter(|&(avoid_entity, avoid_queue, ..)| {
                    let avoid_entity_carry_target = get_be_carried_by_entity(avoid_queue);

                    Some(avoid_entity) != be_carried_by_entity
                        && avoid_entity_carry_target != Some(entity)
                })
                .map(|(.., p, v, ms)| to_boid(p, v, ms));

            steering_avoidance_force.0 = boid.avoidance(iter) * 0.1;
        },
    )
}

fn to_boid(pos: &Position, vel: &Velocity, max_speed: &MaxSpeed) -> primitives::Boid {
    primitives::Boid {
        pos: pos.0,
        vel: vel.0,
        max_vel: max_speed.0,
        radius_sq: 1.5_f32.powi(2),
    }
}

fn truncate(vec: Vec3, max: f32) -> Vec3 {
    let mag = vec.mag();
    let new_mag = mag.min(max);
    if new_mag == 0.0 {
        Vec3::zero()
    } else {
        vec / mag * new_mag
    }
}

#[profiling::function]
pub fn apply_staging_velocity(
    mut query: Query<(
        &mut Velocity,
        &MaxSpeed,
        &StagingPersuitForce,
        &StagingEvasionForce,
        &StagingAvoidanceForce,
    )>,
    paused: Res<Paused>,
) {
    if paused.0 {
        return;
    }
    query.for_each_mut(|(mut velocity, max_speed, persuit, evasion, avoidance)| {
        let max_force = max_speed.0 / 10.0;

        let mut steering = persuit.0 + evasion.0 + avoidance.0;

        if steering == Vec3::zero() {
            steering = -velocity.0;
        }

        let steering = truncate(steering, max_force);

        velocity.0 = truncate(velocity.0 + steering, max_speed.0);
    });
}
