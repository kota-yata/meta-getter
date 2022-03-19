use meta_getter::ThreadPool;
use std::{net::{TcpListener, TcpStream}, io::{Read, Write}, thread, str::FromStr, io::{Result, Error, ErrorKind}};
use httparse::{self, Request};
use reqwest;
use html_parser::Dom;
// use futures::executor::block_on;

fn main() {
  let listener = TcpListener::bind(format!("0.0.0.0:{}", std::env::var("PORT").unwrap_or("8000".to_string()))).unwrap();
  // let pool = ThreadPool::new(4);
  for stream in listener.incoming() {
    let stream = stream.unwrap();
    // pool.execute(|| {
    //   handle_connection(stream);
    // });
    handle_connection(stream);
  }
}

fn handle_connection(mut stream: TcpStream) {
  let mut buffer = [0; 1024];
  stream.read(&mut buffer).unwrap();
  let mut headers = [httparse::EMPTY_HEADER; 32];
  let mut req = Request::new(&mut headers);
  println!("req: {:#?}", req);
  let parse_result = Request::parse(&mut req, &buffer);
  match parse_result {
    Ok(x) => println!("{:#?}", x),
    Err(x) => println!("Not working!!! : {}", x)
  }
  let path = match req.path {
    None => {
      let empty_response = Vec::from([String::from_str("Path not found").unwrap()]);
      response(stream, 200, empty_response).unwrap();
      panic!("Path not found")
    },
    Some(x) => x
  };
  let (is_query_found, query_string) = find_query(path, "url");
  if !is_query_found {
    let empty_response = Vec::from([String::from_str("Query not found").unwrap()]);
    response(stream, 200, empty_response).unwrap();
    // panic!("Query not found")
    return;
  }
  let data = match fetch(&query_string) {
    Err(_) => {
      let empty_response = Vec::from([String::from_str("Invalid URL").unwrap()]);
      response(stream, 200, empty_response).unwrap();
      return;
    },
    Ok(x) => x
  };
  let result_vec = match find_meta(&data) {
    None => panic!("Meta tag not found"),
    Some(x) => x
  };
  match response(stream, 200, result_vec) {
    Err(err) => panic!("{:#?}", err),
    Ok(_) => println!("Successfully responsed")
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

fn fetch(url: &str) -> Result<String> {
  match reqwest::blocking::get(url) {
    Err(err) => return Err(Error::new(ErrorKind::Other, err.to_string())),
    Ok(data) => {
      let res = data.text().unwrap();
      let splitted: Vec<&str> = res.split("</head>").collect();
      let header = splitted[0];
      return Ok(String::from_str(header).unwrap());
    }
  };
}

fn find_meta(data: &String) -> Option<Vec<String>> {
  let parsed = match Dom::parse(data) {
    Err(err) => panic!("{:#?}", err),
    Ok(result) => result
  };
  let json = match parsed.to_json() {
    Err(err) => panic!("{:#?}", err),
    Ok(result) => json::parse(&result).unwrap()
  };
  let json_tree = &json["children"];
  let mut result: Vec<String> = Vec::new();
  for el in 0..json_tree.len() - 1 {
    if json_tree[el]["name"] != "meta" {
      continue;
    }
    result.push(json_tree[el]["attributes"].to_string())
  }
  match result.len() {
    0 => None,
    _ => Some(result)
  }
}

fn response(mut stream: TcpStream, status: u16, data: Vec<String>) -> Result<&'static str> {
  let response = format!(
    "HTTP/2.0 {}\r\nContent-Type: application/json\r\nContent-Language: en-US\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Request-Method: GET\r\n\r\n{:#?}\n",
    status,
    data
  );
  let write_result = stream.write(response.as_bytes());
  match write_result {
    Err(err) => Err(err),
    Ok(_) => {
      stream.flush().unwrap();
      Ok("OK")
    }
  }
}

// TODO
// [x] Return error message to clients when [ query wasn't found, url was invalid ]
// [x] Make a function splitting up meta tag and converting them into object
// [ ] Graceful shutdown and cleanup
