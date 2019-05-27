use crate::cx::*;
use std::io::prelude::*;
use std::fs::File;
use std::io;
use std::net::TcpStream;
use time::precise_time_ns;

#[derive(Clone)]
pub struct CxDesktop{
    pub file_read_id:u64,
    pub file_read_requests:Vec<FileReadRequest>,
    pub profiler_list: Vec<u64>,
    pub profiler_totals: Vec<u64>
}

impl Default for CxDesktop{
    fn default()->CxDesktop{
        CxDesktop{
            file_read_id:1,
            file_read_requests:Vec::new(),
            profiler_list:Vec::new(),
            profiler_totals:Vec::new()
        }
    }
}

impl Cx{

    pub fn read_file(&mut self, path:&str)->FileReadRequest{
        let desktop = &mut self.platform.desktop;
        desktop.file_read_id += 1;
        let read_id = desktop.file_read_id;
        let file_read_req = FileReadRequest{
            read_id:read_id, 
            path:path.to_string()
        };
        desktop.file_read_requests.push(file_read_req.clone());
        file_read_req
    }

    pub fn write_file(&mut self, path:&str, data:&[u8])->u64{
        // just write it right now
        if let Ok(mut file) = File::create(path){
            if let Ok(_) = file.write_all(&data){
            }
            else{
                println!("ERROR WRITING FILE {}", path);
            }
        }
        else{
            println!("ERROR WRITING FILE {}", path);
        }
        0
    }


    pub fn process_desktop_file_read_requests<F>(&mut self, mut event_handler:F)
    where F: FnMut(&mut Cx, &mut Event)
    {
        if self.platform.desktop.file_read_requests.len() == 0{
            return
        }

        let file_read_requests = self.platform.desktop.file_read_requests.clone();
        self.platform.desktop.file_read_requests.truncate(0);

        for read_req in file_read_requests{
            let file_result = File::open(&read_req.path);
            if let Ok(mut file) = file_result{
                let mut buffer = Vec::new();
                // read the whole file
                if file.read_to_end(&mut buffer).is_ok(){
                    event_handler(self, &mut Event::FileRead(FileReadEvent{
                        read_id:read_req.read_id,
                        data:Ok(buffer)
                    }))
                }
                else{ 
                    event_handler(self, &mut Event::FileRead(FileReadEvent{
                        read_id:read_req.read_id,
                        data:Err(format!("Failed to read {}", read_req.path))
                    }))
                }
            }
            else{
                event_handler(self, &mut Event::FileRead(FileReadEvent{
                    read_id:read_req.read_id,
                    data:Err(format!("Failed to open {}", read_req.path))
                }))
            }
        }

        if self.platform.desktop.file_read_requests.len() != 0{
            self.process_desktop_file_read_requests(event_handler);
        }
    }

    pub fn process_to_wasm<F>(&mut self, _msg:u32, mut _event_handler:F)->u32{
        0
    }

    pub fn load_binary_deps_from_file(&mut self){
        let len = self.fonts.len();
        for i in 0..len{
            let resource_name = &self.fonts[i].name.clone();
            // lets turn a file into a binary dep
            let file_result = File::open(&resource_name);
            if let Ok(mut file) = file_result{
                let mut buffer = Vec::<u8>::new();
                // read the whole file
                if file.read_to_end(&mut buffer).is_ok(){
                    // store it in a bindep
                    let mut bin_dep = BinaryDep::new_from_vec(resource_name.clone(), buffer);
                    let _err = self.load_font_from_binary_dep(&mut bin_dep);

                    //     println!("Error loading font {} ", resource_name);
                    //};
                }
            }
            else{
                println!("Error loading font {} ", resource_name);
            }
        }
    }

    /*pub fn log(&mut self, val:&str){
        let mut stdout = io::stdout();
        let _e = stdout.write(val.as_bytes());
        let _e = stdout.flush();
    }*/

    pub fn write_log(data: &str) {
        let _ = io::stdout().write(data.as_bytes());
        let _ = io::stdout().flush();
    }

    pub fn http_send(&self, verb:&str, path:&str, domain:&str, port:&str, body:&str){
        let host = format!("{}:{}",domain,port);
        let stream = TcpStream::connect(&host);
        if let Ok(mut stream) = stream{
            let byte_len = body.as_bytes().len();
            let data = format!("{} /{} HTTP/1.1\r\nHost: {}\r\nConnect: close\r\nContent-Length:{}\r\n\r\n{}", verb, path, domain, byte_len, body);
            if let Err(e) = stream.write(data.as_bytes()){
                println!("http_send error writing stream {}", e);
            }
        }
        else{
             println!("http_send error connecting");
        }
    }

    pub fn profile_clear(&mut self) {
        self.platform.desktop.profiler_totals.truncate(0);
    }
    
    pub fn profile_report(&self) {
        let desktop = &self.platform.desktop;
        println!("-----------------------  Profile Report -------------------------");
        let mut all = 0;
        for (id, total) in desktop.profiler_totals.iter().enumerate() {
            all += total;
            println!("Profile Id:{} time:{} usec", id, total / 1_000);
        }
        println!("Profile total:{} usec", all / 1_000);
    }
    
    pub fn profile_begin(&mut self, id: usize) {
        let desktop = &mut self.platform.desktop;
        while desktop.profiler_list.len() <= id {
            desktop.profiler_list.push(0);
        }
        desktop.profiler_list[id] = precise_time_ns();
    }
    
    pub fn profile_end(&mut self, id: usize) {
        let desktop = &mut self.platform.desktop;
        let delta = precise_time_ns() - desktop.profiler_list[id];
        while desktop.profiler_totals.len() <= id {
            desktop.profiler_totals.push(0);
        }
        desktop.profiler_totals[id] += delta;
    }
    
}