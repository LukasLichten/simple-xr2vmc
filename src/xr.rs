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

    let task = tokio::spawn(event_loop(exit, session.clone()));


    Ok((session, task))
}

async fn event_loop(exit: Arc<AtomicBool>, session: Session<Headless>) {
    let mut buffer = openxr::EventDataBuffer::new();

    let instance = session.instance();
    
    loop {
        match instance.poll_event(&mut buffer) {
            Err(e) => error!("Failed to poll event {}", e.to_string()),
            Ok(None) => (),
            Ok(Some(Event::SessionStateChanged(s))) => {
                if s.session() == session.as_raw() {
                    trace!("Entered State {:?}", s.state());

                    match s.state() {
                        SessionState::IDLE => (),
                        SessionState::READY => {
                            // View configuration is ignored on headless sessions, suggested to 0
                            debug!("Beginning OpenXR Session...");
                            log_error!(session.begin(ViewConfigurationType::from_raw(0)));
                        },
                        SessionState::FOCUSED => {
                            info!("OpenXR Connection Established");
                        },
                        SessionState::STOPPING => {
                            info!("Requesting app end from openxr");
                            log_error!(session.end());
                        },
                        SessionState::EXITING => {
                            info!("OpenXR exit granted, winding down...");
                            exit.store(true, std::sync::atomic::Ordering::Release);
                            break;
                        },
                        _ => ()
                    }
                }
            },
            Ok(Some(_)) => {
                warn!("Unhandled event");
            }
        }

        // tokio::time::sleep(std::time::Duration::from_micros(100)).await;
        std::thread::sleep(std::time::Duration::from_micros(100));
    
        if exit.load(std::sync::atomic::Ordering::Acquire) {
            warn!("Event Loop alternate termination, OpenXR resources might not have been released!");
            break;
        }
    }
    
}
