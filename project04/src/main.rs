use std::env;
use std::fs;
use std::path::Path;
use std::thread;
use std::path::PathBuf;
use std::sync::mpsc;

// use this if depending on local crate
use libsteg;



fn main() -> Result<(), StegError> {

    //prepare arguments and check if proper amount are provided
    let args: Vec<String> = env::args().collect();//arguments
    let thread_count = &args[1];//establish thread count

    //check proper arguments length
    if args.len()!=3 {
        eprintln!("You need to give 2 arguments");
        return Ok(())
    }
    
    match args.len() {
        3 => {
            
            let thread_count = thread_count.parse::<usize>().unwrap();//parse usize from thread count

            //path from second argument 
            let path_string = args[2].to_string();//read from this directory
            let path = Path::new(&path_string);// path from directory

            //vector for storing threads also mpsc channels
            let mut handles = vec![];
            let (sender, receiver) = mpsc::channel();

            //list of files
            let mut file_list: Vec<PathBuf> = Vec::new();
            
            let mut num_files = 0;//number of files
            //sorting for only ppm files
            for entry in fs::read_dir(path).expect("Path not found!") {
                let entry = entry.expect("Valid entry not found!");
                let path = entry.path();
                if path.extension().unwrap() == "ppm" {
                    file_list.push(path);
                    num_files+=1;
                }
                
            }

            //for each thread
            for i in 0..thread_count{

                let tx = sender.clone();//clone the send channel
                let mut job_list: Vec<String> = Vec::new();//initialize job list
                let decimal_length: f64 = file_list.len() as f64;
                let interval = (decimal_length/thread_count as f64).ceil();
                let interval: usize = interval as usize; //determine interval size
                let start =  interval*i; //determine start index for this threads jobs
                let mut last_index = start+interval; //set last index as interval distance from start
                if last_index>=file_list.len()-1 {last_index=file_list.len()-1;} // if last is greater than number of files, set to number of files -1
                
                let mut counter = start;//counter for which job to add

                //until the job list is of properlength(), add jobs
                while job_list.len()<interval{
                    if counter >= last_index {break;}//if counter is greater than index, dont' add
                    job_list.push(file_list[counter].clone().into_os_string().into_string().unwrap());//push the path to the job list
                    counter+=1;//increment
                }

                //spawn a thread
                let handle = thread::spawn(move||{

                    //while jobs are remaining
                    while job_list.len()!=0{

                        //create ppm file from job
                        let ppm = match libsteg::PPM::new(job_list[job_list.len()-1].clone()) {
                            Ok(ppm) => ppm,
                            Err(err) => panic!("Error: {:?}", err),
                         };
                        let decoded:String = decode_message(&ppm.pixels).unwrap();//decode the string
                        let payload = (job_list[job_list.len()-1].clone(),decoded);//create file and decoded message for payload
                        tx.send(payload).unwrap();//send the payload
                        job_list.pop();//take the job off the list
                    }
                });
                handles.push(handle);//add the thread to the group of handles
            }


            //vector of return values, for each file wait for decoded message and add to vector
            let mut returns = Vec::new();
            for _handle in 0..num_files-1 {
                let value = receiver.recv().unwrap();
                returns.push(value.clone());
            }

            //wait for each thread
            for thread in handles{thread.join().unwrap();}

            let mut final_string: String = String::from("");//output string
            returns.sort();//sort the returns by file name
            for r in returns{
                final_string = format!("{}{}",final_string,r.1);//format to add each message to output string
            }
            println!("{}\n",final_string);//print out output string
        }
        _ => println!("You need to give 2 or 4 arguments!"),
    }
    Ok(())
}