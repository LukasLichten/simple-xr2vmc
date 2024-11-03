use log::{debug, error, info};

mod xr;

fn main() {
    let log_level = log::LevelFilter::Trace;
    env_logger::builder().filter_level(log_level).init();


    let res = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .map(|run| run.block_on(runner()));

    match res {
        Ok(Ok(_)) => {
            info!("Successfully exited");
        },
        Ok(Err(e)) => {
            error!("Fatal Error: {e}");
            std::process::exit(1);
        },
        Err(e) => {
            error!("Failed to start tokio runtime: {e}");
            std::process::exit(10);
        }

    }
}

async fn runner() -> Result<(), Box<dyn std::error::Error>> {
    let exit = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    let (session,vr_loop) = xr::openxr_application(exit.clone()).await?;
    
    
    ctrlc::set_handler(move || {
        info!("Signal received, winding down");

        if let Err(e) = session.request_exit() {
            error!("Unable to wind down steamvr: {}", e.to_string());
            exit.store(true, std::sync::atomic::Ordering::Release);
        }
    })?;
    


    // info!("Starting VMC Performer (Client)...");
    // let socket =  vmc::performer!("127.0.0.1:39539").await?;

    // debug!("Sending Ready for calibration");
    // socket.send(vmc::VMCState::new_calibration(vmc::VMCModelState::Loaded,
    //     vmc::VMCCalibrationMode::Normal, vmc::VMCCalibrationState::WaitingForCalibration)).await?;
    
    // let _res = socket.send(VMCMessage::Time(vmc::VMCTime::elapsed())).await;
    
    

    log_error!(vr_loop.await);
    
    
    debug!("Exiting...");
    
    // session.request_exit()?;
    // session.end()?;


    Ok(())
}

#[macro_export]
macro_rules! log_error {
    ($res:expr) => {
        if let Err(e) = $res {
            error!("{}", e.to_string());
        }
    };
}
