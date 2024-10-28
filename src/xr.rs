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
    let mut buffer = SaveBuffer { buffer: openxr::EventDataBuffer::new() };
    
    loop {
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
                        },
                        SessionState::STOPPING => {
                            info!("Requesting app end from openxr");
                            log_error!(app_handle.session.end());
                        },
                        SessionState::EXITING => {
                            info!("OpenXR exit granted, winding down...");
                            app_handle.exit.store(true, std::sync::atomic::Ordering::Release);
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

        tokio::time::sleep(std::time::Duration::from_micros(100)).await;
        // std::thread::sleep(std::time::Duration::from_micros(100));
    
        if app_handle.exit.load(std::sync::atomic::Ordering::Acquire) {
            warn!("Event Loop alternate termination, OpenXR resources might not have been released!");
            break;
        }
    }
}
