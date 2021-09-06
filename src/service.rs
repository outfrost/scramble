use std::convert::Infallible;
use std::sync::{mpsc::Sender, Mutex};
use http::{Method, Response, StatusCode};
use hyper::Body;

pub type Command = (char, char);
type ResponseResult = hyper::Result<Response<Body>>;

pub static mut COMMAND_TX: Option<Mutex<Sender<Command>>> = None;

pub async fn run(command_tx: Sender<Command>) {
	unsafe {
		COMMAND_TX = Some(Mutex::new(command_tx));
	}

	let addr = ([0, 0, 0, 0], 8000).into();

	let make_svc = hyper::service::make_service_fn(|_conn| async {
		Ok::<_, Infallible>(hyper::service::service_fn(process_req))
	});

	let server = hyper::Server::bind(&addr).serve(make_svc);

	if let Err(e) = server.await {
		eprintln!("hyper error: {}", e);
	}
}

async fn process_req(request: http::Request<Body>) -> ResponseResult {
	let (parts, body) = request.into_parts();
	if parts.method != Method::GET {
		return Ok(error_response(
			StatusCode::METHOD_NOT_ALLOWED,
			"Method not allowed\n".into()
		));
	}

	// "/replace/a/with/z"
	let segments: Vec<&str> = parts.uri.path()
		.split('/')
		.filter(|s| !s.is_empty())
		.collect();

	if segments.len() != 4
		|| !segments[0].eq_ignore_ascii_case("replace")
		|| segments[1].len() != 1
		|| !segments[2].eq_ignore_ascii_case("with")
		|| segments[3].len() != 1
	{
		return Ok(error_response(StatusCode::BAD_REQUEST, "Bad request (check your URL format)\n".into()));
	}

	let replace = match segments[1].chars().next() {
		Some(c) => c,
		None => return Ok(error_response(StatusCode::BAD_REQUEST, "Bad request (invalid letter)\n".into())),
	}.to_ascii_uppercase();
	let with = match segments[3].chars().next() {
		Some(c) => c,
		None => return Ok(error_response(StatusCode::BAD_REQUEST, "Bad request (invalid letter)\n".into())),
	}.to_ascii_uppercase();

	if !replace.is_ascii_uppercase() || !with.is_ascii_uppercase() {
		return Ok(error_response(StatusCode::BAD_REQUEST, "Bad request (invalid letter)\n".into()));
	}
	
	if let Some(mutex) = unsafe { &COMMAND_TX } {
		if let Ok(tx) = mutex.lock() {
			tx.send((replace, with));
		}
	}

	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(""))
		.unwrap())
}

fn error_response(status: StatusCode, msg: String) -> Response<Body> {
	Response::builder()
		.status(status)
		.body(Body::from(msg))
		.unwrap()
}
