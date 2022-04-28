#![allow(dead_code, unused)]
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::Error;
use std::path::Path;
use std::sync::{Mutex, Arc};
use std::time::{Duration, Instant};
use std::{fs, thread, mem};
use std::io::Write;

const THRESHOLD_FG: f32 = 4.0;
const THRESHOLD_FCFS: f32 = 7.5;
const SPEEDUP_SIMULATION: u32 = 1; //dont change

#[derive(Debug, Clone)]
struct Job {
        pub id: u64,
        pub remaining_duration: Duration,
        pub duration: Duration,
        pub work_received: Duration,
        pub start: Option<Instant>,
        pub stop: Option<Instant>
}

impl Job {
        pub fn new(v: f32, id: u64) -> Job {
                Job { 
                        id,
                        remaining_duration: Duration::from_secs_f32(v),
                        duration: Duration::from_secs_f32(v),
                        work_received: Duration::from_secs(0),
                        start: None,
                        stop: None
                }
        }

        pub fn started(&mut self) {
                self.start = Some(Instant::now());
        }
        
        pub fn ended(&mut self) {
                self.stop = Some(Instant::now());
        }

        pub fn has_ended(&self) -> bool {
                self.stop.is_some()
        }
}

impl Display for Job {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Job {:<15}, total_duration: {:<15?}, real_time_elapsed: {:<15?}", self.id, self.duration, self.stop.unwrap() - self.start.unwrap())
        }
}
struct ServiceQueue {
        pub foreground: Vec<Job>,
        pub background: Vec<Job>,
        pub fcfs: Vec<Job>,
        pub fcfs_running_job: Option<Job>,
        pub num_jobs: usize,
        pub finished_jobs: Vec<Job>,
}

impl ServiceQueue {
        pub fn new(num_jobs: usize) -> Self {
                Self {
                        foreground: Vec::new(),
                        background: Vec::new(),
                        fcfs: Vec::new(),
                        finished_jobs: Vec::new(),
                        num_jobs,
                        fcfs_running_job: None,
                }
        }

        pub fn add_job(&mut self, mut job: Job) {
                //println!("foreground job -- {}", self.foreground.len());
                job.started();
                self.foreground.push(job);
        }
        
        pub fn add_background_job(&mut self, mut job: Job) {
                //println!("background job -- {}", job.id);
                self.background.push(job);
        }
        
        pub fn add_fcfs_job(&mut self, mut job: Job) {
                //println!("fcfs job -- {}",job.id);
                self.fcfs.push(job);
        }
        
        pub fn add_finished_job(&mut self, mut job: Job, from: &str ) {
                //println!("finished jobs {} ", self.finished_jobs.len());
                //println!("foreground jobs {} ", self.foreground.len());
                //println!("background jobs {} ", self.background.len());
                //println!("fcfs jobs {} ", self.fcfs.len());
                //println!("finished {:<15} -- {:<15?} -- {:<15?} -- from {}", job.id, job.stop.unwrap() - job.start.unwrap(), job.work_received, from);
                self.finished_jobs.push(job);
                if self.finished_jobs.len() % 100 == 0 {
                        println!("{}",self.finished_jobs.len());
                        println!("foreground jobs {} ", self.foreground.len());
                        println!("background jobs {} ", self.background.len());
                        println!("fcfs jobs {} ", self.fcfs.len());
                }
                if self.finished_jobs.len() % 500 == 0 {
                        let mut acc = Duration::from_secs(0);
                        for j in self.finished_jobs.iter() {
                                acc += (j.stop.unwrap() - j.start.unwrap())
                        }
                        println!("########################\nAverage time{:?}\n########################",(acc / self.finished_jobs.len().try_into().unwrap()));
                }
        }

