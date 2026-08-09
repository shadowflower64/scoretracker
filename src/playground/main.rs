use ctrlc::{self};
use scoretracker::config::Config;
use scoretracker::data::game::game_instance_from_id;
use scoretracker::data::library::scan_full;
use scoretracker::hive::job::AnyJob;
use scoretracker::hive::queue::TaskQueue;
use scoretracker::hive::task::{Task, TaskState};
use scoretracker::hive::worker::data::WorkerInfo;
use scoretracker::info;
use scoretracker::log_fn_name;
use scoretracker::util::filelocked::FileLockableDataDefault;
use scoretracker::util::timestamp::NsTimestamp;
use scoretracker::util::{file_ex::FileEx, lockfile::LockfileHandle};
use serde::{Deserialize, Serialize};
use std::env::args;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::Path;
use std::process;
use std::process::exit;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;
use std::time::SystemTime;

#[allow(unused)]
pub fn test_lockfile(_args: &[String]) {
    #[derive(Deserialize, Serialize, Default, Debug)]
    pub struct TestCounter {
        pub counter: i32,
    }

    log_fn_name!("playground:lockfile");

    let sigint = Arc::new(AtomicBool::new(false));
    let sigint_clone = sigint.clone();
    ctrlc::set_handler(move || {
        info!("\nreceived Ctrl+C!");
        sigint_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let lockfile = LockfileHandle::acquire_wait("playground/test_counter.json", None).expect("cannot create lock");

    // Reading
    let mut test_counter: TestCounter = lockfile.read_from_json().unwrap().unwrap_or_default();
    info!("test counter contents after reading: {}", test_counter.counter);

    // Processing
    info!("dummy line");
    for i in 0..30 {
        info!("\x1b[2K\x1b[1Aprocessing{}", ".".repeat(i));
        if sigint.load(Ordering::SeqCst) {
            break;
        }
        sleep(Duration::from_millis(100));
    }
    test_counter.counter += 1;

    // Writing
    info!("test counter contents before writing: {}", test_counter.counter);
    lockfile.write_as_json(test_counter).unwrap();

    // lockfile should get dropped here
    drop(lockfile);

    if sigint.load(Ordering::Relaxed) {
        exit(130); // SIGINT
    }
}

#[allow(unused)]
pub fn test_scanning(args: &[String]) {
    log_fn_name!("playground:scanning");

    let library_dir = Path::new(args.get(1).expect("library dir path not provided"));
    let shared_data_repo_path = Config::load().expect("invalid config").shared_data_repo_path;
    let library_db_path = Path::new("playground/test_library_database.json");
    scan_full(library_dir, library_db_path, None);
}

#[allow(unused)]
pub fn test_timestamp(_args: &[String]) {
    #[derive(Deserialize, Serialize, Debug)]
    pub struct TestTimestamp {
        pub system_time: SystemTime,
        pub ns: NsTimestamp,
        pub ns_system_time: SystemTime,
        pub ns_string: String,
        pub ns_string_local: String,
        pub ns_string_utc: String,
    }

    let system_time = SystemTime::now();
    let ns = NsTimestamp::from(system_time);
    let ns_system_time: SystemTime = ns.try_into().unwrap();
    let ns_string = ns.to_string();
    let ns_string_local = ns.to_date_time_string_local();
    let ns_string_utc = ns.to_date_time_string_utc();
    let test_timestamp = TestTimestamp {
        system_time,
        ns,
        ns_system_time,
        ns_string,
        ns_string_local,
        ns_string_utc,
    };
    Path::new("playground/test_timestamp.json")
        .write_as_json_pretty(test_timestamp)
        .unwrap();
}

#[allow(unused)]
pub async fn test_queue(_args: &[String]) {
    log_fn_name!("playground:queue");

    let mut currently_doing_task_opt = None;

    // Read the queue to either add something or take on a task
    let mut queue = TaskQueue::lock_and_read_or_default("playground/test_queue.jsonl", None).expect("couldn't read queue");
    let task_todo_opt = queue.top_queued_task_mut();
    if let Some(task_todo) = task_todo_opt {
        // Take on a task
        task_todo.state = TaskState::Working;
        task_todo.start_timestamp = Some(NsTimestamp::now());
        task_todo.worker_info = Some(WorkerInfo {
            address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
            birth_timestamp: 0.into(),
            name: "fake-playground-worker".to_string(),
            pid: process::id(),
        });
        task_todo.comment = Some(String::from("this job was started by the playground"));
        currently_doing_task_opt = Some(task_todo.clone());
        info!("started working on task {}", task_todo.uuid.0);
    } else {
        info!("no tasks to do; adding task for next time");

        // Add a new task
        let new_task = Task::new(
            String::from("Test task from playground"),
            AnyJob::DisplayMessageAndSleep {
                message: String::from("Hello world from the queue!"),
                time_nanos: 3_000_000_000,
            },
        );
        queue.add_task(new_task);
    }
    queue.save_to_file();
    drop(queue);
    // Drop here to unlock the lockfile

    // Do some task if there is something to do
    if let Some(mut task) = currently_doing_task_opt {
        // match task.job.run(&Config::load().unwrap(), None).await {
        //     Ok(results) => {
        //         //
        //         task.state = TaskState::Done
        //     }
        //     Err(error) => {
        //         //
        //         task.state = TaskState::Failed
        //     }
        // }
        // task.finish_timestamp = Some(NsTimestamp::now());

        // Read the queue file again to update the state of the task
        let mut queue = TaskQueue::lock_and_read_or_default("playground/test_queue.jsonl", None).expect("couldn't read queue");
        queue.update_task(task);
        queue.save_to_file();
    }
}

#[allow(unused)]
pub fn test_result(_args: &[String]) {
    #[derive(Deserialize, Serialize, Debug)]
    pub struct TestResult {
        pub a: NsTimestamp,
        pub result: Result<u64, NsTimestamp>,
    }

    let a = NsTimestamp::now();
    let test_result_1 = TestResult {
        a,
        result: a.as_millis().try_into().map_err(|_| a),
    };

    let a = NsTimestamp::MAX;
    let test_result_2 = TestResult {
        a,
        result: a.as_millis().try_into().map_err(|_| a),
    };

    Path::new("playground/test_result.json")
        .write_as_json_pretty(vec![test_result_1, test_result_2])
        .unwrap();
}

#[allow(unused)]
pub fn ask_edit(args: &[String]) {
    log_fn_name!("playground:ask_edit");

    let game_id = args.get(1).expect("no argument provided");
    let game = game_instance_from_id(game_id).unwrap_or_else(|| panic!("unknown game: {}", game_id));

    let mut perf = game.ask_for_performance_new().unwrap();
    println!("{:#?}", perf);

    perf.ask_for_performance_edit().unwrap();
    println!("{:#?}", perf);
}

fn main() {
    #[allow(unused)]
    let args: Vec<_> = args().collect();

    // test_lockfile(&args);
    // test_scanning(&args);
    // test_timestamp(&args);
    // test_queue(&args);
    // test_result(&args);
    // ask_edit(&args);
}
