use actix_web::{
    get, post, App, HttpResponse, HttpServer, Responder,
    dev::{ServiceResponse, ServiceRequest, Service},
    http::header::{HeaderName, HeaderValue},
    body::MessageBody,
    Error,
};

use actix_web_lab::middleware::{from_fn, Next};

use crc::{Crc, CRC_32_ISO_HDLC};
pub const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

// See more examples of from_fn middleware here:
// https://github.com/robjtede/actix-web-lab/blob/main/actix-web-lab/examples/from_fn.rs
async fn add_crc32_header_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    let res = next.call(req).await.unwrap();

    let (rq, mut rs) = res.into_parts();

    let body = match rs.body().try_into_bytes() {
        Ok(bytes) => bytes,
        _ => return Err(actix_web::error::ErrorRequestTimeout("")),
    };
    let crc = CRC32.checksum(&body);
    let hash = format!("{:x}", crc);

    let hdrs = rs.headers_mut();
    hdrs.insert( HeaderName::from_static("x-crc32"), HeaderValue::from_str(&hash).unwrap());

    Ok(ServiceResponse::new(rq,rs))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(from_fn(add_crc32_header_middleware))
            .service(hello)
            .service(echo)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
