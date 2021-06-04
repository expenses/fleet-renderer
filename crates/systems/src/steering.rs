use bevy_ecs::prelude::*;
use components_and_resources::components::*;
use components_and_resources::gpu_structs::BackgroundVertex;
use components_and_resources::resources::*;
use ultraviolet::Vec3;

mod primitives;

#[profiling::function]
pub fn run_steering(
    mut query: Query<(
        Entity,
        &Position,
        &Velocity,
        &MaxSpeed,
        Option<&Evading>,
        &mut StagingVelocity,
        Option<&OnBoard>,
    )>,
    mut queues: Query<&mut CommandQueue>,
    boids: Query<(
        Entity,
        &Position,
        &Velocity,
        &MaxSpeed,
    )>,
    commands: Commands,
    carrying: Query<(&mut Carrying, &OnBoard)>,
    task_pool: Res<bevy_tasks::TaskPool>,
    mut lines_buffer: ResMut<GpuBuffer<BackgroundVertex>>,
) {
    let commands = parking_lot::Mutex::new(commands);
    let carrying = parking_lot::Mutex::new(carrying);
    let queues = parking_lot::RwLock::new(queues);
    
    query.par_for_each_mut(&task_pool, 8, |(entity, pos, vel, max_speed, evading, mut sv, on_board)| {
        let mut steering = Vec3::zero();
        let boid = to_boid((entity, pos, vel, max_speed));
        let max_force = max_speed.0 / 10.0;

        let mut queues = queues.write();
        let mut queue = queues.get_mut(entity).ok(); 
       
        if let Some(queue) = queue.as_mut() {
            match queue.0.front() {
                Some(Command::Interact { target, ty }) => {
                    if let Ok(target_boid) = boids.get(*target).map(to_boid) {
                        // Because ships are constantly turning, the predicted
                        // point of contact for a ship far away varies a lot, resulting
                        // in an annoying visual wobble. So we disable leading here.
                        // We should fix this someother how though.
                        let lead_factor = 0.0;

                        let force = boid.persue(target_boid, lead_factor);

                        /*lines_buffer.stage(&[
                            BackgroundVertex {
                                position: pos.0,
                                colour: Vec3::unit_x(),
                            },
                            BackgroundVertex {
                                position: pos.0 + force,
                                colour: Vec3::unit_x(),
                            },
                        ]);*/

                        steering += force;

                        if matches!(*ty, InteractionType::BeCarriedBy) && (boid.pos - target_boid.pos).mag_sq() < max_force {
                            match carrying.lock().get_mut(*target) {
                                Ok((mut carrying, carrying_on_board)) => {
                                    carrying.0.push(entity);
                                    let mut commands = commands.lock();
                                    commands.entity(entity)
                                        .insert(OnBoard(Vec::new()))
                                        .remove::<Position>();
                                    if let Some(on_board) = on_board {
                                        let mut new_carrying_on_board = carrying_on_board.0.clone();
                                        new_carrying_on_board.extend_from_slice(&on_board.0);
                                        commands.entity(*target).insert(OnBoard(new_carrying_on_board));
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
                        }
                    } else {
                        queue.0.pop_front();
                    }
                }
                Some(Command::MoveTo { point, .. }) => {
                    let force = boid.seek(*point);

                    steering += force;

                    if (boid.pos - *point).mag_sq() < max_force {
                        queue.0.pop_front();
                    }
                }
                None => {}
            }
        }

        let queues = parking_lot::RwLockWriteGuard::downgrade(queues); 
 
        if let Some(&Evading(evading_entity)) = evading {
            if let Ok(evading_boid) = boids.get(evading_entity).map(to_boid) {
                let force = boid.evade(evading_boid) * 0.5;

                /*lines_buffer.stage(&[
                    BackgroundVertex {
                        position: pos.0,
                        colour: Vec3::unit_y(),
                    },
                    BackgroundVertex {
                        position: pos.0 + force,
                        colour: Vec3::unit_y(),
                    },
                ]);*/

                steering += force;
            } else {
                commands.lock().entity(entity).remove::<Evading>();
            }
        }

        {
            let get_proximity_interact_entity = |queue: Option<&CommandQueue>| match queue.and_then(|queue| queue.0.front()) {
                    Some(Command::Interact { target, ty: InteractionType::BeCarriedBy }) => Some(*target),
                _ => None
            };
            let proximity_interact_entity = get_proximity_interact_entity(queue.as_ref().map(|queue| &**queue));
            let iter = boids.iter()
                .filter(|&(avoid_entity, ..)| {
                    let avoid_queue = queues.get(avoid_entity).ok();
                    Some(avoid_entity) != proximity_interact_entity &&
                       get_proximity_interact_entity(avoid_queue.as_ref().map(move |q| &**q)) != Some(entity) 
                })
                .map(to_boid);
            let force = boid.avoidance(iter);

            steering += force;

            /*
            lines_buffer.stage(&[
                BackgroundVertex {
                    position: pos.0,
                    colour: Vec3::new(1.0, 0.5, 0.0)
                },
                BackgroundVertex {
                    position: pos.0 + force,
                    colour: Vec3::new(1.0, 0.5, 0.0)
                }
            ]);
            */
        }

        if steering == Vec3::zero() {
            steering = -boid.vel;
        }

        let steering = truncate(steering, max_force);

        /*lines_buffer.stage(&[
            BackgroundVertex {
                position: pos.0,
                colour: Vec3::unit_z(),
            },
            BackgroundVertex {
                position: pos.0 + steering,
                colour: Vec3::unit_z(),
            },
        ]);*/

        *sv = StagingVelocity(truncate(vel.0 + steering, max_speed.0));
    })
}

fn to_boid(
    (_, pos, vel, max_speed): (
        Entity,
        &Position,
        &Velocity,
        &MaxSpeed,
    ),
) -> primitives::Boid {
    primitives::Boid {
        pos: pos.0,
        vel: vel.0,
        max_vel: max_speed.0,
        radius_sq: 1.0_f32.powi(2),
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

pub fn apply_staging_velocity(
    mut query: Query<(&mut Velocity, &StagingVelocity)>,
    paused: Res<Paused>,
) {
    if paused.0 {
        return;
    }
    query.for_each_mut(|(mut velocity, staging_velocity)| {
        velocity.0 = staging_velocity.0;
    });
}
