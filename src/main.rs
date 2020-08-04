use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use threadpool::ThreadPool;

fn main() -> std::io::Result<()> {
    let cores: usize = num_cpus::get();
    let pool = ThreadPool::new(cores * 2);
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("server started");
    for client in listener.incoming() {
        match client {
            Ok(stream) => pool.execute(|| {
                println!("handling the req in, {:?}", thread::current().id());
                match handle_client(stream) {
                    Err(e) => println!("error handling client, {:?}", e),
                    Ok(_) => {}
                }
            }),
            Err(e) => println!("err, {:?}", e),
        }
    }
    Ok(())
}

fn handle_client(stream: TcpStream) -> io::Result<()> {
    let root_dir = std::env::current_dir().expect("Error getting the current dir");
    match read_request(&stream) {
        Err(_) => println!("error handling"),
        Ok(data) => {
            let lines: Vec<&str> = data.split("\r\n").collect();
            // ignore other lines as of now
            let mut res = root_dir.to_str().unwrap().to_owned();
            let resource = find_request_uri(lines[0]);
            res.push_str(resource);
            println!("req, {:?} -> is served with {:?}", resource, res);
            handle_write(stream, res)?;
        }
    }
    Ok(())
}

fn handle_write(mut stream: TcpStream, res: String) -> io::Result<()> {
    let content = std::fs::read_to_string(res)?;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}\r\n",
        content
    );
    stream.write(response.as_bytes())?;
    Ok(())
}

fn find_request_uri(uri: &str) -> &str {
    let request_str: Vec<&str> = uri.split(' ').collect();
    &request_str[1]
}

fn read_request(mut stream: &TcpStream) -> io::Result<String> {
    let mut reader = BufReader::new(&mut stream);

    let received: Vec<u8> = reader.fill_buf()?.to_vec();

    reader.consume(received.len());

    String::from_utf8(received).map_err(|_| {
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
