use std::sync::{atomic::AtomicBool, Arc};

use log::{debug, error, info, warn, trace};
use openxr::{headless, ApplicationInfo, Entry, Event, ExtensionSet, FormFactor, Headless, Session, SessionState, ViewConfigurationType};
use tokio::task::JoinHandle;

use crate::log_error;

pub async fn openxr_application(exit: Arc<AtomicBool>) -> Result<(Session<Headless>,JoinHandle<()>), Box<dyn std::error::Error>> {
    info!("Connecting to OpenXR runtime...");
    let entry = Entry::linked();

    let mut enable_extension = ExtensionSet::default();
    // We could check if headless is available, but nah
    enable_extension.mnd_headless = true; 

    let instance = entry
        .create_instance(
            &ApplicationInfo {
                application_name: "simple-xr2vmc",
                ..Default::default()
            },
            &enable_extension,
            &[],
        )?;
    
    if let Ok(p) = instance.properties() {
        debug!("Runtime: {}-{}", p.runtime_name, p.runtime_version.to_string());
    }

    let system = instance.system(FormFactor::HEAD_MOUNTED_DISPLAY)?;
    if let Ok(p) = instance.system_properties(system) {
        debug!("System: {}", p.system_name);
    }

    // openxr::headless
    let session = unsafe {
        debug!("Creating OpenXR Session...");
        let (session, _, _) = instance.create_session::<Headless>(system, &headless::SessionCreateInfo {})?;

        session
    };



    let app_handle = AppXrHandle {
        session: session.clone(),
        exit
    };

    let task = tokio::spawn(event_loop(app_handle));

    Ok((session, task))
}

struct AppXrHandle {
    session: Session<Headless>,
    exit: Arc<AtomicBool>
}

/// Required, as without it we can't async sleep otherwise
struct SaveBuffer {
    buffer: openxr::EventDataBuffer
}

unsafe impl Send for SaveBuffer {}
unsafe impl Sync for SaveBuffer {}

async fn event_loop(app_handle: AppXrHandle) {
    // Create an action set to encapsulate our actions
    let action_set = app_handle.session.instance()
        .create_action_set("input", "input pose information", 0)
        .unwrap();

    let right_action = action_set
        .create_action::<openxr::Posef>("right_hand", "Right Hand Controller", &[])
        .unwrap();
    let left_action = action_set
        .create_action::<openxr::Posef>("left_hand", "Left Hand Controller", &[])
        .unwrap();

    // Bind our actions to input devices using the given profile
    // If you want to access inputs specific to a particular device you may specify a different
    // interaction profile
    app_handle.session.instance()
        .suggest_interaction_profile_bindings(
            app_handle.session.instance()
                .string_to_path("/interaction_profiles/khr/simple_controller")
                .unwrap(),
            &[
                openxr::Binding::new(
                    &right_action,
                    app_handle.session.instance()
                        .string_to_path("/user/hand/right/input/grip/pose")
                        .unwrap(),
                ),
                openxr::Binding::new(
                    &left_action,
                    app_handle.session.instance()
                        .string_to_path("/user/hand/left/input/grip/pose")
                        .unwrap(),
                ),
            ],
        )
        .unwrap();

    // Attach the action set to the session
    app_handle.session.attach_action_sets(&[&action_set]).unwrap();
    let stage = app_handle.session
            .create_reference_space(openxr::ReferenceSpaceType::STAGE, openxr::Posef::IDENTITY)
            .unwrap();

    // Create an action space for each device we want to locate
    let right_space = right_action
        .create_space(&app_handle.session, openxr::Path::NULL, openxr::Posef::IDENTITY)
        .unwrap();
    let left_space = left_action
        .create_space(&app_handle.session, openxr::Path::NULL, openxr::Posef::IDENTITY)
        .unwrap();

    let mut buffer = SaveBuffer { buffer: openxr::EventDataBuffer::new() };
    let mut running = false;
    let (mut time_rel, mut offset) = (std::time::Instant::now(), openxr::sys::Time::from_nanos(0));
    
    loop {
        // Event Handler
        match app_handle.session.instance().poll_event(&mut buffer.buffer) {
            Err(e) => error!("Failed to poll event {}", e.to_string()),
            Ok(None) => (),
            Ok(Some(Event::SessionStateChanged(s))) => {
                if s.session() == app_handle.session.as_raw() {
                    trace!("Entered State {:?}", s.state());

                    match s.state() {
                        SessionState::IDLE => (),
                        SessionState::READY => {
                            // View configuration is ignored on headless sessions, suggested to 0
                            debug!("Beginning OpenXR Session...");
                            log_error!(app_handle.session.begin(ViewConfigurationType::from_raw(0)));
                        },
                        SessionState::FOCUSED => {
                            info!("OpenXR Connection Established");
                            running = true;
                            (time_rel,offset) = (std::time::Instant::now(),s.time());
                        },
                        SessionState::STOPPING => {
                            info!("Requesting app end from openxr");
                            log_error!(app_handle.session.end());
                            running = false;
                        },
                        SessionState::EXITING => {
                            info!("OpenXR exit granted, winding down...");
                            app_handle.exit.store(true, std::sync::atomic::Ordering::Release);
                            break;
                        },
                        SessionState::LOSS_PENDING => {
                            error!("OpenXR runtime is being lost (Instance loss pending received), we exit");
                            app_handle.exit.store(true, std::sync::atomic::Ordering::Release);
                            break;
                        },
                        _ => ()
                    }
                }
            },
            Ok(Some(Event::InstanceLossPending(_))) => {
                error!("OpenXR runtime is being lost (Instance loss pending received), we exit");
                app_handle.exit.store(true, std::sync::atomic::Ordering::Release);
                break;
            },
            Ok(Some(Event::EventsLost(e))) => {
                error!("OpenXR event loop missed {} events, may not function correctly", e.lost_event_count());
            },
            Ok(Some(_)) => {
                warn!("Unhandled event");
            }
        }

        if running {
            let pred_time = {
                let dur = std::time::Instant::now() - time_rel;
                let add:i64 = dur.as_nanos() as i64; // About 500 years before overflow
                openxr::sys::Time::from_nanos(offset.as_nanos() + add)

                // xr_frame_state.predicted_display_time
                // let xr_frame_state = frame_wait.wait().unwrap();
                // xr_frame_state.predicted_display_time
            };

            app_handle.session.sync_actions(&[(&action_set).into()]).unwrap();

            // Find where our controllers are located in the Stage space
            let right_location = right_space
                .locate(&stage, pred_time)
                .unwrap();

            let left_location = left_space
                .locate(&stage, pred_time)
                .unwrap();

            let mut printed = false;
            
            if left_action.is_active(&app_handle.session, openxr::Path::NULL).unwrap() {
                print!(
                    "Left Hand: ({:0<12},{:0<12},{:0<12}), ",
                    left_location.pose.position.x,
                    left_location.pose.position.y,
                    left_location.pose.position.z
                );
                printed = true;
            }

            if right_action.is_active(&app_handle.session, openxr::Path::NULL).unwrap() {
                print!(
                    "Right Hand: ({:0<12},{:0<12},{:0<12})",
                    right_location.pose.position.x,
                    right_location.pose.position.y,
                    right_location.pose.position.z
                );
                printed = true;
            }
            if printed {
                println!();
            } 
        }

        tokio::time::sleep(std::time::Duration::from_micros(1000)).await;

        // Controller reading
    
        if app_handle.exit.load(std::sync::atomic::Ordering::Acquire) {
            warn!("Event Loop alternate termination, OpenXR resources might not have been released!");
            break;
        }
    }
}


