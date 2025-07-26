use super::utils::{adapt_request, convert_header};
use actix_web::{
  body::{BodyStream, BoxBody},
  http::StatusCode,
  web::{Data, *},
  HttpRequest,
  HttpResponse,
  Responder,
};
use lemmy_api_utils::context::FastJobContext;
use lemmy_db_schema::source::images::RemoteImage;
use lemmy_db_views_local_image::api::{ImageGetParams, ImageProxyParams};
use lemmy_utils::error::FastJobResult;
use moka::future::Cache;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::sync::LazyLock;
use url::Url;

// Cache for image responses to reduce the number of HTTP requests
// Cache up to 1000 images for 10 minutes
static IMAGE_CACHE: LazyLock<Cache<String, Vec<u8>>> = LazyLock::new(|| {
    Cache::builder()
        .max_capacity(1000)
        .time_to_live(std::time::Duration::from_secs(600))
        .build()
});

pub async fn get_image(
  filename: Path<String>,
  Query(params): Query<ImageGetParams>,
  req: HttpRequest,
  context: Data<FastJobContext>,
) -> FastJobResult<HttpResponse> {
  let name = &filename.into_inner();

  // If there are no query params, the URL is original
  let pictrs_url = context.settings().pictrs()?.url;
  let processed_url = if params.file_type.is_none() && params.max_size.is_none() {
    format!("{}image/original/{}", pictrs_url, name)
  } else {
    let file_type = file_type(params.file_type, name);
    let mut url = format!("{}image/process.{}?src={}", pictrs_url, file_type, name);

    if let Some(size) = params.max_size {
      url = format!("{url}&thumbnail={size}",);
    }
    url
  };

  do_get_image(processed_url, req, &context).await
}

pub async fn image_proxy(
  Query(params): Query<ImageProxyParams>,
  req: HttpRequest,
  context: Data<FastJobContext>,
) -> FastJobResult<Either<HttpResponse<()>, HttpResponse<BoxBody>>> {
  let url = Url::parse(&params.url)?;
  let encoded_url = utf8_percent_encode(&params.url, NON_ALPHANUMERIC).to_string();

  // Check that url corresponds to a federated image so that this can't be abused as a proxy
  // for arbitrary purposes.
  RemoteImage::validate(&mut context.pool(), url.clone().into()).await?;

  let pictrs_config = context.settings().pictrs()?;
  let processed_url = if params.file_type.is_none() && params.max_size.is_none() {
    format!("{}image/original?proxy={}", pictrs_config.url, encoded_url)
  } else {
    let file_type = file_type(params.file_type, url.path());
    let mut url = format!(
      "{}image/process.{}?proxy={}",
      pictrs_config.url, file_type, encoded_url
    );

    if let Some(size) = params.max_size {
      url = format!("{url}&thumbnail={size}",);
    }
    url
  };

  let bypass_proxy = pictrs_config
    .proxy_bypass_domains
    .iter()
    .any(|s| url.domain().is_some_and(|d| d == s));
  if bypass_proxy {
    // Bypass proxy and redirect user to original image
    Ok(Either::Left(Redirect::to(url.to_string()).respond_to(&req)))
  } else {
    // Proxy the image data through Lemmy
    Ok(Either::Right(
      do_get_image(processed_url, req, &context).await?,
    ))
  }
}

pub(super) async fn do_get_image(
  url: String,
  req: HttpRequest,
  context: &FastJobContext,
) -> FastJobResult<HttpResponse> {
  // Check if the image is in the cache
  if let Some(cached_data) = IMAGE_CACHE.get(&url).await {
    let mut client_res = HttpResponse::build(StatusCode::OK);
    
    // Set content type based on the file extension or default to image/jpeg
    let content_type = if url.ends_with(".png") {
      "image/png"
    } else if url.ends_with(".gif") {
      "image/gif"
    } else if url.ends_with(".webp") {
      "image/webp"
    } else if url.ends_with(".svg") {
      "image/svg+xml"
    } else {
      "image/jpeg"
    };
    
    client_res.insert_header(("Content-Type", content_type));
    client_res.insert_header(("Cache-Control", "public, max-age=604800")); // Cache for 1 week
    
    return Ok(client_res.body(cached_data));
  }
  
  // If not in cache, fetch the image
  let mut client_req = adapt_request(&req, url.clone(), context);

  if let Some(addr) = req.head().peer_addr {
    client_req = client_req.header("X-Forwarded-For", addr.to_string());
  }

  let res = client_req.send().await?;

  if res.status() == http::StatusCode::NOT_FOUND {
    return Ok(HttpResponse::NotFound().finish());
  }

  let mut client_res = HttpResponse::build(StatusCode::from_u16(res.status().as_u16())?);

  for (name, value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
    client_res.insert_header(convert_header(name, value));
  }
  
  // For successful responses, store the image in the cache
  if res.status().is_success() {
    // Clone the response to avoid consuming it
    let bytes = res.bytes().await?;
    
    // Store in cache
    IMAGE_CACHE.insert(url, bytes.to_vec()).await;
    
    // Return the response
    return Ok(client_res.body(bytes));
  }

  // For non-successful responses, just return the response without caching
  Ok(client_res.body(BodyStream::new(res.bytes_stream())))
}

/// Take file type from param, name, or use jpg if nothing is given
pub(super) fn file_type(file_type: Option<String>, name: &str) -> String {
  file_type
    .clone()
    .unwrap_or_else(|| name.split('.').next_back().unwrap_or("jpg").to_string())
}
