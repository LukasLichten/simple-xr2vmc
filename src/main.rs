use log::{debug, error, info};
use vmc::VMCMessage;

fn main() {
    let log_level = log::LevelFilter::Debug;
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
    // info!("Connecting to OpenXR runtime...");
    // let entry = openxr::Entry::linked();
    // 
    // let _instance = entry
    //     .create_instance(
    //         &openxr::ApplicationInfo {
    //             application_name: "simple-xr2vmc",
    //             ..Default::default()
    //         },
    //         &openxr::ExtensionSet::default(),
    //         &[],
    //     )?;



    // info!("Starting VMC Performer (Client)...");
    // let socket =  vmc::performer!("127.0.0.1:39539").await?;

    // debug!("Sending Ready for calibration");
    // socket.send(vmc::VMCState::new_calibration(vmc::VMCModelState::Loaded,
    //     vmc::VMCCalibrationMode::Normal, vmc::VMCCalibrationState::WaitingForCalibration)).await?;
    
    // let _res = socket.send(VMCMessage::Time(vmc::VMCTime::elapsed())).await;
    
    
    


    
    
    


    debug!("Exiting...");
    Ok(())
}
