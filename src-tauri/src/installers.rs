use tauri::{AppHandle, Emitter};

fn emit_progress(app: &AppHandle, percentage: i32, status: String, ref_name: String, speed_mbps: f64) {
    let _ = app.emit("flatpak-progress", serde_json::json!({
        "percentage": percentage,
        "status": status,
        "ref": ref_name,
        "speed_mbps": speed_mbps
    }));
}

fn emit_operation_started(app: &AppHandle, operation_type: String, ref_name: String) {
    let _ = app.emit("flatpak-operation-started", serde_json::json!({
        "operation_type": operation_type,
        "ref": ref_name
    }));
}

pub mod flatpak {
    use libflatpak::{Installation, Transaction, prelude::*};
    use super::{AppHandle, Emitter, emit_progress, emit_operation_started};

    fn setup_progress_handlers(tx: &Transaction, app: AppHandle) {
        use crate::util::SpeedCalculator;
        
        tx.connect_new_operation({
            let app = app.clone();
            move |_tx, op, progress| {
                let ref_name = op.get_ref().map(|s| s.to_string()).unwrap_or_default();
                let op_type = op.operation_type();
                println!("New operation: {:?} for {}", op_type, ref_name);
                emit_operation_started(&app, format!("{:?}", op_type), ref_name.clone());
                
                let app = app.clone();
                let speed_calc = SpeedCalculator::new();
                
                progress.connect_changed(move |p| {
                    let percentage = p.progress();
                    let bytes = p.bytes_transferred();
                    let speed_mbps = speed_calc.calculate_speed(bytes);
                    let status = p.status().map(|s| s.to_string()).unwrap_or_default();
                    println!("Progress update: {}% - {} ({} bytes)", percentage, status, bytes);
                    emit_progress(&app, percentage, status, ref_name.clone(), speed_mbps);
                });
            }
        });
    }

    fn find_ref(installation: &Installation, ref_id: &str, cancellable: &libflatpak::gio::Cancellable) -> Result<String, String> {
        let remote_ref = installation
            .fetch_remote_ref_sync("flathub", libflatpak::RefKind::App, ref_id, None, Some("stable"), Some(cancellable))
            .or_else(|_| installation.fetch_remote_ref_sync("flathub", libflatpak::RefKind::Runtime, ref_id, None, Some("stable"), Some(cancellable)))
            .map_err(|e| format!("Ref '{}' not found: {}", ref_id, e))?;
        
        remote_ref.format_ref()
            .map(|s| s.to_string())
            .ok_or_else(|| format!("Failed to format ref: {}", ref_id))
    }

    fn find_installed_ref(installation: &Installation, ref_id: &str, cancellable: &libflatpak::gio::Cancellable) -> Result<String, String> {
        let installed_refs = installation.list_installed_refs(Some(cancellable))
            .map_err(|e| format!("Failed to list installed refs: {}", e))?;
        
        for installed_ref in installed_refs {
            if let Some(name) = installed_ref.name() {
                if name == ref_id {
                    return Ok(installed_ref.format_ref().map(|s| s.to_string()).unwrap_or_else(|| format!("app/{}/x86_64/stable", ref_id)));
                }
            }
        }
        
        Err(format!("Ref '{}' is not installed", ref_id))
    }

    #[tauri::command]
    pub async fn install_flatpak(app: AppHandle, ref_name: String) -> Result<(), String> {
        let installation = Installation::new_system(None::<&libflatpak::gio::Cancellable>)
            .map_err(|e| e.to_string())?;
        let cancellable = libflatpak::gio::Cancellable::new();
        
        installation.update_remote_sync("flathub", Some(&cancellable))
            .map_err(|e| e.to_string())?;
        
        let tx = Transaction::for_installation(&installation, Some(&cancellable))
            .map_err(|e| e.to_string())?;

        setup_progress_handlers(&tx, app.clone());

        let ref_id = ref_name.strip_prefix("app/").unwrap_or(&ref_name);
        let full_ref = find_ref(&installation, ref_id, &cancellable)?;
        
        tx.add_install("flathub", &full_ref, &[])
            .map_err(|e| format!("Failed to install {}: {}", ref_id, e))?;

        app.emit("flatpak-install-started", serde_json::json!({ "ref": ref_name }))
            .map_err(|e| e.to_string())?;

        tx.run(Some(&cancellable))
            .map_err(|e| format!("Transaction failed: {}", e))?;

        app.emit("flatpak-install-complete", serde_json::json!({ "ref": ref_name }))
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    #[tauri::command]
    pub async fn uninstall_flatpak(app: AppHandle, ref_name: String) -> Result<(), String> {
        use std::sync::{Arc, Mutex};
        use std::thread;
        use std::time::Duration;
        
        let installation = Installation::new_system(None::<&libflatpak::gio::Cancellable>)
            .map_err(|e| e.to_string())?;
        let cancellable = libflatpak::gio::Cancellable::new();
        
        let tx = Transaction::for_installation(&installation, Some(&cancellable))
            .map_err(|e| e.to_string())?;

        setup_progress_handlers(&tx, app.clone());

        let ref_id = ref_name.strip_prefix("app/").unwrap_or(&ref_name);
        let full_ref = find_installed_ref(&installation, ref_id, &cancellable)?;
        
        app.emit("flatpak-uninstall-started", serde_json::json!({ "ref": ref_name }))
            .map_err(|e| e.to_string())?;
        
        emit_progress(&app, 10, "Preparing uninstallation...".to_string(), ref_name.clone(), 0.0);
        
        tx.add_uninstall(&full_ref)
            .map_err(|e| format!("Failed to uninstall {}: {}", ref_id, e))?;

        let app_clone = app.clone();
        let ref_name_clone = ref_name.clone();
        let is_complete = Arc::new(Mutex::new(false));
        let is_complete_clone = is_complete.clone();
        
        // Start progress simulation in background thread
        let progress_handle = thread::spawn(move || {
            let mut progress = 20;
            loop {
                thread::sleep(Duration::from_millis(300));
                if *is_complete_clone.lock().unwrap() || progress >= 95 {
                    break;
                }
                progress = (progress + 10).min(95);
                emit_progress(&app_clone, progress, "Removing application files...".to_string(), ref_name_clone.clone(), 0.0);
            }
        });

        // Run the transaction (blocking)
        let result = tx.run(Some(&cancellable))
            .map_err(|e| format!("Transaction failed: {}", e));

        *is_complete.lock().unwrap() = true;
        let _ = progress_handle.join();
        
        emit_progress(&app, 100, "Uninstallation complete".to_string(), ref_name.clone(), 0.0);

        app.emit("flatpak-uninstall-complete", serde_json::json!({ "ref": ref_name }))
            .map_err(|e| e.to_string())?;

        result
    }
}
