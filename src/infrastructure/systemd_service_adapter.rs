use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedObjectPath;

use crate::domain::service::Service;
use crate::domain::service_state::ServiceState;
use crate::domain::service_repository::ServiceRepository;
use crate::domain::service_property::{SASBTTUII, ServiceProperty};

type SystemdUnit = (String, String, String, String, String, String, OwnedObjectPath, u32, String, OwnedObjectPath);

pub struct SystemdServiceAdapter;

impl SystemdServiceAdapter {
    fn manager_proxy(&self) -> Result<Proxy<'_>, Box<dyn std::error::Error>> {
        let connection = Connection::system()?;
        let proxy = Proxy::new(
            &connection,
            "org.freedesktop.systemd1",
            "/org/freedesktop/systemd1",
            "org.freedesktop.systemd1.Manager",
        )?;
        Ok(proxy)
    }

    pub fn reload_daemon(&self) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.manager_proxy()?;
        proxy.call::<&str, (), ()>("Reload", &())?; 
        Ok(())
    }

    pub fn get_service_property(&self, name: &str) -> Result<ServiceProperty, Box<dyn std::error::Error>> {
        let manager = self.manager_proxy()?;

        let unit_path: OwnedObjectPath = manager.call("GetUnit", &(name))?;

        let conn = manager.connection();

        let service_proxy = Proxy::new(
            conn,
            "org.freedesktop.systemd1",
            unit_path.as_str(),
            "org.freedesktop.systemd1.Service",
        )?;

        let exec_start: Vec<SASBTTUII> = service_proxy.get_property("ExecStart")?;
        let exec_start_pre: Vec<SASBTTUII> = service_proxy.get_property("ExecStartPre")?;
        let exec_start_post: Vec<SASBTTUII> = service_proxy.get_property("ExecStartPost")?;
        let exec_stop: Vec<SASBTTUII> = service_proxy.get_property("ExecStop")?;
        let exec_stop_post: Vec<SASBTTUII> = service_proxy.get_property("ExecStopPost")?;

        let exec_main_pid: u32 = service_proxy.get_property("ExecMainPID")?;
        let exec_main_start_timestamp: u64 = service_proxy.get_property("ExecMainStartTimestamp")?;
        let exec_main_exit_timestamp: u64 = service_proxy.get_property("ExecMainExitTimestamp")?;
        let exec_main_code: i32 = service_proxy.get_property("ExecMainCode")?;
        let exec_main_status: i32 = service_proxy.get_property("ExecMainStatus")?;

        let main_pid: u32 = service_proxy.get_property("MainPID")?;
        let control_pid: u32 = service_proxy.get_property("ControlPID")?;

        let restart: String = service_proxy.get_property("Restart")?;
        let restart_usec: u64 = service_proxy.get_property("RestartUSec")?;

        let status_text: String = service_proxy.get_property("StatusText")?;
        let result: String = service_proxy.get_property("Result")?;

        let user: String = service_proxy.get_property("User")?;
        let group: String = service_proxy.get_property("Group")?;

        let limit_cpu: u64 = service_proxy.get_property("LimitCPU")?;
        let limit_nofile: u64 = service_proxy.get_property("LimitNOFILE")?;
        let limit_nproc: u64 = service_proxy.get_property("LimitNPROC")?;
        let limit_memlock: u64 = service_proxy.get_property("LimitMEMLOCK")?;
        let memory_limit: u64 = service_proxy.get_property("MemoryLimit")?;
        let cpu_shares: u64 = service_proxy.get_property("CPUShares")?;

        let service_property = ServiceProperty::new(
            exec_start,
            exec_start_pre,
            exec_start_post,
            exec_stop,
            exec_stop_post,

            exec_main_pid,
            exec_main_start_timestamp,
            exec_main_exit_timestamp,
            exec_main_code,
            exec_main_status,

            main_pid,
            control_pid,

            restart,
            restart_usec,

            status_text,
            result,

            user,
            group,

            limit_cpu,
            limit_nofile,
            limit_nproc,
            limit_memlock,
            memory_limit,
            cpu_shares,
        );

        Ok(service_property)
    }
}
impl ServiceRepository for SystemdServiceAdapter {
    fn start_service(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.manager_proxy()?;
        let _job: OwnedObjectPath = proxy.call::<&str, (&str, &str), OwnedObjectPath>("StartUnit", &(name, "replace"))?;
        Ok(())
    }

    fn stop_service(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.manager_proxy()?;
        let _job: OwnedObjectPath = proxy.call::<&str, (&str, &str), OwnedObjectPath>("StopUnit", &(name, "replace"))?;
        Ok(())
    }

    fn restart_service(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.manager_proxy()?;
        let _job: OwnedObjectPath = proxy.call::<&str, (&str, &str), OwnedObjectPath>("RestartUnit", &(name, "replace"))?;
        Ok(())
    }

    fn enable_service(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.manager_proxy()?;
        let (_carries_install_info, _changes): (bool, Vec<(String, String, String)>) = proxy
            .call::<&str, (Vec<&str>, bool, bool), (bool, Vec<(String, String, String)>)>(
                "EnableUnitFiles",
                &(vec![name], false, true),
            )?;

        Ok(())
    }

    fn disable_service(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = self.manager_proxy()?;
        let _changes: Vec<(String, String, String)> = proxy
            .call::<&str, (Vec<&str>, bool), Vec<(String, String, String)>>(
                "DisableUnitFiles",
                &(vec![name], false),
            )?;
        Ok(())
    }

    fn list_services(&self) -> Result<Vec<Service>, Box<dyn std::error::Error>> {
        let proxy = self.manager_proxy()?;

        let units: Vec<SystemdUnit> = proxy.call("ListUnits", &())?;

        let services = units
            .into_iter()
            .filter(|(name, ..)| name.ends_with(".service"))
            .map(|(name, description, load_state, active_state, sub_state, _followed, _object_path, _job_id, _job_type, _job_object)| {
                let state: String = proxy
                    .call("GetUnitFileState", &name)
                    .unwrap_or_else(|_| "unknown".into());

                let service_state = ServiceState::new(load_state, active_state, sub_state, state);

                Service::new(name, description, service_state)
            })
            .collect();

        Ok(services)
    }

    fn get_service_log(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = std::process::Command::new("journalctl")
            .arg("-eu")
            .arg(name)
            .arg("--no-pager")
            .output()?;

        let log = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };

        Ok(log)
    }
}