        pub fn tick(&mut self) { //simula 1/10s
                let d = Duration::from_millis(100);
                thread::sleep(d);
                if self.fcfs_running_job.is_some() {
                        let mut job  = self.fcfs_running_job.take().unwrap();
                        if job.remaining_duration < d {
                                job.ended();
                                self.fcfs_running_job = None;
                                self.add_finished_job(job, "fcfs")
                        }
                        else {
                                job.remaining_duration -= d;
                                job.work_received += d;
                                self.fcfs_running_job = Some(job);
                        }
                }
                else if !self.foreground.is_empty() {
                        let list = std::mem::take(&mut self.foreground);
                        let len = list.len();             
                        for mut job in list {
                                if job.remaining_duration < d {
                                        job.ended();
                                        self.add_finished_job(job, "fg")
                                }
                                else {
                                        let nd = (d / len.try_into().unwrap());
                                        job.remaining_duration -= nd;
                                        job.work_received += nd;
                                        if job.work_received > Duration::from_secs_f32(THRESHOLD_FG) {
                                                self.add_background_job(job)
                                        }
                                        else {
                                                self.foreground.push(job)
                                        }
                                }
                        }
                }
                else if !self.background.is_empty() {
                        let list = std::mem::take(&mut self.background);
                        let len = list.len();           
                        for mut job in list {
                                if job.remaining_duration < d {
                                        job.ended();
                                        self.add_finished_job(job, "bg")
                                }
                                else {
                                        let nd = (d / len.try_into().unwrap());
                                        job.remaining_duration -= nd;
                                        job.work_received += nd;
                                        if job.work_received > Duration::from_secs_f32(THRESHOLD_FCFS) {
                                                self.add_fcfs_job(job)
                                        }
                                        else {
                                                self.background.push(job)
                                        }
                                }
                        }
                }
                else if !self.fcfs.is_empty() {
                        let mut job  = self.fcfs.remove(0);
                        if job.remaining_duration < d {
                                job.ended();
                                self.add_finished_job(job, "fcfs")
                        }
                        else {
                                job.remaining_duration -= d;
                                job.work_received += d;
                                self.fcfs_running_job = Some(job);
                        }
                }
        }

        pub fn is_finished(&self) -> bool {
                self.finished_jobs.len() == self.num_jobs
        }
}

#[derive(Debug)]
struct Traffic {
        pub jobs: Vec<Job>,
        pub inter_arrival_times: Vec<f32>
}

impl Traffic {
        pub fn generate_data() -> Result<Self,Error> {
                let path = Path::new(&std::env::current_dir().unwrap()).join("src").join("jobs.csv");
                let mut id: u64 = 0;
                let v: Vec<Job> = fs::read_to_string(path)?.split(',').map(|s| {
                        id +=1;
                        Job::new(s.parse::<f32>().unwrap_or(1_f32), id )
                }).collect();
                
                let path = Path::new(&std::env::current_dir().unwrap()).join("src").join("times.csv");
                let t: Vec<f32> = fs::read_to_string(path)?.split(',').map(|s| {
                        s.parse::<f32>().unwrap_or(1_f32)
                }).collect();

                const START: usize = 0;
                const END: usize = 4000;

                let  ret = Self { jobs: v[START..END].to_vec(), inter_arrival_times: t[START..END].to_vec() };
                Ok(ret)
        }
}

struct Simulation {
        durations: Vec<Duration>,
}

impl Simulation {

        pub fn new() -> Self {
                Self { durations: Vec::new() }
        }

        pub fn run_simulation(&mut self)  -> Result<(), Error> {
                let mut traffic = Traffic::generate_data()?;

                let mut queue = Arc::new(Mutex::new(ServiceQueue::new(traffic.jobs.len())));
                let creator_queue = Arc::clone(&queue);
                let service_queue = Arc::clone(&queue);

                let jobs_creator = thread::spawn(move || {
                        for t in traffic.inter_arrival_times {
                                thread::sleep(Duration::from_secs_f32(t) );
                                (*creator_queue.lock().unwrap()).add_job(traffic.jobs.pop().unwrap());
                                thread::sleep(Duration::from_nanos(1));
                        }
                });

                let service = thread::spawn(move || {
                        while !(*service_queue.lock().unwrap()).is_finished() {
                                for i in 0..10 { //1s in blocchi da 100ms
                                        (*service_queue.lock().unwrap()).tick(); 
                                        thread::sleep(Duration::from_nanos(1));
                                } 
                        }
                });

                jobs_creator.join();
                println!("Ended adding to queue");

                service.join();
                println!("After service join");


                let path = Path::new(&std::env::current_dir().unwrap()).join("src").join("10k_output.csv");
                let mut file = OpenOptions::new()
                                                        .write(true)
                                                        .append(true)
                                                        .create(true)
                                                        .open(path)?;
                
                let jobs = &*queue.lock().unwrap().finished_jobs;
                for j in jobs.iter() {
                        //println!("{}", j);
                        writeln!(file, "{}", j);
                        self.durations.push((j.stop.unwrap() - j.start.unwrap()) * SPEEDUP_SIMULATION )
                }
                Ok(())
        }

        pub fn get_average_duration(&self) -> Duration {
                let len: u32 = self.durations.len().try_into().unwrap();
                let mut acc = Duration::from_secs(0);
                for d in self.durations.iter() {
                        acc += *d;
                } 
                (acc / len)
        }
}

fn main() -> Result<(),Error> {
        let mut simulation = Simulation::new();
        simulation.run_simulation()?;
        println!("{:?}", simulation.get_average_duration());
        Ok(())       
}


// ro = 0.9 -> lambda / mu 
// lambda = 0.9s ->  mu = 1s
