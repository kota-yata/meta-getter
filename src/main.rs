use std::{net::{TcpListener, TcpStream}, io::{Read, Write}, thread, str::FromStr};
use httparse::{self, Request};
use reqwest;
use json::object;
// use futures::executor::block_on;

fn main() {
  let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
  for stream in listener.incoming() {
    let stream = stream.unwrap();
    thread::spawn(|| {
      handle_connection(stream);
    });
  }
}

fn handle_connection(mut stream: TcpStream) {
  let mut buffer = [0; 1024];
  stream.read(&mut buffer).unwrap();
  let mut headers = [httparse::EMPTY_HEADER; 16];
  let mut req = Request::new(&mut headers);
  let parse_result = Request::parse(&mut req, &buffer);
  match parse_result {
    Err(err) => panic!("{}", err),
    Ok(x) => println!("Parse Status: {:#?}", x)
  }
  let path = req.path.unwrap();
  let (is_query_found, query_string) = find_query(path, "url");
  if !is_query_found { panic!("Query not found") }
  let data = fetch(&query_string);
  let contents = object!{
    thumbnail: data
  };
  let response = format!(
    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Language: en-US\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Request-Method: GET\r\n\r\n{}",
    contents
  );
  let write_result = stream.write(response.as_bytes());
  match write_result {
    Err(err) => panic!("{}", err),
    Ok(_) => stream.flush().unwrap()
  }
}

fn find_query(path: &str, query_name: &str) -> (bool, String) {
  let splitted: Vec<&str> = path.split("?").collect();
  if splitted.len() < 2 {
    return (false, String::from_str("").unwrap());
  }
  let mut queries = splitted[1].split("&");
  let query_url = queries.find(|query| {
    let kv: Vec<&str> = query.split("=").collect();
    kv[0] == query_name
  });
  let result = match query_url {
    None => (false, String::from_str("").unwrap()),
    Some(x) => {
      let kv: Vec<&str> = x.split("=").collect();
      let result = String::from_str(&kv[1]).unwrap();
      (true, result)
    }
  };
  result
}

fn fetch(url: &str) -> String {
  let res = reqwest::blocking::get(url).unwrap().text().unwrap();
  let splitted: Vec<&str> = res.split("</head>").collect();
  let header = splitted[0];
  String::from_str(header).unwrap()
}
