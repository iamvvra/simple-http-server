use std::io::{self, BufRead, BufReader};
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use threadpool::ThreadPool;

fn main() -> std::io::Result<()> {
    let cores: usize = num_cpus::get();
    let pool = ThreadPool::new(cores * 2);
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("server started");
    for client in listener.incoming() {
        match client {
            Ok(mut stream) => pool.execute(move || {
                println!("handling the req in, {:?}", thread::current().id());
                handle_client(&stream);
            }),
            Err(e) => println!("err, {:?}", e),
        }
    }

    Ok(())
}

fn handle_client(stream: &TcpStream) {
    match read_request(stream) {
        Err(e) => println!("{:?}", e),
        Ok(data) => {
            let lines: Vec<&str> = data.split("\r\n").collect();
            // ignore other lines as of now
            let resource = find_request_uri(lines[0]);
            // respond(stream, resource);
            println!("req: {:?}", resource);
        }
    }
}

fn find_request_uri(uri: &str) -> &str {
    let request_str: Vec<&str> = uri.split(' ').collect();
    &request_str[1]
}

fn read_request(mut stream: &TcpStream) -> io::Result<String> {
    let mut reader = BufReader::new(&mut stream);

    let received: Vec<u8> = reader.fill_buf()?.to_vec();

    reader.consume(received.len());

    String::from_utf8(received)
        // .map(|msg| println!("{}", msg))
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Couldn't parse received string as utf8",
            )
        })
}

#[allow(dead_code)]
fn get_current_dir<'a>() -> io::Result<std::ffi::OsString> {
    let dir = std::env::current_dir()?;
    let path = dir.as_os_str();
    Ok(path.to_owned())
}
